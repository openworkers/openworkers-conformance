//! Runs the event model battery on the backend this binary was built against.
//!
//!   cargo run --release --features <backend> --bin events -- [--json] [--filter <substr>] [--strict]
//!
//! --strict turns on the backend's strict respondWith, where it has one.
//!
//! A failing case is a result, not a failure, so the exit code says only
//! whether the battery itself ran.

use std::process::ExitCode;

#[cfg(feature = "_js")]
fn main() -> ExitCode {
    use openworkers_conformance::events;

    let mut json = false;
    let mut filter = None;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--filter" => filter = args.next(),
            "--strict" => {
                if !events::strict() {
                    eprintln!("--strict: this backend has no strict switch");

                    return ExitCode::FAILURE;
                }
            }
            other => {
                eprintln!("unknown argument: {other}");

                return ExitCode::FAILURE;
            }
        }
    }

    let run = events::run(filter.as_deref());

    if run.outcomes.is_empty() {
        eprintln!("no case matched the filter");

        return ExitCode::FAILURE;
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&run).unwrap());
    } else {
        print!("{}", run.render());
    }

    ExitCode::SUCCESS
}

/// The cases are JavaScript workers; a wasm backend has nothing to run them on.
#[cfg(not(feature = "_js"))]
fn main() -> ExitCode {
    println!("events: skipped, the backend does not run JavaScript");

    ExitCode::SUCCESS
}

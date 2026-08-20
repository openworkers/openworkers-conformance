//! Runs the streaming battery on the backend this binary was built against.
//!
//!   cargo run --release --features <backend> --bin streaming -- [--json] [--filter <substr>]
//!
//! A backend that buffers is a result, not a failure, so the exit code says
//! only whether the battery itself ran.

use std::process::ExitCode;

use openworkers_conformance::streaming;

fn main() -> ExitCode {
    let mut json = false;
    let mut filter = None;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--filter" => filter = args.next(),
            other => {
                eprintln!("unknown argument: {other}");

                return ExitCode::FAILURE;
            }
        }
    }

    let run = streaming::run(filter.as_deref());

    if run.probes.is_empty() {
        eprintln!("no probe matched the filter");

        return ExitCode::FAILURE;
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&run).unwrap());
    } else {
        print!("{}", run.render());
    }

    // A probe that hung left its thread where it was, and a hung thread would
    // otherwise keep the process alive after the report is printed.
    std::io::Write::flush(&mut std::io::stdout()).ok();
    std::process::exit(0);
}

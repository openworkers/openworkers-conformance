//! Runs the guest suite on the backend this binary was built against.
//!
//!   cargo run --release --features <backend> --bin conformance -- [--json] [--verbose] [--filter <substr>]
//!
//! A guest assertion failure is the measurement, so it never sets the exit
//! code; only a broken suite does.

#[cfg(feature = "_js")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    use openworkers_conformance::suite;
    use std::path::Path;
    use std::process::ExitCode;

    let mut json = false;
    let mut verbose = false;
    let mut filter = None;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--verbose" => verbose = true,
            "--filter" => filter = args.next(),
            other => {
                eprintln!("unknown argument: {other}");

                return ExitCode::FAILURE;
            }
        }
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

    let entries = match suite::collect(&root, filter.as_deref()) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("cannot read {}: {e}", root.display());

            return ExitCode::FAILURE;
        }
    };

    if entries.is_empty() {
        eprintln!("no test files under {}", root.display());

        return ExitCode::FAILURE;
    }

    let suite = suite::run(entries).await;

    if json {
        println!("{}", serde_json::to_string_pretty(&suite).unwrap());
    } else {
        print!("{}", suite.render(verbose));
    }

    ExitCode::SUCCESS
}

#[cfg(not(feature = "_js"))]
fn main() {
    println!(
        "skipped: the {} backend runs wasm components, not JavaScript, so the guest suite does not apply",
        openworkers_conformance::runtime::NAME
    );
}

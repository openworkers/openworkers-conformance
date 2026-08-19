//! Backend selection: one runtime per build, like everything else in the workspace.

use tokio::task::LocalSet;

#[cfg(not(any(
    feature = "v8",
    feature = "jsc",
    feature = "quickjs",
    feature = "boa",
    feature = "wasm"
)))]
compile_error!("no runtime backend selected: build with --features v8|jsc|quickjs|boa|wasm");

#[cfg(any(
    all(
        feature = "v8",
        any(
            feature = "jsc",
            feature = "quickjs",
            feature = "boa",
            feature = "wasm"
        )
    ),
    all(
        feature = "jsc",
        any(feature = "quickjs", feature = "boa", feature = "wasm")
    ),
    all(feature = "quickjs", any(feature = "boa", feature = "wasm")),
    all(feature = "boa", feature = "wasm"),
))]
compile_error!("runtime backends are mutually exclusive: select exactly one");

/// Name the scoreboard is filed under.
#[cfg(feature = "v8")]
pub const NAME: &str = "v8";
#[cfg(feature = "jsc")]
pub const NAME: &str = "jsc";
#[cfg(feature = "quickjs")]
pub const NAME: &str = "quickjs";
#[cfg(feature = "boa")]
pub const NAME: &str = "boa";
#[cfg(feature = "wasm")]
pub const NAME: &str = "wasm";

#[cfg(feature = "boa")]
pub use openworkers_runtime_boa::Worker;
#[cfg(feature = "jsc")]
pub use openworkers_runtime_jsc::Worker;
#[cfg(feature = "quickjs")]
pub use openworkers_runtime_quickjs::Worker;
#[cfg(feature = "v8")]
pub use openworkers_runtime_v8::Worker;

/// The default 50ms of CPU is a production budget; a test file needs more.
#[cfg(feature = "_js")]
pub fn limits() -> openworkers_core::RuntimeLimits {
    openworkers_core::RuntimeLimits {
        max_cpu_time_ms: 10_000,
        max_wall_clock_time_ms: 20_000,
        ..Default::default()
    }
}

/// Guest console output would corrupt `--json`, and it is not what is measured.
#[cfg(feature = "_js")]
pub fn quiet_ops() -> openworkers_core::OperationsHandle {
    struct Quiet;

    impl openworkers_core::OperationsHandler for Quiet {
        fn handle_log(&self, _level: openworkers_core::LogLevel, _message: String) {}
    }

    std::sync::Arc::new(Quiet)
}

/// Runs an async function inside a LocalSet.
/// Required for tests that use spawn_local (tokio 1.48+).
pub async fn run_in_local<F, Fut, T>(f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let local = LocalSet::new();
    local.run_until(f()).await
}

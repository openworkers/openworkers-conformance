use tokio::task::LocalSet;

#[cfg(feature = "v8")]
pub use openworkers_runtime_v8::Worker;

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

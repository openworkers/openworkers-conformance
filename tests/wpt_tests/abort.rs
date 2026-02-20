use openworkers_conformance::harness::run_wpt_test;

#[tokio::test(flavor = "current_thread")]
async fn abort_signal() {
    run_wpt_test("dom/abort/AbortSignal.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn abort_signal_any() {
    run_wpt_test("dom/abort/abort-signal-any.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn abort_event() {
    run_wpt_test("dom/abort/event.any.js").await;
}

// NOTE: dom/abort/timeout.any.js is excluded because AbortSignal.timeout()
// relies on timers that don't resolve in the current test harness, causing hangs.

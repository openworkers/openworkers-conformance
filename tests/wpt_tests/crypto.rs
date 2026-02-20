use openworkers_conformance::harness::run_wpt_test;

// NOTE: getRandomValues.any.js is excluded because the runtime panics natively
// (unwrap on None in random.rs) on certain typed array types the test exercises,
// which kills the entire test process via SIGABRT.

#[tokio::test(flavor = "current_thread")]
async fn historical() {
    run_wpt_test("WebCryptoAPI/historical.any.js").await;
}

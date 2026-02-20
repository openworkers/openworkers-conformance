use openworkers_conformance::harness::run_wpt_test;

#[tokio::test(flavor = "current_thread")]
async fn api_basics() {
    run_wpt_test("encoding/api-basics.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn api_surrogates_utf8() {
    run_wpt_test("encoding/api-surrogates-utf8.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn encodeinto() {
    run_wpt_test("encoding/encodeInto.any.js").await;
}

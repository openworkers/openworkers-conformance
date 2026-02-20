use openworkers_conformance::harness::run_wpt_test;

#[tokio::test(flavor = "current_thread")]
async fn response_init_001() {
    run_wpt_test("fetch/api/response/response-init-001.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn response_init_contenttype() {
    run_wpt_test("fetch/api/response/response-init-contenttype.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn response_static_json() {
    run_wpt_test("fetch/api/response/response-static-json.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn response_static_error() {
    run_wpt_test("fetch/api/response/response-static-error.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn response_static_redirect() {
    run_wpt_test("fetch/api/response/response-static-redirect.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn response_error() {
    run_wpt_test("fetch/api/response/response-error.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn response_headers_guard() {
    run_wpt_test("fetch/api/response/response-headers-guard.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn json() {
    run_wpt_test("fetch/api/response/json.any.js").await;
}

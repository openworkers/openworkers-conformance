use openworkers_conformance::harness::run_wpt_test;

#[tokio::test(flavor = "current_thread")]
async fn request_structure() {
    run_wpt_test("fetch/api/request/request-structure.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn request_init_002() {
    run_wpt_test("fetch/api/request/request-init-002.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn request_init_contenttype() {
    run_wpt_test("fetch/api/request/request-init-contenttype.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn request_init_priority() {
    run_wpt_test("fetch/api/request/request-init-priority.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn request_init_stream() {
    run_wpt_test("fetch/api/request/request-init-stream.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn request_headers() {
    run_wpt_test("fetch/api/request/request-headers.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn request_bad_port() {
    run_wpt_test("fetch/api/request/request-bad-port.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn request_error() {
    run_wpt_test("fetch/api/request/request-error.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn request_consume_empty() {
    run_wpt_test("fetch/api/request/request-consume-empty.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn request_consume() {
    run_wpt_test("fetch/api/request/request-consume.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn request_disturbed() {
    run_wpt_test("fetch/api/request/request-disturbed.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn request_constructor_init_body_override() {
    run_wpt_test("fetch/api/request/request-constructor-init-body-override.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn forbidden_method() {
    run_wpt_test("fetch/api/request/forbidden-method.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn request_keepalive() {
    run_wpt_test("fetch/api/request/request-keepalive.any.js").await;
}

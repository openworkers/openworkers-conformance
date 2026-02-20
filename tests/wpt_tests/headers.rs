use openworkers_conformance::harness::run_wpt_test;

#[tokio::test(flavor = "current_thread")]
async fn headers_basic() {
    run_wpt_test("fetch/api/headers/headers-basic.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn headers_casing() {
    run_wpt_test("fetch/api/headers/headers-casing.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn headers_combine() {
    run_wpt_test("fetch/api/headers/headers-combine.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn headers_errors() {
    run_wpt_test("fetch/api/headers/headers-errors.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn headers_normalize() {
    run_wpt_test("fetch/api/headers/headers-normalize.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn headers_record() {
    run_wpt_test("fetch/api/headers/headers-record.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn headers_structure() {
    run_wpt_test("fetch/api/headers/headers-structure.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn header_values() {
    run_wpt_test("fetch/api/headers/header-values.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn header_values_normalize() {
    run_wpt_test("fetch/api/headers/header-values-normalize.any.js").await;
}

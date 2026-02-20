use openworkers_conformance::harness::run_wpt_test;

#[tokio::test(flavor = "current_thread")]
async fn body_formdata() {
    run_wpt_test("fetch/api/body/formdata.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn body_mime_type() {
    run_wpt_test("fetch/api/body/mime-type.any.js").await;
}

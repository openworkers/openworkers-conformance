use openworkers_conformance::harness::run_wpt_test;

#[tokio::test(flavor = "current_thread")]
async fn urlsearchparams_append() {
    run_wpt_test("url/urlsearchparams-append.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn urlsearchparams_constructor() {
    run_wpt_test("url/urlsearchparams-constructor.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn urlsearchparams_delete() {
    run_wpt_test("url/urlsearchparams-delete.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn urlsearchparams_foreach() {
    run_wpt_test("url/urlsearchparams-foreach.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn urlsearchparams_get() {
    run_wpt_test("url/urlsearchparams-get.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn urlsearchparams_getall() {
    run_wpt_test("url/urlsearchparams-getall.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn urlsearchparams_has() {
    run_wpt_test("url/urlsearchparams-has.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn urlsearchparams_set() {
    run_wpt_test("url/urlsearchparams-set.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn urlsearchparams_size() {
    run_wpt_test("url/urlsearchparams-size.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn urlsearchparams_sort() {
    run_wpt_test("url/urlsearchparams-sort.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn urlsearchparams_stringifier() {
    run_wpt_test("url/urlsearchparams-stringifier.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn url_statics_canparse() {
    run_wpt_test("url/url-statics-canparse.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn url_statics_parse() {
    run_wpt_test("url/url-statics-parse.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn url_tojson() {
    run_wpt_test("url/url-tojson.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn url_setters_stripping() {
    run_wpt_test("url/url-setters-stripping.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn url_historical() {
    run_wpt_test("url/historical.any.js").await;
}

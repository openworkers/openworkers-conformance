use openworkers_conformance::harness::run_wpt_test;

// --- Base64 (atob/btoa) ---

#[tokio::test(flavor = "current_thread")]
async fn base64() {
    run_wpt_test("html/webappapis/atob/base64.any.js").await;
}

// --- structuredClone ---

#[tokio::test(flavor = "current_thread")]
async fn structured_clone() {
    run_wpt_test("html/webappapis/structured-clone/structured-clone.any.js").await;
}

// --- queueMicrotask ---

#[tokio::test(flavor = "current_thread")]
async fn queue_microtask() {
    run_wpt_test("html/webappapis/microtask-queuing/queue-microtask.any.js").await;
}

// NOTE: queue-microtask-exceptions.any.js hangs in the current runtime.

// --- Timers ---

#[tokio::test(flavor = "current_thread")]
async fn timers_negative_settimeout() {
    run_wpt_test("html/webappapis/timers/negative-settimeout.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn timers_negative_setinterval() {
    run_wpt_test("html/webappapis/timers/negative-setinterval.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn timers_type_long_settimeout() {
    run_wpt_test("html/webappapis/timers/type-long-settimeout.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn timers_type_long_setinterval() {
    run_wpt_test("html/webappapis/timers/type-long-setinterval.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn timers_cleartimeout_clearinterval() {
    run_wpt_test("html/webappapis/timers/cleartimeout-clearinterval.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn timers_clearinterval_from_callback() {
    run_wpt_test("html/webappapis/timers/clearinterval-from-callback.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn timers_missing_timeout_setinterval() {
    run_wpt_test("html/webappapis/timers/missing-timeout-setinterval.any.js").await;
}

// NOTE: evil-spec-example.any.js hangs in the current runtime.

// --- Console ---

#[tokio::test(flavor = "current_thread")]
async fn console_is_a_namespace() {
    run_wpt_test("console/console-is-a-namespace.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn console_label_conversion() {
    run_wpt_test("console/console-label-conversion.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn console_log_symbol() {
    run_wpt_test("console/console-log-symbol.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn console_log_large_array() {
    run_wpt_test("console/console-log-large-array.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn console_namespace_object_class_string() {
    run_wpt_test("console/console-namespace-object-class-string.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn console_tests_historical() {
    run_wpt_test("console/console-tests-historical.any.js").await;
}

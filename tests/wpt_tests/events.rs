use openworkers_conformance::harness::run_wpt_test;

#[tokio::test(flavor = "current_thread")]
async fn event_constructors() {
    run_wpt_test("dom/events/Event-constructors.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn event_is_trusted() {
    run_wpt_test("dom/events/Event-isTrusted.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn event_target_constructible() {
    run_wpt_test("dom/events/EventTarget-constructible.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn event_target_add_remove_listener() {
    run_wpt_test("dom/events/EventTarget-add-remove-listener.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn event_target_add_event_listener() {
    run_wpt_test("dom/events/EventTarget-addEventListener.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn event_target_remove_event_listener() {
    run_wpt_test("dom/events/EventTarget-removeEventListener.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn add_event_listener_options_once() {
    run_wpt_test("dom/events/AddEventListenerOptions-once.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn add_event_listener_options_passive() {
    run_wpt_test("dom/events/AddEventListenerOptions-passive.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn add_event_listener_options_signal() {
    run_wpt_test("dom/events/AddEventListenerOptions-signal.any.js").await;
}

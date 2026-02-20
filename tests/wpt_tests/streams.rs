use openworkers_conformance::harness::run_wpt_test;

// --- Readable Streams ---
// NOTE: Many readable-streams tests (general, default-reader, cancel, tee,
// bad-strategies, bad-underlying-sources, reentrant-strategies, templated)
// hang indefinitely in the current runtime and are excluded.

#[tokio::test(flavor = "current_thread")]
async fn readable_streams_constructor() {
    run_wpt_test("streams/readable-streams/constructor.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn readable_streams_async_iterator() {
    run_wpt_test("streams/readable-streams/async-iterator.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn readable_streams_from() {
    run_wpt_test("streams/readable-streams/from.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn readable_streams_count_queuing_strategy() {
    run_wpt_test("streams/readable-streams/count-queuing-strategy-integration.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn readable_streams_floating_point_total_queue_size() {
    run_wpt_test("streams/readable-streams/floating-point-total-queue-size.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn readable_streams_patched_global() {
    run_wpt_test("streams/readable-streams/patched-global.any.js").await;
}

// --- Writable Streams ---

#[tokio::test(flavor = "current_thread")]
async fn writable_streams_general() {
    run_wpt_test("streams/writable-streams/general.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn writable_streams_constructor() {
    run_wpt_test("streams/writable-streams/constructor.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn writable_streams_close() {
    run_wpt_test("streams/writable-streams/close.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn writable_streams_write() {
    run_wpt_test("streams/writable-streams/write.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn writable_streams_error() {
    run_wpt_test("streams/writable-streams/error.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn writable_streams_aborting() {
    run_wpt_test("streams/writable-streams/aborting.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn writable_streams_start() {
    run_wpt_test("streams/writable-streams/start.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn writable_streams_properties() {
    run_wpt_test("streams/writable-streams/properties.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn writable_streams_bad_strategies() {
    run_wpt_test("streams/writable-streams/bad-strategies.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn writable_streams_bad_underlying_sinks() {
    run_wpt_test("streams/writable-streams/bad-underlying-sinks.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn writable_streams_byte_length_queuing_strategy() {
    run_wpt_test("streams/writable-streams/byte-length-queuing-strategy.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn writable_streams_count_queuing_strategy() {
    run_wpt_test("streams/writable-streams/count-queuing-strategy.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn writable_streams_floating_point_total_queue_size() {
    run_wpt_test("streams/writable-streams/floating-point-total-queue-size.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn writable_streams_reentrant_strategy() {
    run_wpt_test("streams/writable-streams/reentrant-strategy.any.js").await;
}

// --- Transform Streams ---

#[tokio::test(flavor = "current_thread")]
async fn transform_streams_general() {
    run_wpt_test("streams/transform-streams/general.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn transform_streams_errors() {
    run_wpt_test("streams/transform-streams/errors.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn transform_streams_flush() {
    run_wpt_test("streams/transform-streams/flush.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn transform_streams_backpressure() {
    run_wpt_test("streams/transform-streams/backpressure.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn transform_streams_cancel() {
    run_wpt_test("streams/transform-streams/cancel.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn transform_streams_properties() {
    run_wpt_test("streams/transform-streams/properties.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn transform_streams_strategies() {
    run_wpt_test("streams/transform-streams/strategies.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn transform_streams_terminate() {
    run_wpt_test("streams/transform-streams/terminate.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn transform_streams_lipfuzz() {
    run_wpt_test("streams/transform-streams/lipfuzz.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn transform_streams_patched_global() {
    run_wpt_test("streams/transform-streams/patched-global.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn transform_streams_reentrant_strategies() {
    run_wpt_test("streams/transform-streams/reentrant-strategies.any.js").await;
}

// --- Piping ---

#[tokio::test(flavor = "current_thread")]
async fn piping_general() {
    run_wpt_test("streams/piping/general.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn piping_pipe_through() {
    run_wpt_test("streams/piping/pipe-through.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn piping_transform_streams() {
    run_wpt_test("streams/piping/transform-streams.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn piping_close_propagation_forward() {
    run_wpt_test("streams/piping/close-propagation-forward.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn piping_close_propagation_backward() {
    run_wpt_test("streams/piping/close-propagation-backward.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn piping_error_propagation_forward() {
    run_wpt_test("streams/piping/error-propagation-forward.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn piping_error_propagation_backward() {
    run_wpt_test("streams/piping/error-propagation-backward.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn piping_flow_control() {
    run_wpt_test("streams/piping/flow-control.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn piping_multiple_propagation() {
    run_wpt_test("streams/piping/multiple-propagation.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn piping_then_interception() {
    run_wpt_test("streams/piping/then-interception.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn piping_throwing_options() {
    run_wpt_test("streams/piping/throwing-options.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn piping_abort() {
    run_wpt_test("streams/piping/abort.any.js").await;
}

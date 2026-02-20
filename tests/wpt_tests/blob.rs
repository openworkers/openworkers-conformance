use openworkers_conformance::harness::run_wpt_test;

#[tokio::test(flavor = "current_thread")]
async fn blob_constructor() {
    run_wpt_test("FileAPI/blob/Blob-constructor.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn blob_array_buffer() {
    run_wpt_test("FileAPI/blob/Blob-array-buffer.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn blob_bytes() {
    run_wpt_test("FileAPI/blob/Blob-bytes.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn blob_text() {
    run_wpt_test("FileAPI/blob/Blob-text.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn blob_slice() {
    run_wpt_test("FileAPI/blob/Blob-slice.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn blob_slice_overflow() {
    run_wpt_test("FileAPI/blob/Blob-slice-overflow.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn blob_stream() {
    run_wpt_test("FileAPI/blob/Blob-stream.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn blob_newobject() {
    run_wpt_test("FileAPI/blob/Blob-newobject.any.js").await;
}

#[tokio::test(flavor = "current_thread")]
async fn file_constructor() {
    run_wpt_test("FileAPI/file/File-constructor.any.js").await;
}

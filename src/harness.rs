use std::collections::HashMap;
use std::path::Path;

use openworkers_core::{Event, HttpMethod, HttpRequest, RequestBody, Script};
use serde::Deserialize;

use crate::runtime::{Worker, run_in_local};

const JS_HARNESS: &str = r#"
function assert(condition, message) {
    if (!condition) {
        throw new Error(message || 'Assertion failed');
    }
}

function assertEqual(actual, expected, message) {
    if (actual !== expected) {
        throw new Error(message || `Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
    }
}

function assertThrows(fn, expectedMessage) {
    let threw = false;

    try {
        fn();
    } catch (e) {
        threw = true;

        if (expectedMessage && !e.message.includes(expectedMessage)) {
            throw new Error(`Expected error containing "${expectedMessage}", got: "${e.message}"`);
        }
    }

    if (!threw) {
        throw new Error('Expected function to throw, but it did not');
    }
}

function assertType(value, expectedType, message) {
    const actualType = typeof value;

    if (actualType !== expectedType) {
        throw new Error(message || `Expected type "${expectedType}", got "${actualType}"`);
    }
}
"#;

const JS_WRAPPER: &str = r#"
addEventListener('fetch', async (event) => {
    try {
        await __runTests();
        event.respondWith(new Response(JSON.stringify({ pass: true }), {
            headers: { 'Content-Type': 'application/json' }
        }));
    } catch (e) {
        event.respondWith(new Response(JSON.stringify({
            pass: false,
            error: e.message,
            stack: e.stack || ''
        }), {
            headers: { 'Content-Type': 'application/json' }
        }));
    }
});
"#;

#[derive(Deserialize)]
struct TestResult {
    pass: bool,
    #[serde(default)]
    error: String,
    #[serde(default)]
    stack: String,
}

/// Execute inline JavaScript test code.
///
/// The code must define an `async function __runTests()` that uses the
/// harness assertion helpers (`assert`, `assertEqual`, `assertThrows`, `assertType`).
pub async fn run_js_test(code: &str) {
    run_in_local(|| async {
        let full_code = format!("{JS_HARNESS}\n{code}\n{JS_WRAPPER}");

        let script = Script::new(full_code);
        let mut worker = Worker::new(script, None).await.unwrap();

        let req = HttpRequest {
            method: HttpMethod::Get,
            url: "http://localhost/conformance-test".to_string(),
            headers: HashMap::new(),
            body: RequestBody::None,
        };

        let (task, rx) = Event::fetch(req);
        worker.exec(task).await.unwrap();

        let response = rx.await.unwrap();
        let body = response.body.collect().await.unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();

        let result: TestResult = serde_json::from_str(body_str)
            .unwrap_or_else(|e| panic!("Failed to parse test result JSON: {e}\nBody: {body_str}"));

        if !result.pass {
            panic!("Test failed: {}\n{}", result.error, result.stack);
        }
    })
    .await;
}

/// Execute a JavaScript test file relative to the crate root.
pub async fn run_js_test_file(path: &str) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = Path::new(manifest_dir).join(path);

    let code = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|e| panic!("Failed to read test file {}: {e}", full_path.display()));

    run_js_test(&code).await;
}

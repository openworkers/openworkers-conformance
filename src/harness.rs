use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::runtime::{run_in_local, Event, HttpMethod, HttpRequest, RequestBody, Script, Worker};

// ---------------------------------------------------------------------------
// Custom (simple) harness – kept for quick one-off tests
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// WPT harness – runs real Web Platform Tests via testharness.js
// ---------------------------------------------------------------------------

/// JS preamble registered *before* testharness.js.
///
/// It sets up a fetch event listener and a promise that resolves when the WPT
/// completion callback fires. The fetch handler awaits that promise and returns
/// a JSON summary of the results.
const WPT_PREAMBLE: &str = r#"
// --- OpenWorkers WPT preamble ---

// Shim clearTimeout/clearInterval to tolerate null/undefined (spec-compliant no-op).
// The runtime may reject non-integer arguments; testharness.js passes null on init.
const __origClearTimeout = globalThis.clearTimeout;
globalThis.clearTimeout = function(id) {
    if (id == null) return;
    return __origClearTimeout(id);
};
const __origClearInterval = globalThis.clearInterval;
if (typeof __origClearInterval === 'function') {
    globalThis.clearInterval = function(id) {
        if (id == null) return;
        return __origClearInterval(id);
    };
}

// Provide self.GLOBAL used by some WPT tests to detect the execution environment.
if (typeof self === 'undefined') { globalThis.self = globalThis; }
self.GLOBAL = {
    isWindow:  function() { return false; },
    isWorker:  function() { return true; },
    isShadowRealm: function() { return false; },
};

let __wptResolve;
const __wptPromise = new Promise(r => { __wptResolve = r; });

addEventListener('fetch', async (event) => {
    const results = await __wptPromise;
    event.respondWith(new Response(JSON.stringify(results), {
        headers: { 'Content-Type': 'application/json' }
    }));
});
"#;

/// JS snippet injected *after* testharness.js but *before* the actual test.
///
/// Registers a WPT completion callback that feeds into the promise above.
const WPT_COMPLETION_HOOK: &str = r#"
// --- OpenWorkers WPT completion hook ---
add_completion_callback(function(tests, harnessStatus) {
    const results = tests.map(function(t) {
        return { name: t.name, status: t.status, message: t.message || '' };
    });
    const failed = results.filter(function(t) { return t.status !== 0; });
    __wptResolve({
        pass:    failed.length === 0 && harnessStatus.status === 0,
        total:   results.length,
        passed:  results.filter(function(t) { return t.status === 0; }).length,
        failed:  failed.length,
        error:   failed.map(function(t) {
            var s = ['PASS','FAIL','TIMEOUT','NOTRUN','PRECONDITION_FAILED'][t.status] || 'UNKNOWN';
            return s + ': ' + t.name + (t.message ? ' - ' + t.message : '');
        }).join('\n'),
        stack:   ''
    });
});
"#;

// ---------------------------------------------------------------------------
// Shared data types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct TestResult {
    pass: bool,
    #[serde(default)]
    error: String,
    #[serde(default)]
    stack: String,
    #[serde(default)]
    total: usize,
    #[serde(default)]
    passed: usize,
    #[serde(default)]
    failed: usize,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn wpt_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/wpt")
}

fn read_file(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()))
}

/// Parse `// META: script=<path>` directives from a WPT test file and return
/// the concatenated content of all referenced scripts.
fn resolve_meta_scripts(test_code: &str, test_file_dir: &Path) -> String {
    let wpt = wpt_root();
    let mut scripts = String::new();

    for line in test_code.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("// META:") {
            // META directives are always at the top of the file.
            if !trimmed.is_empty() && !trimmed.starts_with("//") {
                break;
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("// META: script=") {
            let script_path = if rest.starts_with('/') {
                // Absolute to WPT root.
                wpt.join(&rest[1..])
            } else {
                // Relative to the test file directory.
                test_file_dir.join(rest)
            };
            scripts.push_str(&read_file(&script_path));
            scripts.push('\n');
        }
    }

    scripts
}

/// Execute assembled JS code through the Worker and return the parsed result.
async fn execute_worker(full_code: &str) -> TestResult {
    run_in_local(|| async {
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

        serde_json::from_str(body_str)
            .unwrap_or_else(|e| panic!("Failed to parse test result JSON: {e}\nBody: {body_str}"))
    })
    .await
}

// ---------------------------------------------------------------------------
// Public API – custom (simple) tests
// ---------------------------------------------------------------------------

/// Execute inline JavaScript test code.
///
/// The code must define an `async function __runTests()` that uses the
/// harness assertion helpers (`assert`, `assertEqual`, `assertThrows`, `assertType`).
pub async fn run_js_test(code: &str) {
    let full_code = format!("{JS_HARNESS}\n{code}\n{JS_WRAPPER}");
    let result = execute_worker(&full_code).await;

    if !result.pass {
        panic!("Test failed: {}\n{}", result.error, result.stack);
    }
}

/// Execute a JavaScript test file relative to the crate root.
pub async fn run_js_test_file(path: &str) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = Path::new(manifest_dir).join(path);
    let code = read_file(&full_path);
    run_js_test(&code).await;
}

// ---------------------------------------------------------------------------
// Public API – WPT tests
// ---------------------------------------------------------------------------

/// Run a WPT `.any.js` test by its path relative to the `tests/wpt/` root.
///
/// Example: `run_wpt_test("url/urlsearchparams-append.any.js")`
pub async fn run_wpt_test(wpt_path: &str) {
    let wpt = wpt_root();
    let test_file = wpt.join(wpt_path);
    let test_code = read_file(&test_file);

    let testharness_js = read_file(&wpt.join("resources/testharness.js"));

    let test_dir = test_file.parent().unwrap();
    let meta_scripts = resolve_meta_scripts(&test_code, test_dir);

    // Assembly order:
    //  1. WPT_PREAMBLE      – registers the fetch handler + result promise
    //  2. testharness.js     – the real WPT test framework
    //  3. WPT_COMPLETION_HOOK – hooks add_completion_callback → __wptResolve
    //  4. META scripts       – additional scripts required by the test
    //  5. Test code          – the actual test
    //  6. done()             – signals ShellTestEnvironment that all tests are loaded
    let full_code = format!(
        "{WPT_PREAMBLE}\n{testharness_js}\n{WPT_COMPLETION_HOOK}\n{meta_scripts}\n{test_code}\ndone();\n"
    );

    let result = execute_worker(&full_code).await;

    if !result.pass {
        panic!(
            "WPT test failed: {wpt_path}\n  {passed}/{total} passed, {failed} failed\n{error}",
            passed = result.passed,
            total = result.total,
            failed = result.failed,
            error = result.error,
        );
    }

    eprintln!("  WPT {wpt_path}: {}/{} passed", result.passed, result.total);
}

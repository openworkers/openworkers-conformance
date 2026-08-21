//! Runs one guest test file and turns what comes back into a [`FileReport`].

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use openworkers_core::{Event, HttpMethod, HttpRequest, RequestBody, Script};
use serde::Deserialize;

use crate::report::{FileReport, TestOutcome, truncate};
use crate::runtime::{Worker, limits, quiet_ops, run_in_local};

/// A guest that hangs must cost one file, not the run.
const FILE_TIMEOUT: Duration = Duration::from_secs(30);

const JS_HARNESS: &str = r#"
var __tests = [];

function test(name, fn) {
    __tests.push({ name: name, fn: fn });
}

function __show(value) {
    if (typeof value === 'string') {
        return JSON.stringify(value);
    }

    if (value === null || value === undefined || typeof value === 'number' || typeof value === 'boolean') {
        return String(value);
    }

    try {
        return JSON.stringify(value) || String(value);
    } catch (e) {
        return String(value);
    }
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message || 'assertion failed');
    }
}

function assertEqual(actual, expected, message) {
    if (actual !== expected) {
        throw new Error((message ? message + ': ' : '') + 'expected ' + __show(expected) + ', got ' + __show(actual));
    }
}

function assertType(value, expected, message) {
    var actual = typeof value;

    if (actual !== expected) {
        throw new Error((message ? message + ': ' : '') + 'expected type ' + expected + ', got ' + actual);
    }
}

function assertArrayEqual(actual, expected, message) {
    var prefix = message ? message + ': ' : '';

    assert(actual && typeof actual.length === 'number', prefix + 'not array-like: ' + __show(actual));
    assertEqual(actual.length, expected.length, prefix + 'length');

    for (var i = 0; i < expected.length; i++) {
        if (actual[i] !== expected[i]) {
            throw new Error(prefix + 'index ' + i + ': expected ' + __show(expected[i]) + ', got ' + __show(actual[i]));
        }
    }
}

// `expected` matches the error name first, its message second.
function assertThrows(fn, expected) {
    var threw = false;
    var error;

    try {
        fn();
    } catch (e) {
        threw = true;
        error = e;
    }

    if (!threw) {
        throw new Error('expected a throw' + (expected ? ' of ' + expected : '') + ', nothing was thrown');
    }

    __matchError(error, expected);
}

function assertRejects(promise, expected) {
    return Promise.resolve(promise).then(
        function (value) {
            throw new Error('expected a rejection' + (expected ? ' of ' + expected : '') + ', resolved with ' + __show(value));
        },
        function (error) {
            __matchError(error, expected);
        }
    );
}

function __matchError(error, expected) {
    if (!expected) {
        return;
    }

    var name = (error && error.name) || '';
    var message = (error && error.message) || String(error);

    if (name !== expected && message.indexOf(expected) < 0) {
        throw new Error('expected ' + expected + ', got ' + (name || '(no name)') + ': ' + message);
    }
}

// One test that never settles must not cost the rest of its file.
function __bounded(promise) {
    if (typeof setTimeout !== 'function') {
        return promise;
    }

    return new Promise(function (resolve, reject) {
        var timer = setTimeout(function () {
            reject(new Error('the test never settled, gave up after 2000ms'));
        }, 2000);

        Promise.resolve(promise).then(
            function (value) {
                clearTimeout(timer);
                resolve(value);
            },
            function (error) {
                clearTimeout(timer);
                reject(error);
            }
        );
    });
}

function __ascii(text) {
    var out = '';

    for (var i = 0; i < text.length; i++) {
        var code = text.charCodeAt(i);

        if (code >= 0x20 && code < 0x7f) {
            out += text.charAt(i);
        } else {
            out += '\\u' + ('000' + code.toString(16)).slice(-4);
        }
    }

    return out;
}
"#;

/// Some backends round-trip bodies as lossy UTF-8, hence the ASCII escaping.
const JS_WRAPPER: &str = r#"
addEventListener('fetch', async function (event) {
    var results = [];

    for (var i = 0; i < __tests.length; i++) {
        var entry = __tests[i];

        try {
            await __bounded(entry.fn());
            results.push({ name: entry.name, ok: true, error: '' });
        } catch (e) {
            var message = (e && e.message) || String(e);
            var name = (e && e.name) || '';

            results.push({
                name: entry.name,
                ok: false,
                error: __ascii(name && message.indexOf(name) !== 0 ? name + ': ' + message : message)
            });
        }
    }

    event.respondWith(new Response(JSON.stringify({ results: results }), {
        headers: { 'content-type': 'application/json' }
    }));
});
"#;

#[derive(Deserialize)]
struct GuestReport {
    results: Vec<GuestResult>,
}

#[derive(Deserialize)]
struct GuestResult {
    name: String,
    ok: bool,
    #[serde(default)]
    error: String,
}

/// Every backend scores against this denominator, so one that cannot even
/// parse a file still reports 0/N rather than 0/0.
fn declared_tests(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| line.strip_prefix("test("))
        .filter_map(|rest| {
            let quote = rest.chars().next()?;

            if quote != '\'' && quote != '"' {
                return None;
            }

            let name = &rest[1..];

            Some(name[..name.find(quote)?].to_string())
        })
        .collect()
}

/// Guest failures are data; only a file that never ran carries an error.
pub async fn run_file(area: &str, path: &Path, display: &str) -> FileReport {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(e) => return FileReport::unreadable(area, display, format!("read failed: {e}")),
    };

    let declared = declared_tests(&source);
    let code = format!("{JS_HARNESS}\n{source}\n{JS_WRAPPER}");

    let body = match tokio::time::timeout(FILE_TIMEOUT, run_in_local(|| evaluate(code))).await {
        Ok(Ok(body)) => body,
        Ok(Err(e)) => return FileReport::aborted(area, display, declared, e),
        Err(_) => {
            let reason = format!("no response after {}s", FILE_TIMEOUT.as_secs());

            return FileReport::aborted(area, display, declared, reason);
        }
    };

    let guest: GuestReport = match serde_json::from_str(&body) {
        Ok(guest) => guest,
        Err(e) => {
            let reason = format!("unparseable report ({e}): {}", truncate(&body, 200));

            return FileReport::aborted(area, display, declared, reason);
        }
    };

    FileReport {
        area: area.to_string(),
        path: display.to_string(),
        error: None,
        tests: merge(declared, guest.results),
    }
}

/// A declared name with no result failed before it could run.
fn merge(declared: Vec<String>, reported: Vec<GuestResult>) -> Vec<TestOutcome> {
    let mut by_name: HashMap<&str, &GuestResult> = HashMap::new();

    for result in &reported {
        by_name.insert(result.name.as_str(), result);
    }

    let mut outcomes: Vec<TestOutcome> = declared
        .iter()
        .map(|name| match by_name.get(name.as_str()) {
            Some(result) => TestOutcome {
                name: name.clone(),
                passed: result.ok,
                error: result.error.clone(),
            },
            None => TestOutcome {
                name: name.clone(),
                passed: false,
                error: "not reported by the guest".to_string(),
            },
        })
        .collect();

    // A name the source scan missed still counts, or the total lies.
    for result in &reported {
        if !declared.iter().any(|name| name == &result.name) {
            outcomes.push(TestOutcome {
                name: result.name.clone(),
                passed: result.ok,
                error: result.error.clone(),
            });
        }
    }

    outcomes
}

async fn evaluate(code: String) -> Result<String, String> {
    let mut worker = Worker::new_with_ops(Script::new(code), Some(limits()), quiet_ops())
        .await
        .map_err(|e| format!("worker init failed: {e}"))?;

    let (event, rx) = Event::fetch(HttpRequest {
        method: HttpMethod::Get,
        url: "http://conformance.invalid/".to_string(),
        headers: HashMap::new(),
        body: RequestBody::None,
    });

    worker
        .exec(event)
        .await
        .map_err(|e| format!("exec failed: {e}"))?;

    let response = rx
        .await
        .map_err(|_| "worker sent no response".to_string())?;

    let body = response
        .body
        .collect()
        .await
        .map_err(|e| format!("body stream failed: {e}"))?
        .ok_or_else(|| "worker sent an empty body".to_string())?;

    String::from_utf8(body.to_vec()).map_err(|e| format!("body is not utf-8: {e}"))
}

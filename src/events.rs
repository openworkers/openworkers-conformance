//! The event model: how a runtime hands an event to the guest's handler and
//! what comes back. A fetch listener and its FetchEvent follow the Service
//! Worker spec; `export default` and the scheduled event follow the Workers
//! module syntax.
//!
//! https://w3c.github.io/ServiceWorker/#fetchevent-interface
//! https://developers.cloudflare.com/workers/runtime-apis/handlers/
//!
//! The fetch event cases port the server side of web-platform-tests
//! service-workers/service-worker; each names the WPT file it comes from.
//! Left out: what needs a page, navigation, clients or the network
//! (outbound fetch, fallback to the network), and waitUntil after an event
//! ended, because a runtime here cancels an ended event's tasks.
//!
//! A case can end `lax`: the runtime accepts what the spec refuses, on
//! purpose. Each run says so, so the leniency stays visible.

use std::collections::HashMap;
use std::fmt::Write;
use std::time::Duration;

use openworkers_core::{
    Event, HttpMethod, HttpRequest, RequestBody, Script, TaskResult, TaskSource,
};
use serde::Serialize;

use crate::runtime::{Worker, limits, quiet_ops, run_in_local};

/// How long a case may take before it fails as hung.
const CASE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    Fail,
    /// The runtime accepts what the spec refuses.
    Lax,
}

#[derive(Debug, Serialize)]
pub struct Outcome {
    pub group: &'static str,
    pub case: &'static str,
    pub source: &'static str,
    pub status: Status,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct Run {
    pub backend: &'static str,
    pub outcomes: Vec<Outcome>,
}

/// One event to send, in order, to the same worker.
#[derive(Clone, Copy)]
enum Send {
    Get(&'static str),
    Post {
        path: &'static str,
        content_type: &'static str,
        body: &'static str,
    },
    Scheduled,
}

/// What one event gave back.
#[derive(Debug)]
enum Answer {
    Http {
        status: u16,
        content_type: Option<String>,
        body: String,
        /// Set when the body started and then failed.
        body_error: Option<String>,
    },
    Task(TaskResult),
    /// No answer; the reason the event ended without one.
    None(String),
}

struct Case {
    group: &'static str,
    name: &'static str,
    /// The WPT file the case comes from, or where its rule is written.
    source: &'static str,
    script: &'static str,
    events: &'static [Send],
    judge: fn(&[Answer]) -> (Status, String),
}

const SCHEDULED_TIME: u64 = 1_700_000_000_000;
const CRON: &str = "*/5 * * * *";

fn request(
    method: HttpMethod,
    path: &str,
    content_type: Option<&str>,
    body: Option<&str>,
) -> HttpRequest {
    // What a browser sends, so a case can look for it
    let mut headers = HashMap::new();
    headers.insert("accept".to_string(), "*/*".to_string());
    headers.insert("x-test".to_string(), "1".to_string());

    if let Some(content_type) = content_type {
        headers.insert("content-type".to_string(), content_type.to_string());
    }

    HttpRequest {
        method,
        url: format!("http://conformance.invalid{path}"),
        headers,
        body: match body {
            Some(body) => RequestBody::Bytes(body.as_bytes().to_vec().into()),
            None => RequestBody::None,
        },
    }
}

async fn fetch(worker: &mut Worker, request: HttpRequest) -> Answer {
    let (event, rx) = Event::fetch(request);
    let ended = worker.exec(event).await;

    let Ok(response) = rx.await else {
        return Answer::None(format!("no response; exec gave {ended:?}"));
    };

    let content_type = response
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.clone());

    let (body, body_error) = match response.body.collect().await {
        Ok(Some(bytes)) => (String::from_utf8_lossy(&bytes).into_owned(), None),
        Ok(None) => (String::new(), None),
        Err(err) => (String::new(), Some(err.to_string())),
    };

    Answer::Http {
        status: response.status,
        content_type,
        body,
        body_error,
    }
}

async fn serve(script: &'static str, events: &'static [Send]) -> Vec<Answer> {
    let mut worker =
        match Worker::new_with_ops(Script::new(script), Some(limits()), quiet_ops()).await {
            Ok(worker) => worker,
            Err(err) => return vec![Answer::None(format!("worker init failed: {err}"))],
        };

    let mut answers = Vec::new();

    for send in events {
        let answer = match *send {
            Send::Get(path) => fetch(&mut worker, request(HttpMethod::Get, path, None, None)).await,
            Send::Post {
                path,
                content_type,
                body,
            } => {
                fetch(
                    &mut worker,
                    request(HttpMethod::Post, path, Some(content_type), Some(body)),
                )
                .await
            }
            Send::Scheduled => {
                let source = TaskSource::Schedule {
                    time: SCHEDULED_TIME,
                    cron: Some(CRON.to_string()),
                };
                let (event, rx) = Event::task("scheduled".to_string(), None, Some(source), 1);
                let ended = worker.exec(event).await;

                match rx.await {
                    Ok(result) => Answer::Task(result),
                    Err(_) => Answer::None(format!("no result; exec gave {ended:?}")),
                }
            }
        };

        answers.push(answer);
    }

    answers
}

fn describe(answers: &[Answer], index: usize) -> String {
    match answers.get(index) {
        Some(Answer::Http {
            status,
            body,
            body_error,
            ..
        }) => match body_error {
            Some(error) => format!("{status}, body failed: {error}"),
            None => format!("{status} {body:?}"),
        },
        Some(Answer::Task(result)) => format!("task {result:?}"),
        Some(Answer::None(reason)) => reason.clone(),
        None => "no answer".to_string(),
    }
}

/// The answer at `index` is `status` with exactly `body`.
fn expect_at(answers: &[Answer], index: usize, status: u16, body: &str) -> (Status, String) {
    match answers.get(index) {
        Some(Answer::Http {
            status: got,
            body: text,
            body_error: None,
            ..
        }) if *got == status && text == body => (Status::Pass, String::new()),
        _ => (
            Status::Fail,
            format!(
                "expected {status} {body:?}, got {}",
                describe(answers, index)
            ),
        ),
    }
}

fn expect(answers: &[Answer], status: u16, body: &str) -> (Status, String) {
    expect_at(answers, 0, status, body)
}

/// The first answer is a network error: no response, or a status of 500 or
/// more, as a host without a network to fall back to gives one.
fn expect_error(answers: &[Answer]) -> (Status, String) {
    match answers.first() {
        Some(Answer::None(_)) => (Status::Pass, String::new()),
        Some(Answer::Http { status, .. }) if *status >= 500 => (Status::Pass, String::new()),
        _ => (
            Status::Fail,
            format!("expected a network error, got {}", describe(answers, 0)),
        ),
    }
}

/// Accepting the call is lax; refusing it, which leaves the request without
/// a response, is the spec.
fn lax_or_error(
    answers: &[Answer],
    accepted_body: &str,
    lenience: &'static str,
) -> (Status, String) {
    match answers.first() {
        Some(Answer::Http {
            status: 200, body, ..
        }) if body == accepted_body => (Status::Lax, lenience.to_string()),
        _ => expect_error(answers),
    }
}

/// The content type and body of a first answer, checked by `accept`.
fn expect_typed(answers: &[Answer], accept: fn(&str, &str) -> bool) -> (Status, String) {
    match answers.first() {
        Some(Answer::Http {
            status: 200,
            content_type: Some(content_type),
            body,
            body_error: None,
        }) if accept(content_type, body) => (Status::Pass, String::new()),
        Some(Answer::Http { content_type, .. }) => (
            Status::Fail,
            format!(
                "got {}, content-type {content_type:?}",
                describe(answers, 0)
            ),
        ),
        _ => (Status::Fail, describe(answers, 0)),
    }
}

macro_rules! wpt {
    ($file:literal) => {
        concat!("WPT service-workers/service-worker/", $file)
    };
}

const CASES: &[Case] = &[
    // respondWith and its argument
    Case {
        group: "respondWith",
        name: "a Response answers",
        source: wpt!("fetch-event-respond-with-argument.https.html"),
        script: "addEventListener('fetch', e => e.respondWith(new Response('body')));",
        events: &[Send::Get("/?response-object")],
        judge: |a| expect(a, 200, "body"),
    },
    Case {
        group: "respondWith",
        name: "a promise for a Response answers",
        source: wpt!("fetch-event-respond-with-argument.https.html"),
        script: "addEventListener('fetch', e => e.respondWith(Promise.resolve(new Response('body'))));",
        events: &[Send::Get("/?response-promise-object")],
        judge: |a| expect(a, 200, "body"),
    },
    Case {
        group: "respondWith",
        name: "a promise settled later answers when it settles",
        source: "Service Worker spec, FetchEvent.respondWith",
        script: "addEventListener('fetch', e => e.respondWith(new Promise(r => setTimeout(() => r(new Response('later')), 10))));",
        events: &[Send::Get("/")],
        judge: |a| expect(a, 200, "later"),
    },
    Case {
        group: "respondWith",
        name: "another object is a network error",
        source: wpt!("fetch-event-respond-with-argument.https.html"),
        script: "addEventListener('fetch', e => e.respondWith(new Object()));",
        events: &[Send::Get("/?other-value")],
        judge: expect_error,
    },
    Case {
        group: "respondWith",
        name: "a rejected promise is a network error",
        source: wpt!("fetch-event-network-error.https.html"),
        script: "addEventListener('fetch', e => e.respondWith(Promise.reject()));",
        events: &[Send::Get("/?reject")],
        judge: expect_error,
    },
    Case {
        group: "respondWith",
        name: "a Response with an unused body answers",
        source: wpt!("fetch-event-network-error.https.html"),
        script: "addEventListener('fetch', e => e.respondWith(new Response('body')));",
        events: &[Send::Get("/?unused-body")],
        judge: |a| expect(a, 200, "body"),
    },
    Case {
        group: "respondWith",
        name: "a Response whose body was used is a network error",
        source: wpt!("fetch-event-network-error.https.html"),
        script: "addEventListener('fetch', e => { const res = new Response('body'); res.text(); e.respondWith(res); });",
        events: &[Send::Get("/?used-body")],
        judge: expect_error,
    },
    // When respondWith may run
    Case {
        group: "respondWith timing",
        name: "in a microtask after the listener returned, it answers",
        source: wpt!("fetch-event-async-respond-with.https.html"),
        script: "addEventListener('fetch', e => { Promise.resolve().then(() => e.respondWith(new Response('microtask'))); });",
        events: &[Send::Get("/respondWith-in-microtask")],
        judge: |a| expect(a, 200, "microtask"),
    },
    Case {
        group: "respondWith timing",
        name: "after an await on a settled promise, it answers",
        source: wpt!("fetch-event-async-respond-with.https.html"),
        script: "addEventListener('fetch', async e => { await null; e.respondWith(new Response('after await')); });",
        events: &[Send::Get("/")],
        judge: |a| expect(a, 200, "after await"),
    },
    Case {
        group: "respondWith timing",
        name: "in a task after the dispatch, it throws InvalidStateError",
        source: wpt!("fetch-event-async-respond-with.https.html"),
        script: "addEventListener('fetch', e => { setTimeout(() => e.respondWith(new Response('task')), 0); });",
        events: &[Send::Get("/respondWith-in-task")],
        judge: |a| {
            lax_or_error(
                a,
                "task",
                "accepted in a task after dispatch; the spec throws InvalidStateError",
            )
        },
    },
    Case {
        group: "respondWith timing",
        name: "a second call throws InvalidStateError",
        source: wpt!("fetch-event.https.html, ?multiple-respond-with"),
        script: r#"
            addEventListener('fetch', e => {
                let log = '';
                for (let i = 0; i < 3; ++i) {
                    log += '(' + i + ')';
                    try {
                        e.respondWith(new Promise(r => setTimeout(() => r(new Response(log)), 0)));
                    } catch (error) {
                        log += '[' + error.name + ']';
                    }
                }
            });
        "#,
        events: &[Send::Get("/?multiple-respond-with")],
        judge: |a| expect(a, 200, "(0)(1)[InvalidStateError](2)[InvalidStateError]"),
    },
    Case {
        group: "respondWith timing",
        name: "a throw after respondWith keeps the response",
        source: wpt!("fetch-event-throws-after-respond-with.https.html"),
        script: r#"
            addEventListener('fetch', e => {
                e.respondWith(new Promise(r => setTimeout(r, 5)).then(() => new Response('intercepted')));
                throw 'error';
            });
        "#,
        events: &[Send::Get("/")],
        judge: |a| expect(a, 200, "intercepted"),
    },
    // Several listeners
    Case {
        group: "listeners",
        name: "respondWith stops the propagation to later listeners",
        source: wpt!("fetch-event-respond-with-stops-propagation.https.html"),
        script: r#"
            let result = null;
            addEventListener('fetch', e => {
                if (new URL(e.request.url).pathname === '/result') {
                    e.respondWith(new Response(String(result)));
                    return;
                }
                if (!result) result = 'PASS';
                e.respondWith(new Response());
            });
            addEventListener('fetch', () => { result = 'FAIL: fetch event propagated'; });
        "#,
        events: &[Send::Get("/"), Send::Get("/result")],
        judge: |a| expect_at(a, 1, 200, "PASS"),
    },
    Case {
        group: "listeners",
        name: "preventDefault, then respondWith in a later listener, answers",
        source: wpt!("fetch-event-network-error.https.html"),
        script: r#"
            addEventListener('fetch', e => e.preventDefault());
            addEventListener('fetch', e => e.respondWith(new Response('responding!')));
        "#,
        events: &[Send::Get("/?prevent-default-and-respond-with")],
        judge: |a| expect(a, 200, "responding!"),
    },
    Case {
        group: "listeners",
        name: "preventDefault without respondWith is a network error",
        source: wpt!("fetch-event-network-error.https.html"),
        script: "addEventListener('fetch', e => e.preventDefault());",
        events: &[Send::Get("/?prevent-default")],
        judge: expect_error,
    },
    Case {
        group: "listeners",
        name: "a listener that throws before answering is an error",
        source: "OpenWorkers: no network to fall back to",
        script: "addEventListener('fetch', () => { throw new Error('boom'); });",
        events: &[Send::Get("/")],
        judge: expect_error,
    },
    // Response bodies
    Case {
        group: "response body",
        name: "a string, as text/plain;charset=UTF-8",
        source: wpt!("fetch-event.https.html, ?string"),
        script: "addEventListener('fetch', e => e.respondWith(new Response('Test string')));",
        events: &[Send::Get("/?string")],
        judge: |a| {
            expect_typed(a, |content_type, body| {
                body == "Test string"
                    && content_type.eq_ignore_ascii_case("text/plain;charset=UTF-8")
            })
        },
    },
    Case {
        group: "response body",
        name: "a Blob",
        source: wpt!("fetch-event-respond-with-custom-response.https.html"),
        script: "addEventListener('fetch', e => e.respondWith(new Response(new Blob(['PASS']))));",
        events: &[Send::Get("/?type=blob")],
        judge: |a| expect(a, 200, "PASS"),
    },
    Case {
        group: "response body",
        name: "an ArrayBuffer",
        source: wpt!("fetch-event-respond-with-custom-response.https.html"),
        script: "addEventListener('fetch', e => e.respondWith(new Response(new TextEncoder().encode('PASS').buffer)));",
        events: &[Send::Get("/?type=buffer")],
        judge: |a| expect(a, 200, "PASS"),
    },
    Case {
        group: "response body",
        name: "an ArrayBufferView",
        source: wpt!("fetch-event-respond-with-custom-response.https.html"),
        script: "addEventListener('fetch', e => e.respondWith(new Response(new TextEncoder().encode('PASS'))));",
        events: &[Send::Get("/?type=buffer-view")],
        judge: |a| expect(a, 200, "PASS"),
    },
    Case {
        group: "response body",
        name: "FormData, as multipart/form-data",
        source: wpt!("fetch-event-respond-with-custom-response.https.html"),
        script: "addEventListener('fetch', e => { const fd = new FormData(); fd.set('result', 'PASS'); e.respondWith(new Response(fd)); });",
        events: &[Send::Get("/?type=form-data")],
        judge: |a| {
            expect_typed(a, |content_type, body| {
                content_type.starts_with("multipart/form-data; boundary=")
                    && body.contains("name=\"result\"")
                    && body.contains("PASS")
            })
        },
    },
    Case {
        group: "response body",
        name: "URLSearchParams",
        source: wpt!("fetch-event-respond-with-custom-response.https.html"),
        script: "addEventListener('fetch', e => e.respondWith(new Response(new URLSearchParams('result=PASS'), { headers: { 'Content-Type': 'text/plain' } })));",
        events: &[Send::Get("/?type=search-params")],
        judge: |a| expect(a, 200, "result=PASS"),
    },
    Case {
        group: "response body",
        name: "no body",
        source: wpt!("fetch-event.https.html, ?null"),
        script: "addEventListener('fetch', e => e.respondWith(new Response()));",
        events: &[Send::Get("/?null")],
        judge: |a| expect(a, 200, ""),
    },
    Case {
        group: "response body",
        name: "a ReadableStream",
        source: wpt!("fetch-event-respond-with-readable-stream.https.html"),
        script: "addEventListener('fetch', e => e.respondWith(new Response(new ReadableStream({ start(c) { c.enqueue(new TextEncoder().encode('PASS')); c.close(); } }))));",
        events: &[Send::Get("/?stream")],
        judge: |a| expect(a, 200, "PASS"),
    },
    Case {
        group: "response body",
        name: "a ReadableStream filled after a delay",
        source: wpt!("fetch-event-respond-with-readable-stream.https.html"),
        script: "addEventListener('fetch', e => e.respondWith(new Response(new ReadableStream({ start(c) { setTimeout(() => { c.enqueue(new TextEncoder().encode('PASS')); c.close(); }, 16); } }))));",
        events: &[Send::Get("/?stream&delay")],
        judge: |a| expect(a, 200, "PASS"),
    },
    Case {
        group: "response body",
        name: "a ReadableStream whose first chunk is empty",
        source: wpt!("fetch-event-respond-with-readable-stream-chunk.https.html"),
        script: r#"
            addEventListener('fetch', e => {
                const parts = ['', 'chunk #1', ' ', 'chunk #2', ' ', 'chunk #3', ' ', 'chunk #4'];
                let i = 0;
                e.respondWith(new Response(new ReadableStream({ pull(c) {
                    if (i < parts.length) c.enqueue(new TextEncoder().encode(parts[i++]));
                    else c.close();
                } })));
            });
        "#,
        events: &[Send::Get("/body-stream")],
        judge: |a| expect(a, 200, "chunk #1 chunk #2 chunk #3 chunk #4"),
    },
    Case {
        group: "response body",
        name: "a chunk that is not a Uint8Array errors the body",
        source: wpt!("fetch-event-respond-with-response-body-with-invalid-chunk.https.html"),
        script: "addEventListener('fetch', e => e.respondWith(new Response(new ReadableStream({ start(c) { c.enqueue('hello'); c.close(); } }))));",
        events: &[Send::Get("/body-stream-with-invalid-chunk")],
        judge: |a| match a.first() {
            Some(Answer::Http {
                body_error: Some(_),
                ..
            }) => (Status::Pass, String::new()),
            Some(Answer::Http {
                status: 200, body, ..
            }) if body == "hello" => (
                Status::Lax,
                "a string chunk is sent as text; the Fetch spec errors the body".to_string(),
            ),
            _ => (
                Status::Fail,
                format!("expected the body to fail, got {}", describe(a, 0)),
            ),
        },
    },
    // event.request
    Case {
        group: "event.request",
        name: "carries the method, the URL and the headers",
        source: "Service Worker spec, FetchEvent.request",
        script: "addEventListener('fetch', e => e.respondWith(new Response([e.request.method, new URL(e.request.url).pathname, e.request.headers.get('x-test')].join(' '))));",
        events: &[Send::Get("/path")],
        judge: |a| expect(a, 200, "GET /path 1"),
    },
    Case {
        group: "event.request",
        name: "lists its header names in lowercase",
        source: wpt!("fetch-event.https.html, ?headers"),
        script: "addEventListener('fetch', e => e.respondWith(new Response(Array.from(e.request.headers).map(([name]) => name).includes('accept') ? 'accept' : 'missing')));",
        events: &[Send::Get("/?headers")],
        judge: |a| expect(a, 200, "accept"),
    },
    Case {
        group: "event.request",
        name: "a form POST keeps its method, content type and body",
        source: wpt!("fetch-event.https.html, ?form-post"),
        script: "addEventListener('fetch', e => e.respondWith(e.request.text().then(t => new Response(e.request.method + ':' + e.request.headers.get('Content-Type') + ':' + t))));",
        events: &[Send::Post {
            path: "/?form-post",
            content_type: "application/x-www-form-urlencoded",
            body: "testName1=testValue1&testName2=testValue2",
        }],
        judge: |a| {
            expect(
                a,
                200,
                "POST:application/x-www-form-urlencoded:testName1=testValue1&testName2=testValue2",
            )
        },
    },
    Case {
        group: "event.request",
        name: "the body reads back as sent",
        source: wpt!("fetch-event.https.html, ?request-body"),
        script: "addEventListener('fetch', e => e.respondWith(e.request.text().then(t => new Response(t))));",
        events: &[Send::Post {
            path: "/?request-body",
            content_type: "text/plain",
            body: "i am the request body",
        }],
        judge: |a| expect(a, 200, "i am the request body"),
    },
    Case {
        group: "event.request",
        name: "integrity defaults to the empty string",
        source: wpt!("fetch-event.https.html, ?integrity"),
        script: "addEventListener('fetch', e => e.respondWith(new Response(JSON.stringify(e.request.integrity))));",
        events: &[Send::Get("/?integrity")],
        judge: |a| expect(a, 200, "\"\""),
    },
    Case {
        group: "event.request",
        name: "keepalive defaults to false",
        source: wpt!("fetch-event.https.html, ?keepalive"),
        script: "addEventListener('fetch', e => e.respondWith(new Response(String(e.request.keepalive))));",
        events: &[Send::Get("/?keepalive")],
        judge: |a| expect(a, 200, "false"),
    },
    // waitUntil
    Case {
        group: "waitUntil",
        name: "keeps the event alive after its response",
        source: "Service Worker spec, ExtendableEvent.waitUntil",
        script: r#"
            globalThis.done = false;
            addEventListener('fetch', e => {
                if (new URL(e.request.url).pathname === '/check') {
                    e.respondWith(new Response(String(globalThis.done)));
                    return;
                }
                e.waitUntil(new Promise(r => setTimeout(() => { globalThis.done = true; r(); }, 20)));
                e.respondWith(new Response('first'));
            });
        "#,
        events: &[Send::Get("/"), Send::Get("/check")],
        judge: |a| expect_at(a, 1, 200, "true"),
    },
    Case {
        group: "waitUntil",
        name: "in a task while respondWith is pending, it is allowed",
        source: wpt!(
            "extendable-event-async-waituntil.https.html, pending-respondwith-async-waituntil"
        ),
        script: r#"
            addEventListener('fetch', e => {
                let answer;
                e.respondWith(new Promise(r => { answer = r; }));
                setTimeout(() => {
                    let result = 'OK';
                    try { e.waitUntil(Promise.resolve()); } catch (error) { result = error.name; }
                    answer(new Response(result));
                }, 0);
            });
        "#,
        events: &[Send::Get("/pending-respondwith-async-waituntil")],
        judge: |a| expect(a, 200, "OK"),
    },
    // export default
    Case {
        group: "module syntax",
        name: "fetch(request, env, ctx) answers with what it returns",
        source: "Workers module syntax",
        script: "globalThis.default = { async fetch(request, env, ctx) { return new Response([request.method, typeof ctx.waitUntil].join(' ')); } };",
        events: &[Send::Get("/")],
        judge: |a| expect(a, 200, "GET function"),
    },
    Case {
        group: "module syntax",
        name: "fetch wins over a fetch listener",
        source: "Workers module syntax",
        script: "globalThis.default = { fetch() { return new Response('module'); } }; addEventListener('fetch', e => e.respondWith(new Response('listener')));",
        events: &[Send::Get("/")],
        judge: |a| expect(a, 200, "module"),
    },
    Case {
        group: "module syntax",
        name: "fetch that returns what is not a Response is an error",
        source: "Workers module syntax",
        script: "globalThis.default = { fetch() { return 'text'; } };",
        events: &[Send::Get("/")],
        judge: expect_error,
    },
    Case {
        group: "module syntax",
        name: "ctx.waitUntil keeps the event alive after its response",
        source: "Workers module syntax",
        script: r#"
            globalThis.done = false;
            globalThis.default = {
                fetch(request, env, ctx) {
                    if (new URL(request.url).pathname === '/check') {
                        return new Response(String(globalThis.done));
                    }
                    ctx.waitUntil(new Promise(r => setTimeout(() => { globalThis.done = true; r(); }, 20)));
                    return new Response('first');
                },
            };
        "#,
        events: &[Send::Get("/"), Send::Get("/check")],
        judge: |a| expect_at(a, 1, 200, "true"),
    },
    // scheduled
    Case {
        group: "scheduled event",
        name: "a listener gets scheduledTime, cron and noRetry",
        source: "Workers scheduled handler",
        script: r#"
            globalThis.seen = 'not called';
            addEventListener('scheduled', e => {
                globalThis.seen = [e.scheduledTime, e.cron, typeof e.noRetry].join(' ');
            });
            addEventListener('fetch', e => e.respondWith(new Response(globalThis.seen)));
        "#,
        events: &[Send::Scheduled, Send::Get("/")],
        judge: |a| expect_at(a, 1, 200, "1700000000000 */5 * * * * function"),
    },
    Case {
        group: "scheduled event",
        name: "scheduled(controller, env, ctx) gets the same fields",
        source: "Workers scheduled handler",
        script: r#"
            globalThis.seen = 'not called';
            globalThis.default = {
                scheduled(controller, env, ctx) {
                    globalThis.seen = [controller.scheduledTime, controller.cron, typeof ctx.waitUntil].join(' ');
                },
                fetch() { return new Response(globalThis.seen); },
            };
        "#,
        events: &[Send::Scheduled, Send::Get("/")],
        judge: |a| expect_at(a, 1, 200, "1700000000000 */5 * * * * function"),
    },
];

/// Turns on the backend's strict mode, for the workers created after.
/// Answers false where the backend has no such switch.
pub fn strict() -> bool {
    #[cfg(feature = "v8")]
    {
        openworkers_runtime_v8::set_strict(true);
        true
    }

    #[cfg(not(feature = "v8"))]
    false
}

/// Runs each case on a fresh worker, on a thread of its own, so a case that
/// panics or hangs fails alone.
pub fn run(filter: Option<&str>) -> Run {
    let outcomes = CASES
        .iter()
        .filter(|case| filter.is_none_or(|f| case.name.contains(f) || case.group.contains(f)))
        .map(|case| {
            let (script, events) = (case.script, case.events);
            let answers = std::thread::spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                runtime.block_on(async move {
                    match tokio::time::timeout(CASE_TIMEOUT, run_in_local(|| serve(script, events)))
                        .await
                    {
                        Ok(answers) => answers,
                        Err(_) => vec![Answer::None(format!("hung for {CASE_TIMEOUT:?}"))],
                    }
                })
            })
            .join();

            let (status, detail) = match answers {
                Ok(answers) => (case.judge)(&answers),
                Err(_) => (Status::Fail, "the case panicked".to_string()),
            };

            Outcome {
                group: case.group,
                case: case.name,
                source: case.source,
                status,
                detail,
            }
        })
        .collect();

    Run {
        backend: crate::runtime::NAME,
        outcomes,
    }
}

impl Run {
    pub fn render(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "events: {}", self.backend);

        let mut group = "";

        for outcome in &self.outcomes {
            if outcome.group != group {
                group = outcome.group;
                let _ = writeln!(out, "{group}");
            }

            let status = match outcome.status {
                Status::Pass => "ok",
                Status::Fail => "FAIL",
                Status::Lax => "lax",
            };
            let _ = writeln!(out, "  {:<66} {status:<4} {}", outcome.case, outcome.detail);
        }

        let count = |status| self.outcomes.iter().filter(|o| o.status == status).count();
        let _ = writeln!(
            out,
            "\nTOTAL  {}/{} pass, {} lax, {} fail",
            count(Status::Pass),
            self.outcomes.len(),
            count(Status::Lax),
            count(Status::Fail)
        );

        for outcome in self.outcomes.iter().filter(|o| o.status == Status::Lax) {
            let _ = writeln!(out, "notice: lax: {}: {}", outcome.case, outcome.detail);
        }

        out
    }
}

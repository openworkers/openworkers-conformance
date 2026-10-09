//! The event model: how a runtime hands an event to the guest's handler and
//! what comes back. A fetch listener and its FetchEvent follow the Service
//! Worker spec (respondWith, waitUntil); `export default` and the scheduled
//! event follow the Workers module syntax.
//!
//! https://w3c.github.io/ServiceWorker/#fetchevent-interface
//! https://developers.cloudflare.com/workers/runtime-apis/handlers/
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
    Scheduled,
}

/// What one event gave back.
#[derive(Debug)]
enum Answer {
    Http {
        status: u16,
        body: String,
    },
    Task(TaskResult),
    /// No answer; the reason the event ended without one.
    None(String),
}

struct Case {
    group: &'static str,
    name: &'static str,
    script: &'static str,
    events: &'static [Send],
    judge: fn(&[Answer]) -> (Status, String),
}

const SCHEDULED_TIME: u64 = 1_700_000_000_000;
const CRON: &str = "*/5 * * * *";

async fn serve(script: &'static str, events: &'static [Send]) -> Vec<Answer> {
    let mut worker =
        match Worker::new_with_ops(Script::new(script), Some(limits()), quiet_ops()).await {
            Ok(worker) => worker,
            Err(err) => return vec![Answer::None(format!("worker init failed: {err}"))],
        };

    let mut answers = Vec::new();

    for send in events {
        let answer = match send {
            Send::Get(path) => {
                let mut headers = HashMap::new();
                headers.insert("x-test".to_string(), "1".to_string());

                let (event, rx) = Event::fetch(HttpRequest {
                    method: HttpMethod::Get,
                    url: format!("http://conformance.invalid{path}"),
                    headers,
                    body: RequestBody::None,
                });
                let ended = worker.exec(event).await;

                match rx.await {
                    Ok(response) => {
                        let status = response.status;
                        let body = match response.body.collect().await {
                            Ok(Some(bytes)) => String::from_utf8_lossy(&bytes).into_owned(),
                            Ok(None) => String::new(),
                            Err(err) => format!("<body failed: {err}>"),
                        };

                        Answer::Http { status, body }
                    }
                    Err(_) => Answer::None(format!("no response; exec gave {ended:?}")),
                }
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

/// The status and body of the answer at `index`, if it is an HTTP response.
fn http(answers: &[Answer], index: usize) -> Option<(u16, &str)> {
    match answers.get(index)? {
        Answer::Http { status, body } => Some((*status, body.as_str())),
        _ => None,
    }
}

fn describe(answers: &[Answer], index: usize) -> String {
    match answers.get(index) {
        Some(Answer::Http { status, body }) => format!("{status} {body:?}"),
        Some(Answer::Task(result)) => format!("task {result:?}"),
        Some(Answer::None(reason)) => reason.clone(),
        None => "no answer".to_string(),
    }
}

/// The first answer is `status` with exactly `body`.
fn expect(answers: &[Answer], status: u16, body: &str) -> (Status, String) {
    match http(answers, 0) {
        Some((got, text)) if got == status && text == body => (Status::Pass, String::new()),
        _ => (
            Status::Fail,
            format!("expected {status} {body:?}, got {}", describe(answers, 0)),
        ),
    }
}

/// The first answer is an error: no response, or a status of 500 or more.
fn expect_error(answers: &[Answer]) -> (Status, String) {
    match answers.first() {
        Some(Answer::None(_)) => (Status::Pass, String::new()),
        Some(Answer::Http { status, .. }) if *status >= 500 => (Status::Pass, String::new()),
        _ => (
            Status::Fail,
            format!("expected an error, got {}", describe(answers, 0)),
        ),
    }
}

const CASES: &[Case] = &[
    Case {
        group: "fetch event",
        name: "respondWith with a Response answers it",
        script: "addEventListener('fetch', e => e.respondWith(new Response('ok', { status: 201 })));",
        events: &[Send::Get("/")],
        judge: |a| expect(a, 201, "ok"),
    },
    Case {
        group: "fetch event",
        name: "respondWith with a promise answers when it settles",
        script: "addEventListener('fetch', e => e.respondWith(new Promise(r => setTimeout(() => r(new Response('later')), 10))));",
        events: &[Send::Get("/")],
        judge: |a| expect(a, 200, "later"),
    },
    Case {
        group: "fetch event",
        name: "event.request carries the method, the URL and the headers",
        script: "addEventListener('fetch', e => e.respondWith(new Response([e.request.method, new URL(e.request.url).pathname, e.request.headers.get('x-test')].join(' '))));",
        events: &[Send::Get("/path")],
        judge: |a| expect(a, 200, "GET /path 1"),
    },
    Case {
        group: "fetch event",
        name: "respondWith with what is not a Response is an error",
        script: "addEventListener('fetch', e => e.respondWith('text'));",
        events: &[Send::Get("/")],
        judge: expect_error,
    },
    Case {
        group: "fetch event",
        name: "respondWith with a rejected promise is an error",
        script: "addEventListener('fetch', e => e.respondWith(Promise.reject(new Error('refused'))));",
        events: &[Send::Get("/")],
        judge: expect_error,
    },
    Case {
        group: "fetch event",
        name: "a listener that throws is an error",
        script: "addEventListener('fetch', () => { throw new Error('boom'); });",
        events: &[Send::Get("/")],
        judge: expect_error,
    },
    Case {
        group: "fetch event",
        name: "a second respondWith throws InvalidStateError",
        script: r#"
            addEventListener('fetch', e => {
                let name = 'no error';
                e.respondWith(new Promise(r => setTimeout(() => r(new Response(name)), 10)));
                try {
                    e.respondWith(new Response('second'));
                } catch (error) {
                    name = error.name;
                }
            });
        "#,
        events: &[Send::Get("/")],
        judge: |a| expect(a, 200, "InvalidStateError"),
    },
    Case {
        group: "fetch event",
        name: "respondWith after the listener returned throws InvalidStateError",
        script: r#"
            addEventListener('fetch', async e => {
                await null;
                e.respondWith(new Response('late'));
            });
        "#,
        events: &[Send::Get("/")],
        judge: |a| match http(a, 0) {
            Some((200, "late")) => (
                Status::Lax,
                "accepted after the listener returned; the spec throws InvalidStateError"
                    .to_string(),
            ),
            _ => expect_error(a),
        },
    },
    Case {
        group: "waitUntil",
        name: "waitUntil keeps the event alive after its response",
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
        judge: |a| match http(a, 1) {
            Some((200, "true")) => (Status::Pass, String::new()),
            _ => (
                Status::Fail,
                format!("second request saw {}", describe(a, 1)),
            ),
        },
    },
    Case {
        group: "module syntax",
        name: "fetch(request, env, ctx) answers with what it returns",
        script: "globalThis.default = { async fetch(request, env, ctx) { return new Response([request.method, typeof ctx.waitUntil].join(' ')); } };",
        events: &[Send::Get("/")],
        judge: |a| expect(a, 200, "GET function"),
    },
    Case {
        group: "module syntax",
        name: "fetch wins over a fetch listener",
        script: "globalThis.default = { fetch() { return new Response('module'); } }; addEventListener('fetch', e => e.respondWith(new Response('listener')));",
        events: &[Send::Get("/")],
        judge: |a| expect(a, 200, "module"),
    },
    Case {
        group: "module syntax",
        name: "fetch that returns what is not a Response is an error",
        script: "globalThis.default = { fetch() { return 'text'; } };",
        events: &[Send::Get("/")],
        judge: expect_error,
    },
    Case {
        group: "module syntax",
        name: "ctx.waitUntil keeps the event alive after its response",
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
        judge: |a| match http(a, 1) {
            Some((200, "true")) => (Status::Pass, String::new()),
            _ => (
                Status::Fail,
                format!("second request saw {}", describe(a, 1)),
            ),
        },
    },
    Case {
        group: "scheduled event",
        name: "a scheduled listener gets scheduledTime, cron and noRetry",
        script: r#"
            globalThis.seen = 'not called';
            addEventListener('scheduled', e => {
                globalThis.seen = [e.scheduledTime, e.cron, typeof e.noRetry].join(' ');
            });
            addEventListener('fetch', e => e.respondWith(new Response(globalThis.seen)));
        "#,
        events: &[Send::Scheduled, Send::Get("/")],
        judge: |a| match http(a, 1) {
            Some((200, "1700000000000 */5 * * * * function")) => (Status::Pass, String::new()),
            _ => (Status::Fail, format!("saw {}", describe(a, 1))),
        },
    },
    Case {
        group: "scheduled event",
        name: "scheduled(controller, env, ctx) gets the same fields",
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
        judge: |a| match http(a, 1) {
            Some((200, "1700000000000 */5 * * * * function")) => (Status::Pass, String::new()),
            _ => (Status::Fail, format!("saw {}", describe(a, 1))),
        },
    },
];

/// Turns on the backend's strict respondWith, for the workers created after.
/// Answers false where the backend has no such switch.
pub fn strict_respond_with() -> bool {
    #[cfg(feature = "v8")]
    {
        openworkers_runtime_v8::set_strict_respond_with(true);
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

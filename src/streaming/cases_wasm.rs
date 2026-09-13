//! The streaming corpus for the wasm backend.
//!
//! The JavaScript fixtures have nothing to say to a runtime that executes
//! components, so the probes here drive the `fetch-worker-v3` example that
//! ships with `openworkers-runtime-wasm`: a guest that already streams 32 MiB
//! through 8 MiB of linear memory, in both directions. The question this
//! battery asks of it is narrower and different, and worth asking anyway:
//! what shape does that body have by the time it reaches `HttpResponse`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use bytes::Bytes;
use openworkers_core::Event;
use openworkers_core::HttpMethod;
use openworkers_core::HttpRequest;
use openworkers_core::RequestBody;
use openworkers_core::RuntimeLimits;
use openworkers_core::Script;
use openworkers_core::WorkerCode;
use sha2::Digest;
use sha2::Sha256;
use tokio::sync::mpsc;

use crate::runtime::Worker;
use crate::runtime::quiet_ops;
use crate::streaming::drive;
use crate::streaming::drive::Exec;
use crate::streaming::drive::ExecResult;
use crate::streaming::outcome::Measures;
use crate::streaming::outcome::Outcome;

use super::Case;
use super::Verdict;

const CHUNK: usize = 64 * 1024;
const MIB: usize = 1024 * 1024;
const BULK_MB: usize = 32;

/// The component the sibling runtime builds for its own p3 tests. Rebuilding
/// it here would measure a different guest than the one those tests measure.
fn component() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../openworkers-runtime-wasm/examples/fetch-worker-v3")
        .join("target/wasm32-wasip2/release/fetch_worker_v3.wasm")
}

fn script() -> Result<Script, String> {
    let path = component();
    let bytes = std::fs::read(&path).map_err(|e| {
        format!(
            "{} is missing ({e}); build it with cargo build --target wasm32-wasip2 --release",
            path.display()
        )
    })?;

    Ok(Script {
        code: WorkerCode::WebAssembly(bytes),
        env: None,
        bindings: vec![],
    })
}

fn limits(heap_max_mb: usize) -> RuntimeLimits {
    RuntimeLimits {
        heap_max_mb,
        max_cpu_time_ms: 0,
        max_wall_clock_time_ms: 120_000,
        ..Default::default()
    }
}

fn request(method: HttpMethod, path: &str, body: RequestBody) -> HttpRequest {
    HttpRequest {
        method,
        url: format!("http://conformance.invalid{path}"),
        headers: HashMap::new(),
        body,
    }
}

fn digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();

    hasher.update(bytes);

    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

struct Exchange {
    drained: drive::Drained,
    exec: ExecResult,
}

impl Exchange {
    fn measures(&self) -> Measures {
        let mut measures = Measures {
            ttfc_ms: self.drained.ttfc().map(|at| at.as_millis() as u64),
            total_ms: Some(self.drained.total.as_millis() as u64),
            chunks: Some(self.drained.chunks.len()),
            bytes: Some(self.drained.bytes().len()),
            notes: Vec::new(),
        };

        measures.note(format!(
            "body arrived as {}",
            if self.drained.streamed {
                "ResponseBody::Stream"
            } else {
                "ResponseBody::Bytes or None"
            }
        ));
        measures.note(drive::exec_note(&self.exec));

        measures
    }
}

async fn exchange(limits: RuntimeLimits, request: HttpRequest) -> Result<Exchange, Verdict> {
    let script = match script() {
        Ok(script) => script,
        Err(detail) => return Err(Verdict::new(Outcome::skipped(detail))),
    };

    let mut worker = match Worker::new_with_ops(script, Some(limits), quiet_ops()).await {
        Ok(worker) => worker,
        Err(reason) => {
            return Err(Verdict::new(Outcome::rejects(format!(
                "worker init failed: {reason:?}"
            ))));
        }
    };

    let (event, rx) = Event::fetch(request);
    let started = Instant::now();
    let future = worker.exec(event);

    tokio::pin!(future);

    let mut exec: Exec<'_> = future;
    let mut done: ExecResult = None;

    let response = match drive::head(&mut exec, &mut done, rx).await {
        Ok(response) => response,
        Err(detail) => {
            // The interesting half of "no response" is why exec gave up.
            let detail = match &done {
                Some(Err(reason)) => format!("{detail}: {reason:?}"),
                _ => detail,
            };

            let mut verdict = Verdict::new(Outcome::rejects(detail));

            verdict.measures.note(drive::exec_note(&done));

            return Err(verdict);
        }
    };

    let drained = drive::drain(&mut exec, &mut done, started, response.body, None).await;

    drive::settle(&mut exec, &mut done, Duration::from_secs(2)).await;

    Ok(Exchange {
        drained,
        exec: done,
    })
}

/// The shape question: a guest that streams internally still hands the host
/// one buffer, so this records which variant arrives.
async fn response_shape() -> Result<Verdict, Verdict> {
    let exchange = exchange(
        limits(0),
        request(HttpMethod::Get, "/hello", RequestBody::None),
    )
    .await?;
    let measures = exchange.measures();

    let outcome = if exchange.drained.text() == "hello from v3" {
        if exchange.drained.streamed {
            Outcome::Streams
        } else {
            Outcome::Buffers
        }
    } else {
        Outcome::wrong(format!("unexpected body {:?}", exchange.drained.text()))
    };

    Ok(Verdict { outcome, measures })
}

/// 32 MiB out of a guest capped at 8 MiB: the p3 pattern, measured at the
/// core boundary instead of at the guest's.
async fn integrity_response() -> Result<Verdict, Verdict> {
    let exchange = exchange(
        limits(8),
        request(HttpMethod::Get, "/generate?mb=32", RequestBody::None),
    )
    .await?;

    let mut measures = exchange.measures();
    let bytes = exchange.drained.bytes();
    let expected = digest(&vec![b'x'; BULK_MB * MIB]);
    let actual = digest(&bytes);

    let outcome = if actual == expected {
        if exchange.drained.streamed {
            Outcome::Streams
        } else {
            Outcome::Buffers
        }
    } else {
        Outcome::wrong(format!(
            "sha256 mismatch on {} bytes of {}",
            bytes.len(),
            BULK_MB * MIB
        ))
    };

    measures.note(format!("sha256 expected {}", &expected[..16]));
    measures.note(format!("sha256 received {}", &actual[..16]));

    Ok(Verdict { outcome, measures })
}

/// 32 MiB into the same guest, fed as a `RequestBody::Stream`. The guest
/// answers with the byte count it counted, so a truncated upload shows up as
/// a number rather than as a crash.
async fn integrity_request() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(16);

    tokio::spawn(async move {
        for _ in 0..(BULK_MB * MIB / CHUNK) {
            if tx.send(Ok(Bytes::from(vec![b'a'; CHUNK]))).await.is_err() {
                break;
            }
        }
    });

    let exchange = exchange(
        limits(8),
        request(HttpMethod::Post, "/consume", RequestBody::Stream(rx)),
    )
    .await?;

    let measures = exchange.measures();
    let expected = (BULK_MB * MIB).to_string();
    let text = exchange.drained.text();

    let outcome = if text == expected {
        Outcome::Ok
    } else {
        Outcome::wrong(format!("guest counted {text:?}, expected {expected:?}"))
    };

    Ok(Verdict { outcome, measures })
}

/// An `Err` chunk mid-upload. The wanted answer is a refusal that carries the
/// upstream message, not a short body the guest counts as a complete upload.
async fn request_error_midstream() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(16);

    tokio::spawn(async move {
        let _ = tx.send(Ok(Bytes::from(vec![b'a'; CHUNK]))).await;
        let _ = tx.send(Err("connection reset".to_string())).await;
        let _ = tx.send(Ok(Bytes::from(vec![b'a'; CHUNK]))).await;
    });

    let request = request(HttpMethod::Post, "/consume", RequestBody::Stream(rx));

    let exchange = match exchange(limits(0), request).await {
        Ok(exchange) => exchange,
        // A refusal repeating the message the probe injected has surfaced the
        // truncation; any other refusal is still a refusal.
        Err(mut verdict) => {
            let named = verdict
                .outcome
                .detail()
                .is_some_and(|detail| detail.contains("connection reset"));

            if named {
                verdict.outcome = Outcome::Ok;
                verdict
                    .measures
                    .note("the refusal carried the upstream message");
            }

            return Ok(verdict);
        }
    };

    let mut measures = exchange.measures();
    let text = exchange.drained.text();

    let outcome = match text.as_str() {
        "65536" => Outcome::wrong("the error was dropped and the short body passed as complete"),
        "131072" => Outcome::wrong("the error was dropped and the body was silently stitched"),
        _ => Outcome::Ok,
    };

    measures.note(format!(
        "guest counted {text:?} of {} bytes sent",
        2 * CHUNK
    ));

    Ok(Verdict { outcome, measures })
}

const BUDGET: Duration = Duration::from_secs(60);
const BULK_BUDGET: Duration = Duration::from_secs(180);

pub fn all() -> Vec<Case> {
    vec![
        Case {
            name: "response_buffered_shape",
            dimension: "response streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(response_shape().await) }),
        },
        Case {
            name: "integrity_response_32mib_capped",
            dimension: "integrity",
            budget: BULK_BUDGET,
            run: || Box::pin(async { Verdict::either(integrity_response().await) }),
        },
        Case {
            name: "integrity_request_32mib_capped",
            dimension: "integrity",
            budget: BULK_BUDGET,
            run: || Box::pin(async { Verdict::either(integrity_request().await) }),
        },
        Case {
            name: "request_error_midstream",
            dimension: "mid-stream error",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(request_error_midstream().await) }),
        },
    ]
}

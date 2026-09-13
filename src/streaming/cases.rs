//! The streaming corpus, run unchanged on every JavaScript backend.
//!
//! Most scenarios are ports of the two twin suites that already existed for v8
//! alone: `openworkers-runtime-v8/tests/request_body_stream_test.rs` for the
//! request side, and the backpressure cadences of
//! `openworkers-runner/tests/request_body_streaming_test.rs`. The names are
//! kept recognisable on purpose. What is new here is the response side: the
//! time-to-first-chunk measurement, the 32 MiB integrity round trip, and the
//! mid-stream error and cancellation probes.

use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

use bytes::Bytes;
use openworkers_core::Event;
use openworkers_core::HttpMethod;
use openworkers_core::HttpRequest;
use openworkers_core::RequestBody;
use openworkers_core::ResponseBody;
use openworkers_core::RuntimeLimits;
use openworkers_core::Script;
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

/// 64 KiB, the chunk size the p3 tests use for the same payload.
const CHUNK: usize = 64 * 1024;

/// 512 chunks of 64 KiB is the 32 MiB those tests move through 8 MiB of guest.
const BULK_CHUNKS: usize = 512;

// ---------------------------------------------------------------------------
// fixtures
// ---------------------------------------------------------------------------

macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/streaming/",
            $name
        ))
    };
}

/// Fixtures carry their sizes as `{NAME}` placeholders rather than reading a
/// query string or a header, so a backend with a weak `URL` still gets the
/// guest the corpus meant to run.
fn render(source: &str, vars: &[(&str, String)]) -> String {
    let mut out = source.to_string();

    for (name, value) in vars {
        out = out.replace(&format!("{{{name}}}"), value);
    }

    out
}

// ---------------------------------------------------------------------------
// limits
// ---------------------------------------------------------------------------

/// Enough rope for a paced guest, but still bounded.
fn quick() -> RuntimeLimits {
    RuntimeLimits {
        max_cpu_time_ms: 10_000,
        max_wall_clock_time_ms: 20_000,
        ..Default::default()
    }
}

/// 32 MiB of memcpy is not an attack, so the CPU budget is off, like the p3
/// tests do for the same reason.
fn bulk() -> RuntimeLimits {
    RuntimeLimits {
        max_cpu_time_ms: 0,
        max_wall_clock_time_ms: 120_000,
        ..Default::default()
    }
}

fn capped(heap_max_mb: usize) -> RuntimeLimits {
    RuntimeLimits {
        heap_max_mb,
        ..bulk()
    }
}

// ---------------------------------------------------------------------------
// plumbing
// ---------------------------------------------------------------------------

async fn start(source: String, limits: RuntimeLimits) -> Result<Worker, String> {
    Worker::new_with_ops(Script::new(source), Some(limits), quiet_ops())
        .await
        .map_err(|reason| format!("worker init failed: {reason:?}"))
}

fn post(body: RequestBody) -> HttpRequest {
    HttpRequest {
        method: HttpMethod::Post,
        url: "http://conformance.invalid/".to_string(),
        headers: HashMap::new(),
        body,
    }
}

fn get() -> HttpRequest {
    HttpRequest {
        method: HttpMethod::Get,
        url: "http://conformance.invalid/".to_string(),
        headers: HashMap::new(),
        body: RequestBody::None,
    }
}

/// Feeds a request channel on a timer and reports how long the last `send`
/// took to be accepted: with a working backpressure path that span tracks the
/// guest, and without one it collapses to nothing.
struct Feed {
    sent: usize,
    span: Duration,
    refused: bool,
}

fn feed(
    tx: mpsc::Sender<Result<Bytes, String>>,
    items: Vec<Result<Bytes, String>>,
    delay: Duration,
) -> tokio::task::JoinHandle<Feed> {
    tokio::spawn(async move {
        let started = Instant::now();
        let mut sent = 0;
        let mut refused = false;

        for item in items {
            if tx.send(item).await.is_err() {
                refused = true;

                break;
            }

            sent += 1;

            if !delay.is_zero() {
                tokio::time::sleep(delay).await;
            }
        }

        Feed {
            sent,
            span: started.elapsed(),
            refused,
        }
    })
}

fn chunks_of(payload: &[u8], size: usize) -> Vec<Result<Bytes, String>> {
    payload
        .chunks(size)
        .map(|chunk| Ok(Bytes::copy_from_slice(chunk)))
        .collect()
}

/// A deterministic payload: any dropped or reordered chunk changes the digest.
fn payload(chunks: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(chunks * CHUNK);

    for _ in 0..chunks {
        for i in 0..CHUNK {
            out.push((i & 0xff) as u8);
        }
    }

    out
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

/// What one request/response exchange produced, once the body has been read.
struct Exchange {
    status: u16,
    drained: drive::Drained,
    exec: ExecResult,
}

impl Exchange {
    fn text(&self) -> String {
        self.drained.text()
    }

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

        // Many chunks that all land at once is the signature of a runtime
        // that collected the body and then replayed it down a channel.
        if self.drained.chunks.len() > 1 && self.drained.ttfc() == Some(self.drained.total) {
            measures.note("every chunk arrived at the same instant".to_string());
        }

        if let Some(error) = &self.drained.error {
            measures.note(format!("body channel carried Err({error})"));
        }

        measures
    }
}

/// The common path: start a worker, run one fetch, read the whole body.
async fn exchange(
    source: String,
    limits: RuntimeLimits,
    request: HttpRequest,
    per_chunk: Option<Duration>,
) -> Result<Exchange, Verdict> {
    let mut worker = match start(source, limits).await {
        Ok(worker) => worker,
        Err(detail) => return Err(Verdict::new(Outcome::rejects(detail))),
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

    let status = response.status;
    let drained = drive::drain(&mut exec, &mut done, started, response.body, per_chunk).await;

    drive::settle(&mut exec, &mut done, Duration::from_secs(2)).await;

    Ok(Exchange {
        status,
        drained,
        exec: done,
    })
}

/// A body that matches is `Ok`; anything else is reported with what came back.
fn against(exchange: &Exchange, expected: &str) -> Outcome {
    let text = exchange.text();

    if text == expected {
        return Outcome::Ok;
    }

    Outcome::wrong(format!(
        "expected {expected:?}, got {:?} (status {})",
        crate::report::truncate(&text, 120),
        exchange.status
    ))
}

// ---------------------------------------------------------------------------
// request streaming: host to guest
// ---------------------------------------------------------------------------

/// Ported from `test_request_body_stream_text`.
async fn request_text() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(16);

    feed(
        tx,
        vec![
            Ok(Bytes::from("Hello")),
            Ok(Bytes::from(" ")),
            Ok(Bytes::from("World")),
        ],
        Duration::ZERO,
    );

    let exchange = exchange(
        fixture!("request-text.js").to_string(),
        quick(),
        post(RequestBody::Stream(rx)),
        None,
    )
    .await?;

    let outcome = against(&exchange, "Got: Hello World");

    Ok(Verdict {
        outcome,
        measures: exchange.measures(),
    })
}

/// Ported from `test_request_body_stream_reader`. Whether the guest sees three
/// chunks or one is the whole question: one means the runtime collected the
/// body before handing it over.
async fn request_reader() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(16);

    feed(
        tx,
        vec![
            Ok(Bytes::from("one")),
            Ok(Bytes::from("two")),
            Ok(Bytes::from("three")),
        ],
        Duration::from_millis(5),
    );

    let exchange = exchange(
        fixture!("request-reader.js").to_string(),
        quick(),
        post(RequestBody::Stream(rx)),
        None,
    )
    .await?;

    let text = exchange.text();
    let mut measures = exchange.measures();

    let outcome = if text == "chunks=3:one|two|three" {
        Outcome::Streams
    } else if text == "chunks=1:onetwothree" {
        Outcome::Buffers
    } else {
        against(&exchange, "chunks=3:one|two|three")
    };

    measures.note(format!("guest reported {text:?}"));

    Ok(Verdict { outcome, measures })
}

/// Ported from `test_request_body_stream_json`: the object is split mid-value.
async fn request_json() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(16);

    feed(
        tx,
        vec![
            Ok(Bytes::from(r#"{"name":"#)),
            Ok(Bytes::from(r#""Alice","#)),
            Ok(Bytes::from(r#""age":30}"#)),
        ],
        Duration::ZERO,
    );

    let exchange = exchange(
        fixture!("request-json.js").to_string(),
        quick(),
        post(RequestBody::Stream(rx)),
        None,
    )
    .await?;

    let outcome = against(&exchange, "name=Alice,age=30");

    Ok(Verdict {
        outcome,
        measures: exchange.measures(),
    })
}

/// Ported from `test_request_body_stream_arraybuffer` and
/// `test_request_body_stream_binary_nulls`.
async fn request_binary() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(16);

    feed(
        tx,
        vec![
            Ok(Bytes::from(vec![1u8, 0, 2, 0])),
            Ok(Bytes::from(vec![0u8, 3, 0])),
        ],
        Duration::ZERO,
    );

    let exchange = exchange(
        fixture!("request-arraybuffer.js").to_string(),
        quick(),
        post(RequestBody::Stream(rx)),
        None,
    )
    .await?;

    let outcome = against(&exchange, "len=7,sum=6,nulls=4");

    Ok(Verdict {
        outcome,
        measures: exchange.measures(),
    })
}

/// Ported from `test_request_body_stream_utf8_boundary`.
async fn request_utf8_boundary() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(16);

    feed(
        tx,
        vec![
            Ok(Bytes::from(vec![b'H', b'i', b' ', 0xF0, 0x9F])),
            Ok(Bytes::from(vec![0x8E, 0x89, b'!'])),
        ],
        Duration::ZERO,
    );

    let exchange = exchange(
        fixture!("request-utf8.js").to_string(),
        quick(),
        post(RequestBody::Stream(rx)),
        None,
    )
    .await?;

    let outcome = against(&exchange, "chars=5:Hi \u{1F389}!");

    Ok(Verdict {
        outcome,
        measures: exchange.measures(),
    })
}

/// Ported from `test_request_body_stream_large`: 100 chunks of 1 KiB.
async fn request_large() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(16);
    let items = (0..100)
        .map(|_| Ok(Bytes::from(vec![b'X'; 1024])))
        .collect();

    feed(tx, items, Duration::ZERO);

    let exchange = exchange(
        fixture!("request-count.js").to_string(),
        quick(),
        post(RequestBody::Stream(rx)),
        None,
    )
    .await?;

    let text = exchange.text();
    let mut measures = exchange.measures();

    let outcome = if text == "chunks=100,bytes=102400" {
        Outcome::Streams
    } else if text == "chunks=1,bytes=102400" {
        Outcome::Buffers
    } else {
        against(&exchange, "chunks=100,bytes=102400")
    };

    measures.note(format!("guest reported {text:?}"));

    Ok(Verdict { outcome, measures })
}

/// Ported from `test_request_body_stream_empty`: the channel closes at once.
async fn request_empty() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel::<Result<Bytes, String>>(16);

    drop(tx);

    let exchange = exchange(
        fixture!("request-text.js").to_string(),
        quick(),
        post(RequestBody::Stream(rx)),
        None,
    )
    .await?;

    let outcome = against(&exchange, "Got: ");

    Ok(Verdict {
        outcome,
        measures: exchange.measures(),
    })
}

/// Ported from `test_request_body_stream_double_consume`.
async fn request_double_consume() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(16);

    feed(tx, vec![Ok(Bytes::from("data"))], Duration::ZERO);

    let exchange = exchange(
        fixture!("request-double-consume.js").to_string(),
        quick(),
        post(RequestBody::Stream(rx)),
        None,
    )
    .await?;

    let outcome = against(&exchange, "bodyUsed:data");

    Ok(Verdict {
        outcome,
        measures: exchange.measures(),
    })
}

/// Ported from `test_request_body_stream_never_consumed`: the guest answers
/// without reading, and the producer must not be left holding the channel.
async fn request_never_consumed() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(2);
    let items = (0..5).map(|_| Ok(Bytes::from("data"))).collect();
    let producer = feed(tx, items, Duration::from_millis(10));

    let exchange = exchange(
        fixture!("request-ignored.js").to_string(),
        quick(),
        post(RequestBody::Stream(rx)),
        None,
    )
    .await?;

    let mut measures = exchange.measures();
    let outcome = against(&exchange, "ignored");

    match tokio::time::timeout(Duration::from_secs(2), producer).await {
        Ok(Ok(feed)) => measures.note(format!(
            "producer sent {} of 5 chunks in {}ms, channel {}",
            feed.sent,
            feed.span.as_millis(),
            if feed.refused {
                "closed"
            } else {
                "stayed open"
            }
        )),
        Ok(Err(e)) => measures.note(format!("producer task failed: {e}")),
        Err(_) => measures.note("producer was still blocked 2s after the response".to_string()),
    }

    Ok(Verdict { outcome, measures })
}

/// Ported from `test_request_body_stream_partial_read`: the guest cancels
/// after one chunk.
async fn request_partial_read() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(2);
    let items = (1..=10)
        .map(|i| Ok(Bytes::from(format!("chunk{i}"))))
        .collect();
    let producer = feed(tx, items, Duration::from_millis(10));

    let exchange = exchange(
        fixture!("request-partial-read.js").to_string(),
        quick(),
        post(RequestBody::Stream(rx)),
        None,
    )
    .await?;

    let mut measures = exchange.measures();
    let outcome = against(&exchange, "first=chunk1");

    match tokio::time::timeout(Duration::from_secs(2), producer).await {
        Ok(Ok(feed)) => measures.note(format!(
            "producer sent {} of 10 chunks, channel {}",
            feed.sent,
            if feed.refused {
                "closed"
            } else {
                "stayed open"
            }
        )),
        Ok(Err(e)) => measures.note(format!("producer task failed: {e}")),
        Err(_) => measures.note("producer was still blocked 2s after the response".to_string()),
    }

    Ok(Verdict { outcome, measures })
}

/// Ported from `test_request_body_stream_error`: an `Err` chunk mid-body. A
/// read that rejects is the wanted answer; a body that just ends is the
/// interesting failure, because upstream loss then looks like a short upload.
async fn request_error_midstream() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(16);

    feed(
        tx,
        vec![
            Ok(Bytes::from("partial")),
            Err("connection reset".to_string()),
        ],
        Duration::ZERO,
    );

    let exchange = exchange(
        fixture!("request-error.js").to_string(),
        quick(),
        post(RequestBody::Stream(rx)),
        None,
    )
    .await?;

    let text = exchange.text();
    let mut measures = exchange.measures();

    let outcome = if text.starts_with("threw:partial:") {
        Outcome::Ok
    } else if text == "done:partial" {
        Outcome::wrong("the error became a clean end of body: truncation is silent")
    } else {
        against(&exchange, "threw:partial:<error>")
    };

    measures.note(format!("guest reported {text:?}"));

    Ok(Verdict { outcome, measures })
}

// ---------------------------------------------------------------------------
// response streaming: guest to host
// ---------------------------------------------------------------------------

/// Eight chunks, 40ms apart, each stamped by the guest clock. A backend
/// that streams shows a first chunk long before the last one is produced; a
/// backend that collects shows both at the same instant.
async fn response_ttfc() -> Result<Verdict, Verdict> {
    let source = render(
        fixture!("response-paced.js"),
        &[("CHUNKS", "8".to_string()), ("DELAY", "40".to_string())],
    );

    let exchange = exchange(source, quick(), get(), None).await?;
    let mut measures = exchange.measures();

    let ttfc = exchange.drained.ttfc().unwrap_or(exchange.drained.total);
    let total = exchange.drained.total;
    let text = exchange.text();
    let stamps = stamp_spread(&text);

    if let Some(spread) = stamps {
        measures.note(format!("guest produced over {spread}ms of its own clock"));
    }

    let outcome = if exchange.drained.chunks.is_empty() {
        Outcome::wrong("no body at all: the paced guest produced nothing")
    } else if text.lines().count() != 8 {
        Outcome::wrong(format!(
            "expected 8 stamped lines, got {}",
            text.lines().count()
        ))
    } else if ttfc.as_millis() * 2 < total.as_millis() {
        Outcome::Streams
    } else {
        Outcome::Buffers
    };

    Ok(Verdict { outcome, measures })
}

/// A pull-driven producer that stops on a round number of chunks, read
/// as fast as the host can. Sixteen is the high-water mark every runtime here
/// picked for its response pipeline, so a stream that ends exactly on a
/// multiple of it is the case a notification bug hides in.
async fn response_pipeline_multiple() -> Result<Verdict, Verdict> {
    let source = render(
        fixture!("response-pull-stamped.js"),
        &[("CHUNKS", "32".to_string())],
    );

    let exchange = exchange(source, quick(), get(), None).await?;
    let mut measures = exchange.measures();
    let lines = exchange.text().lines().count();

    let outcome = if lines == 32 {
        Outcome::Ok
    } else {
        Outcome::wrong(format!("expected 32 stamped lines, got {lines}"))
    };

    measures.note("the guest closed on exactly 2 x the 16-slot high-water mark".to_string());

    Ok(Verdict { outcome, measures })
}

/// Spread between the first and last guest timestamp, in ms.
fn stamp_spread(text: &str) -> Option<u64> {
    let stamps: Vec<u64> = text
        .lines()
        .filter_map(|line| line.split(':').nth(1))
        .filter_map(|stamp| stamp.trim().parse().ok())
        .collect();

    match (stamps.first(), stamps.last()) {
        (Some(first), Some(last)) => Some(last.saturating_sub(*first)),
        _ => None,
    }
}

/// A plain string body has nothing to stream, so which variant the
/// runtime picks says whether `ResponseBody::Stream` means anything here.
async fn response_buffered_shape() -> Result<Verdict, Verdict> {
    let exchange = exchange(fixture!("hello.js").to_string(), quick(), get(), None).await?;

    let outcome = against(&exchange, "hello");

    Ok(Verdict {
        outcome,
        measures: exchange.measures(),
    })
}

/// The guest errors its own stream after two chunks. The host should get
/// an `Err` on the channel; a clean close means a truncated response is
/// indistinguishable from a complete one.
async fn response_error_midstream() -> Result<Verdict, Verdict> {
    let source = render(
        fixture!("response-error-midstream.js"),
        &[("GOOD", "2".to_string())],
    );

    let exchange = exchange(source, quick(), get(), None).await?;
    let mut measures = exchange.measures();
    let text = exchange.text();

    let outcome = match (&exchange.drained.error, text.as_str()) {
        (Some(_), _) => Outcome::Ok,
        (None, "chunk0chunk1") => {
            Outcome::wrong("the guest error became a clean end of body: truncation is silent")
        }
        (None, "") => Outcome::rejects("no body: the error swallowed the whole response"),
        (None, other) => Outcome::wrong(format!("unexpected body {other:?}")),
    };

    measures.note(format!("host received {text:?}"));

    Ok(Verdict { outcome, measures })
}

/// A client that hangs up must cost the worker its stream, not its life.
const RECOVERY: Duration = Duration::from_secs(1);

/// Past this the probe stops waiting and calls the worker stuck.
const STUCK: Duration = Duration::from_secs(5);

/// The contract `disconnect_recovery` checks, in two halves.
///
/// 1. The host drops the response receiver three chunks into a long stream.
///    `exec` has to come back inside `RECOVERY`, rather than keep producing
///    for a client that has left until the guest runs out of chunks.
/// 2. The same worker has to answer a second fetch.
///
/// The second fetch is driven, not awaited. A backend that streams with real
/// backpressure only ends `exec` once the host has drained the body, so a
/// probe that awaited `exec` and read the body afterwards would deadlock
/// against exactly the backends it is meant to reward.
///
/// Whether that second stream ever ends is not asked here; that is
/// `response_pipeline_multiple`.
async fn disconnect_recovery() -> Result<Verdict, Verdict> {
    let source = render(
        fixture!("response-long.js"),
        &[("CHUNKS", "200".to_string()), ("DELAY", "10".to_string())],
    );

    let mut worker = match start(source, quick()).await {
        Ok(worker) => worker,
        Err(detail) => return Err(Verdict::new(Outcome::rejects(detail))),
    };

    let mut measures = Measures::default();

    // Scoped, because the borrow the first `exec` holds on the worker has to
    // end before the follow-up fetch can start.
    let recovery = match hang_up(&mut worker, &mut measures).await {
        Ok(recovery) => recovery,
        Err(outcome) => return Ok(Verdict { outcome, measures }),
    };

    let followed = tokio::time::timeout(STUCK, follow_up(&mut worker, &mut measures)).await;

    let outcome = match followed {
        Ok(outcome) => outcome,
        Err(_) => {
            measures.note(format!(
                "the follow-up fetch was still going {}s in",
                STUCK.as_secs()
            ));

            Outcome::Hangs
        }
    };

    if outcome == Outcome::Ok && recovery > RECOVERY {
        return Ok(Verdict {
            outcome: Outcome::wrong(format!(
                "the worker recovered, but only {}ms after the client left",
                recovery.as_millis()
            )),
            measures,
        });
    }

    Ok(Verdict { outcome, measures })
}

/// Hangs up three chunks in and returns how long `exec` then took to come back.
async fn hang_up(worker: &mut Worker, measures: &mut Measures) -> Result<Duration, Outcome> {
    let (event, rx) = Event::fetch(get());
    let started = Instant::now();
    let future = worker.exec(event);

    tokio::pin!(future);

    let mut exec: Exec<'_> = future;
    let mut done: ExecResult = None;

    let response = match drive::head(&mut exec, &mut done, rx).await {
        Ok(response) => response,
        Err(detail) => {
            measures.note(drive::exec_note(&done));

            return Err(Outcome::rejects(detail));
        }
    };

    let ResponseBody::Stream(mut body) = response.body else {
        measures.note("the body was not a stream, so there was nothing to cancel".to_string());

        return Err(Outcome::skipped("the backend never streams a response"));
    };

    let mut taken = 0;

    while taken < 3 {
        match drive::while_running(&mut exec, &mut done, body.recv()).await {
            Some(Ok(_)) => taken += 1,
            _ => break,
        }
    }

    measures.chunks = Some(taken);
    measures.note(format!(
        "{taken} chunks read over {}ms before hanging up",
        started.elapsed().as_millis()
    ));

    drop(body);

    let hung_up = Instant::now();

    drive::settle(&mut exec, &mut done, STUCK).await;

    let recovery = hung_up.elapsed();

    measures.total_ms = Some(recovery.as_millis() as u64);
    measures.note(drive::exec_note(&done));
    measures.note(format!(
        "exec came back {}ms after the client left",
        recovery.as_millis()
    ));

    if done.is_none() {
        return Err(Outcome::Hangs);
    }

    Ok(recovery)
}

/// Fetches the same worker again and reads far enough to prove bytes flow.
async fn follow_up(worker: &mut Worker, measures: &mut Measures) -> Outcome {
    let (event, rx) = Event::fetch(get());
    let started = Instant::now();
    let future = worker.exec(event);

    tokio::pin!(future);

    let mut exec: Exec<'_> = future;
    let mut done: ExecResult = None;

    let response = match drive::head(&mut exec, &mut done, rx).await {
        Ok(response) => response,
        Err(detail) => {
            measures.note(format!("the follow-up fetch: {detail}"));
            measures.note(drive::exec_note(&done));

            return Outcome::rejects("the worker was unusable after the client left");
        }
    };

    measures.note(format!(
        "the follow-up head arrived {}ms in with status {}",
        started.elapsed().as_millis(),
        response.status
    ));

    let mut chunks = 0;

    match response.body {
        ResponseBody::None => {}
        ResponseBody::Bytes(bytes) => chunks = usize::from(!bytes.is_empty()),
        ResponseBody::Stream(mut body) => {
            while chunks < 3 {
                match drive::while_running(&mut exec, &mut done, body.recv()).await {
                    Some(Ok(_)) => chunks += 1,
                    _ => break,
                }
            }
        }
    }

    measures.note(format!("the follow-up delivered {chunks} chunks"));

    if response.status != 200 {
        return Outcome::wrong(format!("the follow-up answered {}", response.status));
    }

    if chunks == 0 {
        return Outcome::wrong("the follow-up answered with no body at all");
    }

    Outcome::Ok
}

/// Ported from `test_request_body_stream_echo`.
async fn response_echo() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(16);
    let items = ["Hello", " ", "World", "!"]
        .iter()
        .map(|part| Ok(Bytes::from(*part)))
        .collect();

    feed(tx, items, Duration::from_millis(5));

    let exchange = exchange(
        fixture!("request-echo.js").to_string(),
        quick(),
        post(RequestBody::Stream(rx)),
        None,
    )
    .await?;

    let mut measures = exchange.measures();
    let mut outcome = against(&exchange, "Hello World!");

    if outcome == Outcome::Ok && exchange.drained.chunks.len() >= 4 {
        outcome = Outcome::Streams;
    }

    measures.note(format!(
        "{} chunks in, {} chunks out",
        4,
        exchange.drained.chunks.len()
    ));

    Ok(Verdict { outcome, measures })
}

// ---------------------------------------------------------------------------
// bidirectional
// ---------------------------------------------------------------------------

/// Ported from `test_request_body_stream_bidirectional` and the runner's
/// `test_backpressure_bidirectional`: numbers go in one at a time through a
/// two-slot channel while their doubles come back out to a reader that pauses
/// between chunks. Both directions have to be live at once for this to pass.
async fn bidirectional_transform() -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(2);
    let items = (1..=5).map(|n| Ok(Bytes::from(n.to_string()))).collect();
    let producer = feed(tx, items, Duration::from_millis(5));

    let exchange = exchange(
        fixture!("request-transform.js").to_string(),
        quick(),
        post(RequestBody::Stream(rx)),
        Some(Duration::from_millis(2)),
    )
    .await?;

    let mut measures = exchange.measures();

    let values: Vec<String> = exchange
        .drained
        .chunks
        .iter()
        .map(|chunk| String::from_utf8_lossy(&chunk.bytes).into_owned())
        .collect();

    match tokio::time::timeout(Duration::from_secs(2), producer).await {
        Ok(Ok(feed)) => measures.note(format!(
            "producer sent {} of 5 numbers over {}ms",
            feed.sent,
            feed.span.as_millis()
        )),
        Ok(Err(e)) => measures.note(format!("producer task failed: {e}")),
        Err(_) => measures.note("producer was still blocked 2s after the response".to_string()),
    }

    measures.note(format!("host received {values:?}"));

    let outcome = if values == ["2", "4", "6", "8", "10"] {
        Outcome::Streams
    } else if exchange.text() == "246810" {
        Outcome::Buffers
    } else {
        against(&exchange, "246810")
    };

    Ok(Verdict { outcome, measures })
}

// ---------------------------------------------------------------------------
// backpressure
// ---------------------------------------------------------------------------

/// One shape for the three input-backpressure cadences the two twin suites
/// use: a fast host producer against a guest that sleeps per chunk.
async fn input_backpressure(
    capacity: usize,
    count: u64,
    guest_delay: u64,
) -> Result<Verdict, Verdict> {
    let (tx, rx) = mpsc::channel(capacity);
    let items = (1..=count)
        .map(|n| Ok(Bytes::from(n.to_string())))
        .collect();
    let producer = feed(tx, items, Duration::ZERO);

    let source = render(
        fixture!("request-slow-reader.js"),
        &[("DELAY", guest_delay.to_string())],
    );

    let exchange = exchange(source, quick(), post(RequestBody::Stream(rx)), None).await?;

    let expected = format!("chunks={},sum={}", count, count * (count + 1) / 2);
    let mut measures = exchange.measures();
    let mut outcome = against(&exchange, &expected);

    // With backpressure the producer cannot finish before the guest has read
    // count - capacity chunks, each of which costs guest_delay.
    let floor = guest_delay.saturating_mul(count.saturating_sub(capacity as u64)) / 2;

    match tokio::time::timeout(Duration::from_secs(5), producer).await {
        Ok(Ok(feed)) => {
            let span = feed.span.as_millis() as u64;

            measures.note(format!(
                "producer pushed {} chunks in {span}ms through a {capacity}-slot channel",
                feed.sent
            ));

            if outcome == Outcome::Ok {
                outcome = if span >= floor {
                    Outcome::Streams
                } else {
                    Outcome::Buffers
                };

                measures.note(format!("backpressure floor for this cadence is {floor}ms"));
            }
        }
        Ok(Err(e)) => measures.note(format!("producer task failed: {e}")),
        Err(_) => measures.note("producer was still blocked 5s after the response".to_string()),
    }

    Ok(Verdict { outcome, measures })
}

/// Ported from the runner's `test_backpressure_input_slow_consumer`.
async fn backpressure_slow_reader() -> Result<Verdict, Verdict> {
    input_backpressure(4, 10, 5).await
}

/// Ported from the runner's `test_backpressure_minimal_buffer`.
async fn backpressure_minimal_buffer() -> Result<Verdict, Verdict> {
    input_backpressure(1, 10, 5).await
}

/// Ported from the runner's `test_backpressure_no_data_loss`.
async fn backpressure_no_data_loss() -> Result<Verdict, Verdict> {
    input_backpressure(4, 20, 2).await
}

/// A guest
/// that produces on demand, against a host that reads every 20ms. If the
/// slow reader reaches back to the guest, the guest's own stamps spread out
/// to match; if it does not, the guest ran to completion at once and whatever
/// it produced is sitting in memory.
async fn backpressure_slow_drain() -> Result<Verdict, Verdict> {
    // More than the 32 slots a two-hop 16-deep pipeline holds, so a runtime
    // that never pulls lazily cannot pass by having room to spare. Not a
    // multiple of 16, because that is its own probe below.
    let source = render(
        fixture!("response-pull-stamped.js"),
        &[("CHUNKS", "60".to_string())],
    );

    let exchange = exchange(source, quick(), get(), Some(Duration::from_millis(20))).await?;
    let mut measures = exchange.measures();

    let text = exchange.text();
    let host_span = exchange.drained.total.as_millis() as u64;
    let guest_span = stamp_spread(&text);

    let outcome = match guest_span {
        _ if text.lines().count() != 60 => Outcome::wrong(format!(
            "expected 60 stamped lines, got {}",
            text.lines().count()
        )),
        Some(guest) => {
            measures.note(format!(
                "guest produced over {guest}ms while the host read over {host_span}ms"
            ));

            if guest * 2 >= host_span {
                Outcome::Streams
            } else {
                Outcome::Buffers
            }
        }
        None => Outcome::wrong("no guest timestamps to compare"),
    };

    Ok(Verdict { outcome, measures })
}

// ---------------------------------------------------------------------------
// integrity
// ---------------------------------------------------------------------------

/// 32 MiB out of the guest in 64 KiB chunks, compared by digest so a
/// dropped or reordered chunk cannot pass as a size match.
async fn integrity_response() -> Result<Verdict, Verdict> {
    let source = render(
        fixture!("response-generate.js"),
        &[
            ("CHUNKS", BULK_CHUNKS.to_string()),
            ("CHUNK_SIZE", CHUNK.to_string()),
        ],
    );

    let exchange = exchange(source, bulk(), get(), None).await?;
    let mut measures = exchange.measures();

    let expected = digest(&payload(BULK_CHUNKS));
    let actual = digest(&exchange.drained.bytes());

    let outcome = if actual == expected {
        Outcome::Ok
    } else {
        Outcome::wrong(format!(
            "sha256 {} bytes out, expected {} bytes",
            exchange.drained.bytes().len(),
            BULK_CHUNKS * CHUNK
        ))
    };

    measures.note(format!("sha256 expected {}", &expected[..16]));
    measures.note(format!("sha256 received {}", &actual[..16]));

    Ok(Verdict { outcome, measures })
}

/// The same 32 MiB in and straight back out again, digested on both
/// sides. Optionally under a heap cap smaller than the payload, which is the
/// question the p3 tests ask of the wasm guest.
async fn integrity_echo(heap_max_mb: Option<usize>) -> Result<Verdict, Verdict> {
    let bytes = payload(BULK_CHUNKS);
    let expected = digest(&bytes);
    let limits = match heap_max_mb {
        Some(mb) => capped(mb),
        None => bulk(),
    };

    if let Some(mb) = heap_max_mb {
        // A cap the engine cannot even boot under would report as a streaming
        // failure, which would be a lie about a different thing.
        if let Err(detail) = start(fixture!("hello.js").to_string(), capped(mb)).await {
            return Ok(Verdict::new(Outcome::skipped(format!(
                "the backend does not start under a {mb}MB heap cap: {detail}"
            ))));
        }
    }

    let (tx, rx) = mpsc::channel(16);

    feed(tx, chunks_of(&bytes, CHUNK), Duration::ZERO);

    let exchange = exchange(
        fixture!("request-echo.js").to_string(),
        limits,
        post(RequestBody::Stream(rx)),
        None,
    )
    .await?;

    let actual = digest(&exchange.drained.bytes());
    let mut measures = exchange.measures();

    let outcome = if actual == expected {
        Outcome::Ok
    } else {
        Outcome::wrong(format!(
            "sha256 mismatch on {} bytes back of {}",
            exchange.drained.bytes().len(),
            bytes.len()
        ))
    };

    measures.note(format!("sha256 expected {}", &expected[..16]));
    measures.note(format!("sha256 received {}", &actual[..16]));

    Ok(Verdict { outcome, measures })
}

async fn integrity_echo_uncapped() -> Result<Verdict, Verdict> {
    integrity_echo(None).await
}

async fn integrity_echo_capped() -> Result<Verdict, Verdict> {
    integrity_echo(Some(16)).await
}

// ---------------------------------------------------------------------------
// the corpus
// ---------------------------------------------------------------------------

const BUDGET: Duration = Duration::from_secs(30);
const BULK_BUDGET: Duration = Duration::from_secs(180);

pub fn all() -> Vec<Case> {
    vec![
        Case {
            name: "request_text",
            dimension: "request streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(request_text().await) }),
        },
        Case {
            name: "request_reader",
            dimension: "request streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(request_reader().await) }),
        },
        Case {
            name: "request_json",
            dimension: "request streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(request_json().await) }),
        },
        Case {
            name: "request_binary",
            dimension: "request streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(request_binary().await) }),
        },
        Case {
            name: "request_utf8_boundary",
            dimension: "request streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(request_utf8_boundary().await) }),
        },
        Case {
            name: "request_large",
            dimension: "request streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(request_large().await) }),
        },
        Case {
            name: "request_empty",
            dimension: "request streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(request_empty().await) }),
        },
        Case {
            name: "request_double_consume",
            dimension: "request streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(request_double_consume().await) }),
        },
        Case {
            name: "request_never_consumed",
            dimension: "request streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(request_never_consumed().await) }),
        },
        Case {
            name: "request_partial_read",
            dimension: "request streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(request_partial_read().await) }),
        },
        Case {
            name: "request_error_midstream",
            dimension: "mid-stream error",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(request_error_midstream().await) }),
        },
        Case {
            name: "response_ttfc",
            dimension: "response streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(response_ttfc().await) }),
        },
        Case {
            name: "response_buffered_shape",
            dimension: "response streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(response_buffered_shape().await) }),
        },
        Case {
            name: "response_echo",
            dimension: "response streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(response_echo().await) }),
        },
        Case {
            name: "response_pipeline_multiple",
            dimension: "response streaming",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(response_pipeline_multiple().await) }),
        },
        Case {
            name: "response_error_midstream",
            dimension: "mid-stream error",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(response_error_midstream().await) }),
        },
        Case {
            name: "disconnect_recovery",
            dimension: "cancellation",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(disconnect_recovery().await) }),
        },
        Case {
            name: "bidirectional_transform",
            dimension: "bidirectional",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(bidirectional_transform().await) }),
        },
        Case {
            name: "backpressure_slow_reader",
            dimension: "backpressure",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(backpressure_slow_reader().await) }),
        },
        Case {
            name: "backpressure_minimal_buffer",
            dimension: "backpressure",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(backpressure_minimal_buffer().await) }),
        },
        Case {
            name: "backpressure_no_data_loss",
            dimension: "backpressure",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(backpressure_no_data_loss().await) }),
        },
        Case {
            name: "backpressure_slow_drain",
            dimension: "backpressure",
            budget: BUDGET,
            run: || Box::pin(async { Verdict::either(backpressure_slow_drain().await) }),
        },
        Case {
            name: "integrity_response_32mib",
            dimension: "integrity",
            budget: BULK_BUDGET,
            run: || Box::pin(async { Verdict::either(integrity_response().await) }),
        },
        Case {
            name: "integrity_echo_32mib",
            dimension: "integrity",
            budget: BULK_BUDGET,
            run: || Box::pin(async { Verdict::either(integrity_echo_uncapped().await) }),
        },
        Case {
            name: "integrity_echo_32mib_capped",
            dimension: "integrity",
            budget: BULK_BUDGET,
            run: || Box::pin(async { Verdict::either(integrity_echo_capped().await) }),
        },
    ]
}

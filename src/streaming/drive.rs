//! Driving a fetch while watching the body cross the boundary.
//!
//! The runtimes disagree on who drives the stream, so a probe that awaited
//! `exec` before reading the body would deadlock against one and mismeasure
//! another. Everything here polls `exec` and the body at once, and records
//! when each chunk landed rather than what the totals add up to.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use std::time::Instant;

use bytes::Bytes;
use openworkers_core::HttpResponse;
use openworkers_core::ResponseBody;
use openworkers_core::TerminationReason;
use tokio::sync::oneshot;

/// An `exec` future, pinned so it can be polled across a probe.
pub type Exec<'a> = Pin<&'a mut (dyn Future<Output = Result<(), TerminationReason>> + 'a)>;

/// `exec`'s verdict, once it has one.
pub type ExecResult = Option<Result<(), TerminationReason>>;

/// One chunk as the host saw it: when it arrived, and what it held.
pub struct Chunk {
    pub at: Duration,
    pub bytes: Bytes,
}

/// Runs `fut` while `exec` keeps making progress, and remembers `exec`'s
/// verdict if it lands first. Without this, a probe that blocks on the body
/// deadlocks against a runtime that only produces from inside `exec`.
pub async fn while_running<F>(exec: &mut Exec<'_>, done: &mut ExecResult, fut: F) -> F::Output
where
    F: Future,
{
    tokio::pin!(fut);

    loop {
        tokio::select! {
            biased;

            output = &mut fut => return output,
            result = exec.as_mut(), if done.is_none() => *done = Some(result),
        }
    }
}

/// Sleeps without stalling the guest.
pub async fn pause(exec: &mut Exec<'_>, done: &mut ExecResult, delay: Duration) {
    while_running(exec, done, tokio::time::sleep(delay)).await;
}

/// Waits for the response head, keeping `exec` alive while it does.
pub async fn head(
    exec: &mut Exec<'_>,
    done: &mut ExecResult,
    rx: oneshot::Receiver<HttpResponse>,
) -> Result<HttpResponse, String> {
    while_running(exec, done, rx)
        .await
        .map_err(|_| "the worker sent no response".to_string())
}

/// What draining a response body produced.
pub struct Drained {
    /// The body arrived as `ResponseBody::Stream`, whatever it then did.
    pub streamed: bool,
    pub chunks: Vec<Chunk>,
    /// An `Err(...)` chunk, which ends the drain.
    pub error: Option<String>,
    /// How long the whole body took, measured from `started`.
    pub total: Duration,
}

impl Drained {
    pub fn bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();

        for chunk in &self.chunks {
            out.extend_from_slice(&chunk.bytes);
        }

        out
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.bytes()).into_owned()
    }

    /// Time the first byte landed.
    pub fn ttfc(&self) -> Option<Duration> {
        self.chunks.first().map(|chunk| chunk.at)
    }
}

/// Drains a response body, optionally pausing `per_chunk` between reads to
/// stand in for a slow client.
pub async fn drain(
    exec: &mut Exec<'_>,
    done: &mut ExecResult,
    started: Instant,
    body: ResponseBody,
    per_chunk: Option<Duration>,
) -> Drained {
    let mut drained = Drained {
        streamed: matches!(body, ResponseBody::Stream(_)),
        chunks: Vec::new(),
        error: None,
        total: Duration::ZERO,
    };

    match body {
        ResponseBody::None => {}
        ResponseBody::Bytes(bytes) => drained.chunks.push(Chunk {
            at: started.elapsed(),
            bytes,
        }),
        ResponseBody::Stream(mut rx) => loop {
            match while_running(exec, done, rx.recv()).await {
                Some(Ok(bytes)) => drained.chunks.push(Chunk {
                    at: started.elapsed(),
                    bytes,
                }),
                Some(Err(error)) => {
                    drained.error = Some(error);

                    break;
                }
                None => break,
            }

            if let Some(delay) = per_chunk {
                pause(exec, done, delay).await;
            }
        },
    }

    drained.total = started.elapsed();

    drained
}

/// Gives `exec` a last chance to finish once the body is done, so the probe
/// can say whether the worker came back or was still going.
pub async fn settle(exec: &mut Exec<'_>, done: &mut ExecResult, grace: Duration) {
    if done.is_some() {
        return;
    }

    if let Ok(result) = tokio::time::timeout(grace, exec.as_mut()).await {
        *done = Some(result);
    }
}

/// How `exec` ended, as a line for the report.
pub fn exec_note(done: &ExecResult) -> String {
    match done {
        None => "exec was still running when the probe ended".to_string(),
        Some(Ok(())) => "exec returned Ok".to_string(),
        Some(Err(reason)) => format!("exec returned Err({reason:?})"),
    }
}

//! The streaming battery: what each backend actually does with a body that
//! does not arrive all at once.
//!
//! Separate from the guest test suite on purpose. The suite asks what a script
//! can observe about itself; this asks what crosses the host boundary, which
//! only the host can see. A probe that reports `buffers` has succeeded at its
//! job.

pub mod drive;
pub mod outcome;

#[cfg(feature = "_js")]
mod cases;

#[cfg(feature = "wasm")]
mod cases_wasm;

#[cfg(feature = "_js")]
use cases as corpus;

#[cfg(feature = "wasm")]
use cases_wasm as corpus;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use std::sync::Once;
use std::sync::OnceLock;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use crate::runtime::NAME;
use outcome::Measures;
use outcome::Outcome;
use outcome::Probe;
use outcome::Run;

/// What one probe concluded, and the numbers behind it.
pub struct Verdict {
    pub outcome: Outcome,
    pub measures: Measures,
}

impl Verdict {
    pub fn new(outcome: Outcome) -> Self {
        Self {
            outcome,
            measures: Measures::default(),
        }
    }

    /// A case body uses `Err` to leave early, so both arms are verdicts.
    pub fn either(result: Result<Self, Self>) -> Self {
        match result {
            Ok(verdict) | Err(verdict) => verdict,
        }
    }
}

/// Probe futures hold engine handles that are not `Send`, hence the local box.
pub type CaseFuture = Pin<Box<dyn Future<Output = Verdict>>>;

pub struct Case {
    pub name: &'static str,
    pub dimension: &'static str,
    pub budget: Duration,
    pub run: fn() -> CaseFuture,
}

/// Panic messages, by the thread that raised them. A probe thread that dies
/// takes its channel with it, so the message has to be recorded on the way
/// out or the report can only say "it stopped".
fn panics() -> &'static Mutex<HashMap<String, String>> {
    static PANICS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

    PANICS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn capture_panics() {
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        std::panic::set_hook(Box::new(|info| {
            let thread = std::thread::current();
            let name = thread.name().unwrap_or("unnamed").to_string();
            let message = match info.payload().downcast_ref::<&str>() {
                Some(message) => (*message).to_string(),
                None => match info.payload().downcast_ref::<String>() {
                    Some(message) => message.clone(),
                    None => "panicked".to_string(),
                },
            };

            let location = info
                .location()
                .map(|at| format!(" at {}:{}", at.file(), at.line()))
                .unwrap_or_default();

            if let Ok(mut panics) = panics().lock() {
                panics.insert(name, format!("{message}{location}"));
            }
        }));
    });
}

fn panic_detail(name: &str) -> String {
    panics()
        .lock()
        .ok()
        .and_then(|panics| panics.get(name).cloned())
        .unwrap_or_else(|| "the probe thread died without a message".to_string())
}

/// Runs the corpus, one probe per thread.
///
/// A probe gets its own thread and its own runtime so that a hang costs one
/// line rather than the run, and so that a panic inside a runtime is reported
/// instead of taking the process down. A thread that hangs is left where it
/// is; the binary exits without waiting for it.
pub fn run(filter: Option<&str>) -> Run {
    capture_panics();

    let mut probes = Vec::new();

    for case in corpus::all() {
        if let Some(filter) = filter
            && !case.name.contains(filter)
            && !case.dimension.contains(filter)
        {
            continue;
        }

        probes.push(probe(&case));
    }

    Run {
        backend: NAME,
        probes,
    }
}

fn probe(case: &Case) -> Probe {
    let (tx, rx) = std::sync::mpsc::channel();
    let run = case.run;

    let spawned = std::thread::Builder::new()
        .name(case.name.to_string())
        // A JS engine wants more than the 2 MiB a spawned thread gets.
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("the probe needs a runtime");

            let local = tokio::task::LocalSet::new();
            let verdict = runtime.block_on(local.run_until(run()));

            let _ = tx.send(verdict);
        });

    if let Err(e) = spawned {
        return Probe {
            name: case.name,
            dimension: case.dimension,
            outcome: Outcome::rejects(format!("could not start the probe thread: {e}")),
            measures: Measures::default(),
        };
    }

    let (outcome, measures) = match rx.recv_timeout(case.budget) {
        Ok(verdict) => (verdict.outcome, verdict.measures),
        Err(RecvTimeoutError::Timeout) => {
            let mut measures = Measures::default();

            measures.note(format!(
                "nothing came back within {}s",
                case.budget.as_secs()
            ));

            (Outcome::Hangs, measures)
        }
        Err(RecvTimeoutError::Disconnected) => (
            Outcome::Panics {
                detail: panic_detail(case.name),
            },
            Measures::default(),
        ),
    };

    Probe {
        name: case.name,
        dimension: case.dimension,
        outcome,
        measures,
    }
}

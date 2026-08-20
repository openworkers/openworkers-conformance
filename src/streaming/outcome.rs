//! What a streaming probe can say, and how a run of them is rendered.
//!
//! A probe never asserts that a backend streams. It reports what the backend
//! did, because "this one buffers" is the measurement, not a failure.

use serde::Serialize;

/// The six things a backend can do with a streaming body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Outcome {
    /// Bytes crossed the boundary while the other side was still producing.
    Streams,
    /// Correct bytes, but only after the whole body was collected somewhere.
    Buffers,
    /// Correct, and streaming was not what this probe asked about.
    Ok,
    /// It answered, with the wrong bytes.
    Wrong { detail: String },
    /// It refused: an error, a missing response, an unsupported shape.
    Rejects { detail: String },
    /// Nothing came back inside the probe deadline.
    Hangs,
    /// The runtime panicked.
    Panics { detail: String },
    /// The probe does not apply to this backend.
    Skipped { detail: String },
}

impl Outcome {
    pub fn label(&self) -> &'static str {
        match self {
            Outcome::Streams => "streams",
            Outcome::Buffers => "buffers",
            Outcome::Ok => "ok",
            Outcome::Wrong { .. } => "wrong",
            Outcome::Rejects { .. } => "rejects",
            Outcome::Hangs => "hangs",
            Outcome::Panics { .. } => "panics",
            Outcome::Skipped { .. } => "skipped",
        }
    }

    pub fn detail(&self) -> Option<&str> {
        match self {
            Outcome::Wrong { detail }
            | Outcome::Rejects { detail }
            | Outcome::Panics { detail }
            | Outcome::Skipped { detail } => Some(detail),
            _ => None,
        }
    }

    pub fn wrong(detail: impl Into<String>) -> Self {
        Outcome::Wrong {
            detail: detail.into(),
        }
    }

    pub fn rejects(detail: impl Into<String>) -> Self {
        Outcome::Rejects {
            detail: detail.into(),
        }
    }

    pub fn skipped(detail: impl Into<String>) -> Self {
        Outcome::Skipped {
            detail: detail.into(),
        }
    }
}

/// The numbers behind one probe. All times are milliseconds from `exec` start.
#[derive(Debug, Default, Clone, Serialize)]
pub struct Measures {
    /// Time the first byte of the body reached the host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttfc_ms: Option<u64>,
    /// Time the body ended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_ms: Option<u64>,
    /// Chunks the host received, before any reassembly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunks: Option<usize>,
    /// Bytes the host received.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<usize>,
    /// Free-form facts worth keeping next to the verdict.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl Measures {
    pub fn note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }
}

/// One probe, its verdict and its numbers.
#[derive(Debug, Clone, Serialize)]
pub struct Probe {
    pub name: &'static str,
    pub dimension: &'static str,
    pub outcome: Outcome,
    pub measures: Measures,
}

/// Everything one backend did, in the order the probes ran.
#[derive(Debug, Serialize)]
pub struct Run {
    pub backend: &'static str,
    pub probes: Vec<Probe>,
}

impl Run {
    /// One line per probe, plus a per-dimension tally.
    pub fn render(&self) -> String {
        let mut out = String::new();
        let width = self
            .probes
            .iter()
            .map(|probe| probe.name.len())
            .max()
            .unwrap_or(0);

        out.push_str(&format!("streaming: {}\n\n", self.backend));

        let mut dimension = "";

        for probe in &self.probes {
            if probe.dimension != dimension {
                dimension = probe.dimension;

                out.push_str(&format!("{dimension}\n"));
            }

            out.push_str(&format!(
                "  {:width$}  {:<8}",
                probe.name,
                probe.outcome.label()
            ));

            let measures = &probe.measures;

            if let (Some(ttfc), Some(total)) = (measures.ttfc_ms, measures.total_ms) {
                out.push_str(&format!("  ttfc {ttfc}ms / total {total}ms"));
            } else if let Some(total) = measures.total_ms {
                out.push_str(&format!("  total {total}ms"));
            }

            if let Some(chunks) = measures.chunks {
                out.push_str(&format!("  {chunks} chunks"));
            }

            if let Some(bytes) = measures.bytes {
                out.push_str(&format!("  {bytes} bytes"));
            }

            out.push('\n');

            if let Some(detail) = probe.outcome.detail() {
                out.push_str(&format!("  {:width$}  -> {detail}\n", ""));
            }

            for note in &measures.notes {
                out.push_str(&format!("  {:width$}     {note}\n", ""));
            }
        }

        out.push_str(&format!("\n{}\n", self.tally()));

        out
    }

    fn tally(&self) -> String {
        let mut counts: Vec<(&str, usize)> = Vec::new();

        for probe in &self.probes {
            let label = probe.outcome.label();

            match counts.iter_mut().find(|(name, _)| *name == label) {
                Some((_, count)) => *count += 1,
                None => counts.push((label, 1)),
            }
        }

        counts
            .iter()
            .map(|(label, count)| format!("{label} {count}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

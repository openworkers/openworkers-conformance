//! Scoreboard: what each runtime actually implements, not whether it passed.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct TestOutcome {
    pub name: String,
    pub passed: bool,
    pub error: String,
}

#[derive(Clone, Serialize)]
pub struct FileReport {
    pub area: String,
    pub path: String,
    /// Set when the file never ran: parse error, worker init failure, timeout.
    pub error: Option<String>,
    pub tests: Vec<TestOutcome>,
}

impl FileReport {
    /// The file was found but never reached the guest.
    pub fn aborted(
        area: &str,
        path: &str,
        declared: Vec<String>,
        reason: impl Into<String>,
    ) -> Self {
        let reason = reason.into();

        Self {
            area: area.to_string(),
            path: path.to_string(),
            error: Some(reason.clone()),
            tests: declared
                .into_iter()
                .map(|name| TestOutcome {
                    name,
                    passed: false,
                    error: reason.clone(),
                })
                .collect(),
        }
    }

    /// The suite itself is broken, so the file has no denominator at all.
    pub fn unreadable(area: &str, path: &str, reason: impl Into<String>) -> Self {
        Self {
            area: area.to_string(),
            path: path.to_string(),
            error: Some(reason.into()),
            tests: Vec::new(),
        }
    }

    pub fn passed(&self) -> usize {
        self.tests.iter().filter(|test| test.passed).count()
    }

    pub fn total(&self) -> usize {
        self.tests.len()
    }

    pub fn first_failure(&self) -> Option<&TestOutcome> {
        self.tests.iter().find(|test| !test.passed)
    }
}

#[derive(Serialize)]
pub struct Suite {
    pub backend: String,
    pub passed: usize,
    pub total: usize,
    pub files: Vec<FileReport>,
}

impl Suite {
    pub fn new(backend: &str, files: Vec<FileReport>) -> Self {
        Self {
            backend: backend.to_string(),
            passed: files.iter().map(FileReport::passed).sum(),
            total: files.iter().map(FileReport::total).sum(),
            files,
        }
    }

    fn areas(&self) -> BTreeMap<&str, (usize, usize)> {
        let mut areas: BTreeMap<&str, (usize, usize)> = BTreeMap::new();

        for file in &self.files {
            let entry = areas.entry(file.area.as_str()).or_default();

            entry.0 += file.passed();
            entry.1 += file.total();
        }

        areas
    }

    pub fn render(&self, verbose: bool) -> String {
        let mut out = String::new();
        let width = self
            .files
            .iter()
            .map(|file| file.path.len())
            .max()
            .unwrap_or(4)
            .max(4);

        let _ = writeln!(out, "backend: {}", self.backend);
        let _ = writeln!(out, "files:   {}", self.files.len());
        let _ = writeln!(out);
        let _ = writeln!(out, "{:width$}  score      first failure", "file");
        let _ = writeln!(out, "{}", "-".repeat(width + 60));

        for file in &self.files {
            let score = format!("{}/{}", file.passed(), file.total());
            let note = match (&file.error, file.first_failure()) {
                (Some(error), _) => format!("file: {error}"),
                (None, Some(test)) => format!("{}: {}", test.name, test.error),
                (None, None) => String::new(),
            };

            let _ = writeln!(
                out,
                "{:width$}  {:<9}  {}",
                file.path,
                score,
                truncate(&note, 90)
            );

            if verbose {
                for test in file.tests.iter().filter(|test| !test.passed) {
                    let _ = writeln!(out, "{:width$}    FAIL {}: {}", "", test.name, test.error);
                }
            }
        }

        let _ = writeln!(out);
        let _ = writeln!(out, "{:14}  score      %", "area");
        let _ = writeln!(out, "{}", "-".repeat(34));

        for (area, (passed, total)) in self.areas() {
            let _ = writeln!(
                out,
                "{area:14}  {:<9}  {}",
                format!("{passed}/{total}"),
                percent(passed, total)
            );
        }

        let _ = writeln!(out, "{}", "-".repeat(34));
        let _ = writeln!(
            out,
            "{:14}  {:<9}  {}",
            "TOTAL",
            format!("{}/{}", self.passed, self.total),
            percent(self.passed, self.total)
        );

        out
    }
}

fn percent(passed: usize, total: usize) -> String {
    if total == 0 {
        return "-".to_string();
    }

    format!("{}%", passed * 100 / total)
}

pub fn truncate(text: &str, max: usize) -> String {
    let flat = text.replace('\n', " ");

    match flat.char_indices().nth(max) {
        Some((cut, _)) => format!("{}...", &flat[..cut]),
        None => flat,
    }
}

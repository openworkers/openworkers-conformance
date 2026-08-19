//! Collects the guest test files and runs them, one worker per file.

use std::path::{Path, PathBuf};

use crate::harness::run_file;
use crate::report::Suite;
use crate::runtime::NAME;

/// One guest file, and the area it scores under.
pub struct Entry {
    pub area: String,
    pub display: String,
    pub path: PathBuf,
}

/// Every `.js` file under `root`, sorted, filed under its parent directory.
pub fn collect(root: &Path, filter: Option<&str>) -> std::io::Result<Vec<Entry>> {
    let mut entries = Vec::new();

    walk(root, root, &mut entries)?;
    entries.sort_by(|a, b| a.display.cmp(&b.display));

    if let Some(filter) = filter {
        entries.retain(|entry| entry.display.contains(filter));
    }

    Ok(entries)
}

fn walk(root: &Path, dir: &Path, entries: &mut Vec<Entry>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();

        if path.is_dir() {
            walk(root, &path, entries)?;
            continue;
        }

        if path.extension().is_none_or(|ext| ext != "js") {
            continue;
        }

        let display = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();

        let area = path
            .parent()
            .and_then(Path::file_name)
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "root".to_string());

        entries.push(Entry {
            area,
            display,
            path,
        });
    }

    Ok(())
}

pub async fn run(entries: Vec<Entry>) -> Suite {
    let mut files = Vec::with_capacity(entries.len());

    for entry in entries {
        files.push(run_file(&entry.area, &entry.path, &entry.display).await);
    }

    Suite::new(NAME, files)
}

//! Records the V8 reference responses for the SvelteKit fixture.
//!
//!   cargo run --release --features v8 --example ssr_oracle [-- --check]
//!
//! Reads `scenarios.json`, runs every scenario on a fresh worker, and writes
//! `oracle.json` plus one body file per scenario. `--check` recomputes instead
//! and fails on any difference, so the oracle stays honest across rebuilds.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use openworkers_core::{
    BindingInfo, Event, HttpMethod, HttpRequest, HttpResponse, LogLevel, OpFuture,
    OperationsHandler, RequestBody, ResponseBody, RuntimeLimits, Script, WorkerCode,
};
use openworkers_runtime_v8::Worker;
use openworkers_transform::{CodeLanguage, parse_worker_code};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::task::LocalSet;

#[derive(Deserialize)]
struct Spec {
    bundle: String,
    base_url: String,
    bindings: Vec<String>,
    scenarios: Vec<Scenario>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Scenario {
    name: String,
    exercises: String,
    request: RequestSpec,
}

/// Headers are `name: value` strings so the manifest stays one line per header
/// and diffs cleanly. Order is significant, duplicates are allowed.
#[derive(Clone, Deserialize, Serialize)]
struct RequestSpec {
    method: String,
    path: String,
    headers: Vec<String>,
    body: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Oracle {
    runtime: String,
    runtime_rev: String,
    transform: String,
    base_url: String,
    bindings: Vec<String>,
    bundle: Artifact,
    lowered: Lowered,
    scenarios: Vec<Recorded>,
}

#[derive(PartialEq, Serialize, Deserialize)]
struct Artifact {
    path: String,
    bytes: usize,
    sha256: String,
}

/// The classic script the runtimes actually execute, derived from the bundle.
#[derive(PartialEq, Serialize, Deserialize)]
struct Lowered {
    bytes: usize,
    sha256: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Recorded {
    name: String,
    exercises: String,
    request: RequestSpec,
    response: RecordedResponse,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct RecordedResponse {
    status: u16,
    headers: Vec<String>,
    body_file: String,
    body_bytes: usize,
    body_sha256: String,
    /// False when a second request on the same worker answered differently,
    /// which would mean the fixture leaks state between requests.
    warm_identical: bool,
}

/// Every binding call 404s: the fixture is meant to run without static assets.
struct Ops;

impl OperationsHandler for Ops {
    fn handle_binding_fetch(
        &self,
        _binding: &str,
        _request: HttpRequest,
    ) -> OpFuture<'_, Result<HttpResponse, String>> {
        Box::pin(async {
            Ok(HttpResponse {
                status: 404,
                headers: Vec::new(),
                body: ResponseBody::None,
            })
        })
    }

    fn handle_log(&self, level: LogLevel, message: String) {
        println!("  [{level:?}] {message}");
    }
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn extension(headers: &[String]) -> &'static str {
    let content_type = headers
        .iter()
        .find(|h| h.to_ascii_lowercase().starts_with("content-type:"))
        .map(|h| h.as_str())
        .unwrap_or("");

    if content_type.contains("html") {
        "html"
    } else if content_type.contains("json") {
        "json"
    } else {
        "txt"
    }
}

async fn spawn(code: &WorkerCode, bindings: &[String]) -> Worker {
    let bindings = bindings.iter().map(BindingInfo::assets).collect();
    let script = Script::with_bindings(code.clone(), None, bindings);

    Worker::new_with_ops(
        script,
        Some(RuntimeLimits::default()),
        std::sync::Arc::new(Ops),
    )
    .await
    .expect("worker creation failed")
}

async fn dispatch(
    worker: &mut Worker,
    base_url: &str,
    spec: &RequestSpec,
) -> (u16, Vec<String>, Vec<u8>) {
    let headers: HashMap<String, String> = spec
        .headers
        .iter()
        .map(|h| {
            let (name, value) = h.split_once(": ").expect("header must be `name: value`");
            (name.to_string(), value.to_string())
        })
        .collect();

    let req = HttpRequest {
        method: spec.method.parse::<HttpMethod>().expect("bad method"),
        url: format!("{base_url}{}", spec.path),
        headers,
        body: match &spec.body {
            Some(b) => RequestBody::Bytes(b.clone().into_bytes().into()),
            None => RequestBody::None,
        },
    };

    let (task, rx) = Event::fetch(req);
    worker.exec(task).await.expect("exec failed");

    let res = rx.await.expect("no response");
    let body = res.body.collect().await.unwrap_or_default();

    let headers = res
        .headers
        .iter()
        .map(|(k, v)| format!("{k}: {v}"))
        .collect();

    (res.status, headers, body.to_vec())
}

fn git_rev(repo: &Path) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "--short", "HEAD"])
        .output();

    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => "unknown".to_string(),
    }
}

fn main() {
    let check = std::env::args().any(|a| a == "--check");
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = crate_dir.join("fixtures/sveltekit-app");

    let spec: Spec = serde_json::from_slice(
        &std::fs::read(root.join("scenarios.json")).expect("no scenarios.json"),
    )
    .expect("bad scenarios.json");

    let bundle = std::fs::read(root.join(&spec.bundle)).expect("no bundle, run `bun run build`");
    let lowered = parse_worker_code(&bundle, CodeLanguage::JavaScript).expect("transform failed");

    println!(
        "bundle    {} ({} bytes ESM) -> {} bytes classic script",
        spec.bundle,
        bundle.len(),
        lowered.len()
    );

    let code = WorkerCode::JavaScript(lowered.clone());
    std::fs::create_dir_all(root.join("oracle")).expect("cannot create oracle dir");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let recorded = LocalSet::new().block_on(&rt, async {
        let mut recorded = Vec::new();

        for scenario in &spec.scenarios {
            let mut worker = spawn(&code, &spec.bindings).await;
            let (status, headers, body) =
                dispatch(&mut worker, &spec.base_url, &scenario.request).await;
            let (warm_status, warm_headers, warm_body) =
                dispatch(&mut worker, &spec.base_url, &scenario.request).await;
            drop(worker);

            let warm_identical =
                warm_status == status && warm_headers == headers && warm_body == body;

            println!(
                "{:<24} {} {:>7} bytes{}",
                scenario.name,
                status,
                body.len(),
                if warm_identical { "" } else { "  WARM DIFFERS" }
            );

            let body_file = format!("oracle/{}.{}", scenario.name, extension(&headers));

            recorded.push((
                Recorded {
                    name: scenario.name.clone(),
                    exercises: scenario.exercises.clone(),
                    request: scenario.request.clone(),
                    response: RecordedResponse {
                        status,
                        headers,
                        body_file,
                        body_bytes: body.len(),
                        body_sha256: sha256(&body),
                        warm_identical,
                    },
                },
                body,
            ));
        }

        recorded
    });

    let oracle = Oracle {
        runtime: "openworkers-runtime-v8".to_string(),
        runtime_rev: git_rev(&crate_dir.join("../openworkers-runtime-v8")),
        transform: "openworkers-transform v0.1.0".to_string(),
        base_url: spec.base_url.clone(),
        bindings: spec.bindings.clone(),
        bundle: Artifact {
            path: spec.bundle.clone(),
            bytes: bundle.len(),
            sha256: sha256(&bundle),
        },
        lowered: Lowered {
            bytes: lowered.len(),
            sha256: sha256(lowered.as_bytes()),
        },
        scenarios: recorded.iter().map(|(r, _)| r.clone()).collect(),
    };

    let manifest_path = root.join("oracle.json");

    if check {
        let failures = compare(&manifest_path, &root, &oracle.bundle, &recorded);

        if failures > 0 {
            eprintln!("\n{failures} scenario(s) differ from the recorded oracle");
            std::process::exit(1);
        }

        println!(
            "\noracle matches on all {} scenarios",
            oracle.scenarios.len()
        );
        return;
    }

    for (rec, body) in &recorded {
        std::fs::write(root.join(&rec.response.body_file), body).expect("cannot write body");
    }

    let json = serde_json::to_string_pretty(&oracle).expect("serialize failed");
    std::fs::write(&manifest_path, format!("{json}\n")).expect("cannot write oracle.json");

    println!("\nwrote {}", manifest_path.display());
}

fn compare(
    manifest_path: &Path,
    root: &Path,
    bundle: &Artifact,
    bodies: &[(Recorded, Vec<u8>)],
) -> usize {
    let previous: Oracle =
        serde_json::from_slice(&std::fs::read(manifest_path).expect("no oracle.json"))
            .expect("bad oracle.json");

    let mut failures = 0;

    if &previous.bundle != bundle {
        println!(
            "bundle differs: recorded {}, current {}",
            previous.bundle.sha256, bundle.sha256
        );
        failures += 1;
    }

    for (rec, body) in bodies {
        let Some(old) = previous.scenarios.iter().find(|s| s.name == rec.name) else {
            println!("{}: not in the recorded oracle", rec.name);
            failures += 1;
            continue;
        };

        if old.response != rec.response {
            println!("{}: response differs", rec.name);
            failures += 1;
            continue;
        }

        let stored = std::fs::read(root.join(&old.response.body_file)).unwrap_or_default();

        if &stored != body {
            println!("{}: body file differs", rec.name);
            failures += 1;
        }
    }

    failures
}

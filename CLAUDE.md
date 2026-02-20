# CLAUDE.md

Project context for Claude Code sessions.

## What is this project?

A Rust-based conformance test suite for OpenWorkers runtimes. It runs Web Platform Tests (WPT) and custom JavaScript tests against the OpenWorkers v8 runtime to verify Web API compliance.

## Build & test commands

```sh
# Build (always use --features v8)
cargo build --features v8

# Run all tests
cargo test --features v8

# Run a specific test
cargo test --features v8 -- url::urlsearchparams_append

# Run a test category
cargo test --features v8 -- headers::
```

## Important: the v8 feature flag

The project does not compile without `--features v8`. The `Worker` type and runtime types are gated behind this feature. Always pass `--features v8` for build and test commands.

## Git submodule

The `tests/wpt/` directory is a git submodule pointing to `web-platform-tests/wpt`. After cloning or switching branches, initialize it:

```sh
git submodule update --init
```

## Architecture

- `src/lib.rs` - Crate root, exports `harness` and `runtime` modules
- `src/runtime.rs` - Re-exports `Worker` from `openworkers-runtime-v8` (behind `v8` feature) and provides `run_in_local` helper for tokio `LocalSet`
- `src/harness.rs` - Two test harnesses:
  - Custom: `run_js_test(code)` / `run_js_test_file(path)` - uses built-in JS assertion helpers
  - WPT: `run_wpt_test(wpt_path)` - runs real WPT `.any.js` tests with `testharness.js`
- `tests/wpt_tests/` - Rust integration tests organized by API (url, headers, encoding, response, crypto)

## Dependencies

- `openworkers-core` (v0.13.0) - Core types: `Event`, `HttpMethod`, `HttpRequest`, `RequestBody`, `Script`
- `openworkers-runtime-v8` (v0.13.3, optional) - The v8-based worker runtime
- Both are git dependencies from the openworkers GitHub org. The `openworkers-core` version must match the one used transitively by `openworkers-runtime-v8` to avoid type mismatches.

## Adding new WPT tests

1. Find the `.any.js` test under `tests/wpt/`
2. Add a `#[tokio::test(flavor = "current_thread")]` in the appropriate `tests/wpt_tests/*.rs` file
3. Call `run_wpt_test("path/relative/to/wpt/root.any.js").await`
4. Register new modules in `tests/wpt_tests/main.rs`

## Current test status

As of last run: 7 passing, 30 failing. Failures are genuine conformance gaps in the runtime (not harness bugs).

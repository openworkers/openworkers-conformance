# openworkers-conformance

Conformance test suite for [OpenWorkers](https://github.com/openworkers) runtimes. Validates Web API implementations against the [Web Platform Tests (WPT)](https://web-platform-tests.org/) and custom test cases.

## Prerequisites

- Rust (edition 2024)
- The `v8` feature requires the OpenWorkers v8 runtime

## Setup

Clone with submodules (the WPT test suite is a git submodule):

```sh
git clone --recurse-submodules https://github.com/openworkers/openworkers-conformance.git
```

If already cloned, initialize the submodule:

```sh
git submodule update --init
```

## Build

```sh
cargo build --features v8
```

## Run tests

```sh
cargo test --features v8
```

Run a specific test category:

```sh
cargo test --features v8 -- url::
cargo test --features v8 -- headers::
cargo test --features v8 -- encoding::
cargo test --features v8 -- response::
cargo test --features v8 -- crypto::
```

## Project structure

```
src/
  lib.rs          - Crate root, exports harness and runtime modules
  harness.rs      - Test harness (custom assertions + WPT runner)
  runtime.rs      - Runtime abstraction (v8 backend behind feature flag)
tests/
  wpt/            - WPT git submodule (web-platform-tests/wpt)
  wpt_tests/      - Rust test files that invoke WPT tests
    main.rs       - Test entry point
    url.rs        - URL/URLSearchParams conformance tests
    headers.rs    - Fetch Headers conformance tests
    encoding.rs   - TextEncoder/TextDecoder conformance tests
    response.rs   - Response API conformance tests
    crypto.rs     - WebCrypto conformance tests
```

## Test harness

Two modes are available:

- **Custom tests** (`run_js_test` / `run_js_test_file`): Run inline JS with built-in assertion helpers (`assert`, `assertEqual`, `assertThrows`, `assertType`). The JS code must define `async function __runTests()`.

- **WPT tests** (`run_wpt_test`): Run real `.any.js` files from the WPT suite using `testharness.js`. Supports `// META: script=` directives for additional script dependencies.

## Adding new tests

1. Find the WPT test file under `tests/wpt/` (e.g. `fetch/api/headers/headers-basic.any.js`)
2. Add a Rust test function in the appropriate file under `tests/wpt_tests/`:

```rust
#[tokio::test(flavor = "current_thread")]
async fn my_new_test() {
    run_wpt_test("path/to/test.any.js").await;
}
```

3. Register the module in `tests/wpt_tests/main.rs` if it's a new category.

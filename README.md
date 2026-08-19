# openworkers-conformance

One suite of guest JavaScript, run unchanged on every OpenWorkers runtime, to
measure what each backend actually implements.

A low score is a result, not a failure: the suite is written against the
[WinterTC (TC55) Minimum Common API](https://min-common-api.proposal.wintertc.org/)
and the specs it points at, never against what a runtime happens to do. Guest
assertions never set the exit code, so the run stays usable while the runtimes
are incomplete.

## Layout

`tests/` holds plain classic scripts. Each declares its tests with
`test('name', fn)` at column 0, and asserts with the helpers the harness
injects (`assert`, `assertEqual`, `assertType`, `assertArrayEqual`,
`assertThrows`, `assertRejects`). Nothing in a test file is backend-specific.

| area       | file                             | tests |
| ---------- | -------------------------------- | ----: |
| crypto     | `js/crypto/crypto.js`            |    30 |
| encoding   | `js/encoding/base64.js`          |    20 |
| encoding   | `js/encoding/text.js`            |    35 |
| globals    | `js/globals/abort.js`            |    24 |
| globals    | `js/globals/blob.js`             |    26 |
| globals    | `js/globals/microtasks.js`       |    10 |
| globals    | `js/globals/misc.js`             |    16 |
| globals    | `js/globals/structured-clone.js` |    17 |
| globals    | `js/globals/timers.js`           |    16 |
| headers    | `js/headers/headers.js`          |    28 |
| request    | `js/request/request.js`          |    29 |
| response   | `js/response/response.js`        |    38 |
| streams    | `js/streams/streams.js`          |    25 |
| url        | `js/url/search-params.js`        |    35 |
| url        | `js/url/url.js`                  |    33 |
| wintercg   | `wintercg/minimum-common-api.js` |    66 |
|            | **total**                        |   448 |

Every file runs in a fresh worker, and every test runs under a 2s deadline, so
one missing global costs its own line and nothing else. The denominator comes
from the source, not from the run: a backend that cannot parse a file still
scores 0/N against the same N as the others.

## Running

```bash
cargo run --release --features v8 --bin conformance
cargo run --release --features boa --bin conformance -- --verbose
cargo run --release --features quickjs --bin conformance -- --filter url
cargo run --release --features jsc --bin conformance -- --json
```

One backend per build, like the rest of the workspace. `--verbose` lists every
failing test, `--json` emits the whole run, `--filter` selects by path.

## Scoreboard

Measured 2026-08-19, one commit per runtime, all against `openworkers-core`
v0.14.0.

| area                 | v8      | jsc     | quickjs | boa     |
| -------------------- | ------- | ------- | ------- | ------- |
| crypto (30)          | 22      | 23      | 21      | 10      |
| encoding (55)        | 45      | 33      | 44      | 47      |
| globals (109)        | 86      | 37      | 43      | 80      |
| headers (28)         | 23      | 24      | 24      | 24      |
| request (29)         | 21      | 19      | 16      | 20      |
| response (38)        | 28      | 27      | 21      | 30      |
| streams (25)         | 13      | 12      | 11      | 13      |
| url (68)             | 46      | 66      | 66      | 66      |
| wintercg (66)        | 36      | 27      | 24      | 29      |
| **total (448)**      | **320** | **268** | **270** | **319** |
|                      | 71%     | 59%     | 60%     | 71%     |

| backend | branch                       | commit    |
| ------- | ---------------------------- | --------- |
| v8      | `feat/upstream-v8`           | `8f6710c` |
| jsc     | `feat/core-0.14`             | `2f82095` |
| quickjs | `feat/core-0.14`             | `7ca14d3` |
| boa     | `feat/core-0.14`             | `e19da53` |
| wasm    | `feat/core-0.14-wasmtime-bump` | `54ee096` |

### What the numbers say

84 of the 448 tests fail on all four backends, so most of the gap is the
platform's, not any one engine's.

- **v8 has the weakest `URL`**, at 46/68. It normalizes nothing: no
  default-port removal, no dot-segment removal, no percent-encoding, no
  punycode, no lowercasing, no setters, no `toJSON`, no `canParse`, and it
  accepts `new URL('not a url')`. jsc, quickjs and boa back `URL` with the
  `url` crate and score 66/68; boa is the only backend to take `url.js` whole.
- **boa scores like v8 overall** (319 against 320) on the strength of
  `boa_wintertc`, while being the weakest on crypto by far (10/30).
- **No backend exposes `Event` or `EventTarget`**, so the DOM event core of the
  Minimum Common API is missing everywhere, and with it `{ once: true }`,
  `CustomEvent` and `AbortSignal.any`. `AbortController` itself exists on v8
  and boa only.
- **No backend sorts `Headers` iteration** by name, and none validates a header
  name or value: `set('x a', ...)` and a value with a newline both go through.
- **No backend sets `Content-Type` from the body** on `Request` or `Response`,
  so a string body arrives without `text/plain;charset=UTF-8`, and none
  enforces the status rules (a body with 204, a status of 600).
- **Streams stop at `ReadableStream`**: `WritableStream`, `TransformStream`,
  the queuing strategies, `TextEncoderStream`/`TextDecoderStream` and
  `CompressionStream` are missing on all four, `tee` exists on v8 and boa only,
  and no backend makes a stream async-iterable.
- **`structuredClone`, `Blob` and `File` exist on v8 and boa only**, and
  `multipart/form-data` parses on v8 alone.
- **v8 `getRandomValues` takes `Uint8Array` only**, returning `undefined`
  instead of filling any other integer array; the other three handle all of
  them.
- **jsc `console.*` writes straight to stdout** with `println!`, ignoring the
  `OperationsHandler` it was given, which is why `--json` needs a
  `grep -v '^\[LOG\]'` on that backend.

## Backends

One cargo feature per backend, mutually exclusive, the same pattern as
`openworkers-task-executor`.

`wasm` builds and reports a skip: that runtime executes wasm components, not
JavaScript, so this suite has nothing to say about it. Reporting 0/448 there
would be a lie, not a measurement.

`nova` has no feature at all. `nova_vm` 1.0 pulls `temporal_rs` 0.1.2, written
against `icu_calendar` 2.1, while v8 152 pulls `temporal_capi` 0.2.3+, which
wants 2.2. One lockfile cannot hold both, and this crate needs v8.

## SSR oracle

`examples/ssr_oracle.rs` and `fixtures/sveltekit-app/` are a separate,
complementary measurement: 17 scenarios of a real SvelteKit app, recorded on v8
and diffed byte for byte by the other runtimes.

```bash
cargo run --release --features v8 --example ssr_oracle -- --check
```

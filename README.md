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

v8 measured 2026-09-12, the other three on 2026-09-11, one commit per runtime,
all against `openworkers-core` v0.15.0.

| area                 | v8      | jsc     | quickjs | boa     |
| -------------------- | ------- | ------- | ------- | ------- |
| crypto (30)          | 30      | 23      | 21      | 10      |
| encoding (55)        | 55      | 33      | 44      | 47      |
| globals (109)        | 109     | 37      | 43      | 80      |
| headers (28)         | 28      | 24      | 24      | 24      |
| request (29)         | 29      | 19      | 16      | 20      |
| response (38)        | 38      | 27      | 21      | 30      |
| streams (25)         | 25      | 12      | 11      | 13      |
| url (68)             | 68      | 66      | 66      | 66      |
| wintercg (66)        | 63      | 27      | 24      | 29      |
| **total (448)**      | **445** | **268** | **270** | **319** |
|                      | 99%     | 59%     | 60%     | 71%     |

Branch names lag the code: every one of these builds against core v0.15.0.

| backend | branch                         | commit    |
| ------- | ------------------------------ | --------- |
| v8      | `main`                         | `dfb5a08` |
| surface | `openworkers-wintertc` `main`  | `51a4a86` |
| jsc     | `feat/core-0.14`               | `2df2560` |
| quickjs | `feat/core-0.14`               | `ed14afe` |
| boa     | `feat/core-0.14`               | `9dec1ee` |
| wasm    | `feat/core-0.14-wasmtime-bump` | `c97afa3` |

### What the numbers say

3 of the 448 tests fail on all four backends, down from 77 in August. Every
one of the 74 that left the set left because v8 took it from
`openworkers-wintertc`: jsc, quickjs and boa score exactly what they scored in
August. v8 has 3 failures left and **not one of them is its own**: what it
still fails, every backend fails.

- **v8 is the only backend with a complete `URL` and a complete `crypto`**,
  68/68 and 30/30. Its `subtle` covers the six digests, AES-GCM round trips,
  raw HMAC and AES key import and export, and ECDSA key generation. jsc and
  quickjs stop after digest and HMAC; boa has no `subtle.digest` at all, which
  is most of the distance between its 10/30 and everyone else.
- **jsc and quickjs miss `URL.canParse` and `URL.parse`** and nothing else in
  `url.js`; boa takes `url.js` whole and loses its two points on
  `URLSearchParams`. All three back `URL` with the `url` crate.
- **boa is 126 behind v8** (319 against 445) on the strength of `boa_wintertc`,
  while being the weakest on crypto by far.
- **The DOM event core exists on v8 alone**, from `openworkers-wintertc`:
  `Event`, `EventTarget`, `CustomEvent`, `ErrorEvent`, `MessageEvent`,
  `PromiseRejectionEvent`, `MessagePort` and `MessageChannel`, with
  `AbortSignal` built on `EventTarget`. v8 takes `js/globals/abort.js` whole,
  24/24, against 11/24 for boa, 2/24 for quickjs and 0/24 for jsc.
- **`Headers` is complete on v8 alone**, 28/28, since it took the interface
  from `openworkers-wintertc`. The other three sort no iteration, reject
  neither an invalid name nor a value with a newline, and trim no whitespace
  around a value.
- **`Request` and `Response` are complete on v8**, 29/29 and 38/38, from
  `openworkers-wintertc`. The other three set no `Content-Type` from the body,
  so neither a string nor a `URLSearchParams` arrives with its type, and none
  enforces the status rules (a body with 204 or 304, a status below 200 or
  above 599).
- **Streams stop at `ReadableStream` on three of the four**: `WritableStream`,
  `TransformStream`, the queuing strategies, `TextEncoderStream`/
  `TextDecoderStream`, `pipeTo` and `ReadableStream.from` are missing on jsc,
  quickjs and boa, and only v8 makes a stream async-iterable. **v8 is the only
  backend with `CompressionStream` and `DecompressionStream`**, which compress
  chunk by chunk over a host codec instead of buffering the body.
  `ReadableByteStreamController`, `ReadableStreamBYOBReader` and
  `ReadableStreamBYOBRequest` are exposed nowhere: no backend implements a byte
  stream, and a class exposed without one behind it would be a score, not a
  measurement.
- **`structuredClone`, `Blob` and `File` exist on v8 and boa only**, and
  `multipart/form-data` parses on v8 alone.
- **`performance` exists on v8 and quickjs**, and v8 alone exposes the
  `Performance` interface the standard names behind it; `self` exists on v8 and
  boa. **The WebAssembly streaming entry points are on v8 alone**, and they
  check the response's status and media type before compiling.

## Streaming battery

The guest suite asks what a script can observe about itself. It cannot see a
body that does not arrive all at once, because only the host is on the other
end of the channel. `src/streaming/` is that second measurement, and
[STREAMING.md](STREAMING.md) holds its matrix.

```bash
cargo run --release --features v8 --bin streaming
cargo run --release --features jsc --bin streaming -- --filter backpressure
cargo run --release --features wasm --bin streaming -- --json
```

25 probes on the JavaScript backends, 4 on `wasm`, each in its own thread with
its own deadline, so a runtime that deadlocks mid-stream costs one line and
gets reported as `hangs` rather than taking the run with it.

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

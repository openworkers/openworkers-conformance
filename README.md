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

v8 and nova measured 2026-09-13, the other three 2026-09-12, one commit per
runtime, all against `openworkers-core` v0.15.0.

| area                 | v8      | nova    | boa     | quickjs | jsc     |
| -------------------- | ------- | ------- | ------- | ------- | ------- |
| crypto (30)          | 30      | 23      | 10      | 21      | 23      |
| encoding (55)        | 55      | 54      | 47      | 44      | 33      |
| globals (109)        | 109     | 103     | 80      | 43      | 37      |
| headers (28)         | 28      | **28**  | 24      | 24      | 24      |
| request (29)         | 29      | **29**  | 20      | 16      | 19      |
| response (38)        | 38      | **38**  | 30      | 21      | 27      |
| streams (25)         | 25      | 24      | 13      | 11      | 12      |
| url (68)             | 68      | **68**  | 66      | 66      | 66      |
| wintercg (66)        | 66      | 55      | 29      | 24      | 27      |
| **total (448)**      | **448** | **422** | **319** | **270** | **268** |
|                      | 100%    | 94%     | 71%     | 60%     | 59%     |

Branch names lag the code: every one of these builds against core v0.15.0.

| backend | branch                         | commit    |
| ------- | ------------------------------ | --------- |
| v8      | `main`                         | `6a898ab` |
| nova    | `main`                         | `2857ade` |
| boa     | `main`                         | `be42bf1` |
| surface | `openworkers-wintertc` `main`  | `e7825c2` |
| jsc     | `feat/core-0.14`               | `2df2560` |
| quickjs | `feat/core-0.14`               | `ed14afe` |
| wasm    | `feat/core-0.14-wasmtime-bump` | `c97afa3` |

### What the numbers say

No test fails on every backend any more, down from 77 in August. Every one of
the 77 that left the set left because v8 took it from `openworkers-wintertc`:
jsc, quickjs and boa score exactly what they scored in August. **v8 passes all
448**, so the set that measures what the backends share is now the set that
measures what they lack.

**nova scores 422**, against 242 on its own interfaces in August. The surface
took it to 368 with no op answered at all: thirteen of the nineteen modules ask
nothing of their host. Five ops took it the rest of the way, along with timers,
`encodeInto` and the four digests. Streams 0 to 24 of 25, `Request` 22 to 29,
`Response` 30 to 38, globals 18 to 103 of 109, crypto 9 to 23 of 30. Of the 26
it still fails, 7 ask for `WebAssembly`, which nova_vm does not have, and 8 for
a `CryptoKey`.

Two of the ops it cannot answer are held up by the same gap: `textEncode` and
the compression codecs hand back bytes, and nova_vm gives an embedder no way to
build a typed array. Everything else crosses as text, which is why `URL` is
answered by a parser that replies in JSON and `subtle.digest` in hex.

The same run found five deviations in the surface, caught by nova's own test
suite rather than by this one: a `FormData` body reached the wire as
`[object FormData]`, every method was uppercased where the standard normalizes
six, a body-less read marked the body used, a `Request` built from another took
its stream instead of a tee, and a `Headers` init accepted only an array.

jsc, quickjs and boa write every one of these interfaces themselves.

- **v8 is the only backend with a complete `crypto`**, 30/30, and shares a
  complete `URL` with nova, 68/68. Its `subtle` covers the six digests,
  AES-GCM round trips, raw HMAC and AES key import and export, and ECDSA key
  generation. jsc and quickjs stop after digest and HMAC; boa has no
  `subtle.digest` at all, which is most of the distance between its 10/30 and
  everyone else.
- **jsc and quickjs miss `URL.canParse` and `URL.parse`** and nothing else in
  `url.js`; boa takes `url.js` whole and loses its two points on
  `URLSearchParams`. All three back `URL` with the `url` crate.
- **boa is 129 behind v8** (319 against 448) on the strength of `boa_wintertc`,
  while being the weakest on crypto by far.
- **The DOM event core exists on v8 and nova**, from `openworkers-wintertc`:
  `Event`, `EventTarget`, `CustomEvent`, `ErrorEvent`, `MessageEvent`,
  `PromiseRejectionEvent`, `MessagePort` and `MessageChannel`, with
  `AbortSignal` built on `EventTarget`. v8 and nova take `js/globals/abort.js`
  whole, 24/24, against 11/24 for boa, 2/24 for quickjs and 0/24 for jsc.
- **`Headers` is complete on v8 and nova**, 28/28, on the interface both take
  from `openworkers-wintertc`. The other three sort no iteration, reject
  neither an invalid name nor a value with a newline, and trim no whitespace
  around a value.
- **`Request` and `Response` are complete on v8 and nova**, 29/29 and 38/38,
  from `openworkers-wintertc`. The other three set no `Content-Type` from the
  body, so neither a string nor a `URLSearchParams` arrives with its type, and
  none enforces the status rules (a body with 204 or 304, a status below 200
  or above 599).
- **Streams stop at `ReadableStream` on three of the five**: `WritableStream`,
  `TransformStream`, the queuing strategies, `TextEncoderStream`/
  `TextDecoderStream`, `pipeTo` and `ReadableStream.from` are missing on jsc,
  quickjs and boa, and only v8 and nova make a stream async-iterable. **v8 is
  the only backend with `CompressionStream` and `DecompressionStream`**, which
  compress chunk by chunk over a host codec instead of buffering the body, and
  the codec is an op nova does not answer.
  **The byte tier is on v8 and nova**: `ReadableByteStreamController`,
  `ReadableStreamBYOBReader` and `ReadableStreamBYOBRequest`, with the buffer
  transfer the standard asks of `read(view)`. The three other backends expose
  none of them, and asking one for a byob reader still returns a default reader
  whose `read` drops the view on the floor.
- **`structuredClone`, `Blob` and `File` exist on v8, nova and boa**, and
  `multipart/form-data` parses on v8 and nova.
- **`performance` exists on v8, nova and quickjs**, and quickjs alone lacks the
  `Performance` interface the standard names behind it; `self` exists on v8,
  nova and boa. **Timers are on v8 and nova**, 16/16 both, and on no one else.
- **The WebAssembly streaming entry points are on v8 alone**, and they check
  the response's status and media type before compiling: nova takes the module,
  but its engine has no `WebAssembly` for it to stand on.

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

`nova` builds with `temporal` off. `nova_vm` 1.0 with the feature pulls
`temporal_rs` 0.1.2, written against `icu_calendar` 2.1, while v8 152 pulls
`temporal_capi` 0.2.3+, which wants 2.2. One lockfile cannot hold both, and
this crate needs v8.

## SSR oracle

`examples/ssr_oracle.rs` and `fixtures/sveltekit-app/` are a separate,
complementary measurement: 17 scenarios of a real SvelteKit app, recorded on v8
and diffed byte for byte by the other runtimes.

```bash
cargo run --release --features v8 --example ssr_oracle -- --check
```

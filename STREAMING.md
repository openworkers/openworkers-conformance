# Streaming

What each backend does with a body that does not arrive all at once, measured
at the host boundary rather than from inside the guest.

The guest suite in `tests/` cannot answer this. A script can watch its own
`ReadableStream` behave, and every backend passes about half of
`js/streams/streams.js`, but none of that says whether a byte reached the host
before the last one was produced. Only the host is on the other end of
`ResponseBody::Stream`.

`buffers` is a result, not a failure. So is `rejects`. The battery reports what
happened and never asserts that a backend streams.

## Running

```bash
cargo run --release --features v8 --bin streaming
cargo run --release --features jsc --bin streaming -- --filter backpressure
cargo run --release --features wasm --bin streaming -- --json
```

Each probe runs in its own thread, with its own runtime and its own deadline. A
probe that hangs costs one line; the binary prints the report and exits without
waiting for the stuck thread. Nothing in this run reached its deadline, but a
deadlock mid-stream is the kind of thing this corpus exists to find, so the
guarantee stays.

| verdict   | meaning                                                   |
| --------- | --------------------------------------------------------- |
| `streams` | bytes crossed while the other side was still producing     |
| `buffers` | right bytes, but only after the whole body was collected   |
| `ok`      | right bytes, and streaming was not what the probe asked    |
| `wrong`   | it answered, with the wrong bytes                          |
| `rejects` | it refused: an error, no response, an unsupported shape    |
| `hangs`   | nothing came back inside the deadline                      |
| `panics`  | the runtime panicked                                       |
| `skipped` | the probe does not apply to this backend                   |

## Matrix

Measured 2026-08-21, all five backends against `openworkers-core` v0.15.0.
`n/a` means the probe belongs to the other corpus: wasm takes a component, not
a script, so it runs four probes of its own.

| probe                             | v8      | jsc     | quickjs | boa     | wasm    |
| --------------------------------- | ------- | ------- | ------- | ------- | ------- |
| **request streaming**             |         |         |         |         |         |
| `request_text`                    | ok      | rejects | rejects | rejects | n/a     |
| `request_reader`                  | streams | rejects | rejects | rejects | n/a     |
| `request_json`                    | ok      | rejects | rejects | rejects | n/a     |
| `request_binary`                  | ok      | rejects | rejects | rejects | n/a     |
| `request_utf8_boundary`           | ok      | rejects | rejects | rejects | n/a     |
| `request_large`                   | streams | rejects | rejects | rejects | n/a     |
| `request_empty`                   | ok      | rejects | rejects | rejects | n/a     |
| `request_double_consume`          | ok      | rejects | rejects | rejects | n/a     |
| `request_never_consumed`          | ok      | rejects | rejects | rejects | n/a     |
| `request_partial_read`            | ok      | rejects | rejects | rejects | n/a     |
| **response streaming**            |         |         |         |         |         |
| `response_ttfc`                   | streams | streams | buffers | wrong   | n/a     |
| `response_buffered_shape`         | ok      | ok      | ok      | ok      | buffers |
| `response_echo`                   | streams | rejects | rejects | rejects | n/a     |
| `response_pipeline_multiple`      | ok      | ok      | ok      | wrong   | n/a     |
| **bidirectional**                 |         |         |         |         |         |
| `bidirectional_transform`         | streams | rejects | rejects | rejects | n/a     |
| **mid-stream error**              |         |         |         |         |         |
| `request_error_midstream`         | ok      | rejects | rejects | rejects | ok      |
| `response_error_midstream`        | ok      | ok      | rejects | rejects | n/a     |
| **cancellation**                  |         |         |         |         |         |
| `disconnect_recovery`             | ok      | ok      | ok      | skipped | n/a     |
| **backpressure**                  |         |         |         |         |         |
| `backpressure_slow_reader`        | buffers | rejects | rejects | rejects | n/a     |
| `backpressure_minimal_buffer`     | buffers | rejects | rejects | rejects | n/a     |
| `backpressure_no_data_loss`       | buffers | rejects | rejects | rejects | n/a     |
| `backpressure_slow_drain`         | buffers | streams | buffers | wrong   | n/a     |
| **integrity**                     |         |         |         |         |         |
| `integrity_response_32mib`        | ok      | ok      | ok      | wrong   | n/a     |
| `integrity_echo_32mib`            | ok      | rejects | rejects | rejects | n/a     |
| `integrity_echo_32mib_capped`     | ok      | rejects | rejects | rejects | n/a     |
| `integrity_response_32mib_capped` | n/a     | n/a     | n/a     | n/a     | buffers |
| `integrity_request_32mib_capped`  | n/a     | n/a     | n/a     | n/a     | ok      |

| backend | totals                                  |
| ------- | --------------------------------------- |
| v8      | ok 16, streams 5, buffers 4             |
| jsc     | rejects 18, ok 5, streams 2             |
| quickjs | rejects 19, ok 4, buffers 2             |
| boa     | rejects 19, wrong 4, ok 1, skipped 1    |
| wasm    | buffers 2, ok 2                         |

## Measurements

Time to the first chunk against time to the last, both from the start of
`exec`, on a guest that emits 8 chunks 40ms apart and stamps each one with its
own clock.

| backend | first chunk | last chunk | chunks | guest produced over |
| ------- | ----------- | ---------- | -----: | ------------------- |
| v8      | 43ms        | 382ms      |      8 | 297ms               |
| jsc     | 43ms        | 384ms      |      8 | 299ms               |
| quickjs | 383ms       | 383ms      |      8 | 299ms               |
| boa     | never       | never      |      0 | -                   |

quickjs is the line to read twice: its guest paced itself correctly over 299ms,
and all eight chunks still reached the host in the same millisecond, 383ms in.
That is what a replay of an already-collected body looks like from the outside.

`backpressure_slow_drain` asks the reverse question, with a pull-driven guest of
60 chunks against a host that pauses 20ms per read. If the slow reader reaches
back, the guest's own stamps spread to match.

| backend | guest produced over | host read over | verdict |
| ------- | ------------------- | -------------- | ------- |
| jsc     | 703ms               | 1357ms         | streams |
| v8      | 536ms               | 1324ms         | buffers |
| quickjs | 2ms                 | 1368ms         | buffers |
| boa     | -                   | 0ms, 0 bytes   | wrong   |

`disconnect_recovery` drops the response receiver three chunks in, then fetches
the same worker again.

| backend | chunks, and when | exec back after | follow-up head |
| ------- | ---------------- | --------------- | -------------- |
| v8      | 3 over 38ms      | 13ms            | 0ms            |
| jsc     | 3 over 40ms      | 13ms            | 0ms            |
| quickjs | 3 over 2571ms    | 0ms             | 2547ms         |
| boa     | no stream        | -               | -              |

The rest, where a body arrived at all:

| probe                             | backend | first | last   | chunks |    bytes |
| --------------------------------- | ------- | ----- | ------ | -----: | -------: |
| `response_echo`                   | v8      | 0ms   | 29ms   |      4 |       12 |
| `bidirectional_transform`         | v8      | 1ms   | 33ms   |      5 |        6 |
| `integrity_response_32mib`        | v8      | 6ms   | 102ms  |    512 | 33554432 |
| `integrity_response_32mib`        | jsc     | 7ms   | 57ms   |    512 | 33554432 |
| `integrity_response_32mib`        | quickjs | 83ms  | 83ms   |    512 | 33554432 |
| `integrity_echo_32mib`            | v8      | 0ms   | 0ms    |    512 | 33554432 |
| `response_pipeline_multiple`      | v8      | 1ms   | 4ms    |     32 |      534 |
| `response_pipeline_multiple`      | jsc     | 1ms   | 3ms    |     32 |      534 |
| `integrity_response_32mib_capped` | wasm    | 10ms  | 10ms   |      1 | 33554432 |
| `integrity_request_32mib_capped`  | wasm    | 2ms   | 2ms    |      1 |        8 |

`integrity_echo_32mib` on v8 reads 0ms for both because the host had already
queued all 32 MiB into the request channel before `exec` started; the 512 chunks
crossing back with their boundaries intact is the measurement there, not the
clock.

## What the numbers say

**Two backends stream a response.** v8 and jsc both put the first of eight
40ms-paced chunks on the wire at 43ms and the last around 380ms, while the guest
is still producing. quickjs reports `ResponseBody::Stream` for every body,
including a plain string, and fills it from a buffer it has already
materialised. boa has no `ResponseBody::Stream` branch at all: it drains a
response body through a synchronous `__extractBody` helper, so whatever a guest
produces from a timer or a pull callback is gone by the time the helper runs,
and the host gets 0 bytes.

**Response backpressure reaches the guest on jsc only.** Against a host reading
one chunk every 20ms, the jsc guest spread its 60 chunks over 703ms, which
clears the probe's 2x threshold. The v8 guest spread over 536ms against a host
reading over 1324ms, so some pressure reaches it, but not enough to track the
reader; the probe calls that `buffers`. The quickjs guest was done in 2ms.

**A pull-driven guest that closes on a full pipeline delivers end-of-stream.**
`response_pipeline_multiple` closes after exactly 32 chunks, twice the 16-slot
high-water mark every runtime here picked, read as fast as the host can. v8, jsc
and quickjs all deliver 32 stamped lines. boa delivers none, for the same reason
it delivers none of anything asynchronous.

**Request streaming exists on v8 only, and it has no backpressure.** v8 hands
the guest a real `ReadableStream` with the chunk boundaries intact: 3 chunks in,
3 seen; 100 chunks in, 100 seen. But the host channel is drained as fast as it
fills, whatever its capacity. 10 chunks through a 4-slot channel are accepted in
0ms against a 15ms floor, 10 through a 1-slot channel likewise, and 20 through 4
slots in 0ms against a 16ms floor, all while the guest sleeps per chunk. A guest
that never reads the body still lets all 5 chunks through in 65ms with the
channel open, and `reader.cancel()` after the first chunk does not close it
either, so the producer uploads all 10 to a guest that stopped listening.

**The other three refuse a streamed request body, and all three say so.** jsc
answers `Other("Streaming request bodies are not supported: collect the body
into RequestBody::Bytes before calling exec")`, the only one of the three that
names the fix; quickjs and boa answer `Other("Streaming request bodies are not
supported")`. The refusal costs jsc 18 of its 25 probes and quickjs and boa 19
each, and it is worth more than any of them scoring on a body they quietly
truncated.

**A guest that errors its own stream reaches the host as an error.**
`controller.error(...)` after two chunks arrives on v8 and jsc as two chunks
followed by `Err("guest gave up mid-stream")` on the response channel, so a
truncated response is distinguishable from a complete one. quickjs turns the
same case into `Exception("Failed to read chunk: Exception generated by
QuickJS")` and sends no response at all, which loses the two good chunks but at
least says something happened. boa answers 200 with an empty body, the one shape
a caller cannot tell from a legitimate short response.

**An `Err` chunk in a request body stops the read.** `RequestBody::collect` in
core 0.15 returns at the first failed chunk, so the wasm runtime refuses the
request with `Other("request body failed: connection reset")` rather than
handing its guest 131072 stitched bytes to count as a complete upload. v8, which
does not collect, surfaces it inside the guest: `reader.read()` rejects with the
upstream message. The other three never reach the question, having refused the
streamed body outright.

**A client that leaves costs the worker its stream, not its life.** Three chunks
into a 200-chunk stream the host drops the receiver; on v8 and jsc `exec` comes
back 13ms later and the next fetch on the same worker answers with its head at
0ms. quickjs passes for a different reason: it had already run the guest to
completion before sending the head, so hanging up costs nothing, and the three
chunks the host asked for took 2571ms to appear. boa is skipped, having no
response stream to cancel.

**32 MiB survives the round trip where a round trip exists.** v8 echoes 32 MiB
back byte for byte in 512 chunks, uncapped and again under a 16 MB heap cap. v8
and jsc both generate the same 32 MiB out of a guest with a matching digest,
over 102ms and 57ms; quickjs matches the digest too but delivers all 512 chunks
in the same millisecond, 83ms in. boa produces 0 bytes. wasm moves 32 MiB in
both directions under an 8 MB cap with matching digests and hands the host one
`Bytes`: the p3 boundary streams inside the guest and buffers at `HttpResponse`.

**A plain string body is not a stream, except where it is.** v8, boa and wasm
return `ResponseBody::Bytes` for `new Response('hello')`; jsc and quickjs wrap
those five bytes in a `ResponseBody::Stream` of one chunk. A host that switches
on the variant to decide how to forward a body learns nothing from it.

**`stream_buffer_size` is decoration.** `RuntimeLimits` carries it, the probes
vary the request channel capacity from 1 to 16, and nothing about any
measurement changes on any backend.

## Provenance

Two suites already covered most of this, both v8 only:
`openworkers-runtime-v8/tests/request_body_stream_test.rs` against the `Worker`
trait, and `openworkers-runner/tests/request_body_streaming_test.rs` through
the runner. The names below are theirs where the scenario is theirs, so a
disagreement between the three can be looked up rather than re-derived.

| probe                         | from                                                |
| ----------------------------- | --------------------------------------------------- |
| `request_text`                | `test_request_body_stream_text`                      |
| `request_reader`              | `test_request_body_stream_reader`                    |
| `request_json`                | `test_request_body_stream_json`                      |
| `request_binary`              | `..._arraybuffer` + `..._binary_nulls`               |
| `request_utf8_boundary`       | `test_request_body_stream_utf8_boundary`             |
| `request_large`               | `test_request_body_stream_large`                     |
| `request_empty`               | `test_request_body_stream_empty`                     |
| `request_double_consume`      | `test_request_body_stream_double_consume`            |
| `request_never_consumed`      | `test_request_body_stream_never_consumed`            |
| `request_partial_read`        | `test_request_body_stream_partial_read`              |
| `request_error_midstream`     | `test_request_body_stream_error`                     |
| `response_echo`               | `test_request_body_stream_echo`                      |
| `bidirectional_transform`     | `..._bidirectional` + `test_backpressure_bidirectional` |
| `backpressure_slow_reader`    | `test_backpressure_input_slow_consumer`              |
| `backpressure_minimal_buffer` | `test_backpressure_minimal_buffer`                   |
| `backpressure_no_data_loss`   | `test_backpressure_no_data_loss`                     |
| `response_ttfc`               | new                                                  |
| `response_buffered_shape`     | new                                                  |
| `response_pipeline_multiple`  | new                                                  |
| `response_error_midstream`    | new                                                  |
| `disconnect_recovery`         | new                                                  |
| `backpressure_slow_drain`     | new                                                  |
| `integrity_response_32mib`    | new                                                  |
| `integrity_echo_32mib`        | new                                                  |
| `integrity_echo_32mib_capped` | new                                                  |

The ported ones asserted; here they measure, which is what makes them run on
five backends instead of one. `bidirectional_transform` keeps the runner's
cadence, the one that matters: a two-slot request channel fed every 5ms while
the host reads the doubled values back with 2ms between chunks.

`disconnect_recovery` states its contract in `src/streaming/cases.rs`: `exec`
has to come back within a second of the receiver being dropped, and the same
worker has to answer a second fetch. That second fetch is driven alongside
`exec` rather than awaited before the body is read, because a backend whose
`exec` ends only once the host has drained the body would otherwise deadlock
against the probe and be reported for it.

The four wasm probes are their own corpus, driven by the `fetch-worker-v3`
example in `openworkers-runtime-wasm`, because that backend takes a component
rather than a script. They ask the one question the p3 tests in
`openworkers-worker` deliberately do not: what shape the body has once it
reaches `HttpResponse`.

## Reproducing

Measured 2026-08-21 on aarch64-apple-darwin, against these checkouts. Every one
of them builds against `openworkers-core` v0.15.0, whatever its branch name
says.

| backend | branch                         | commit    |
| ------- | ------------------------------ | --------- |
| v8      | `main`                         | `fe3fb23` |
| jsc     | `feat/core-0.14`               | `2df2560` |
| quickjs | `feat/core-0.14`               | `ed14afe` |
| boa     | `feat/core-0.14`               | `9dec1ee` |
| wasm    | `feat/core-0.14-wasmtime-bump` | `c97afa3` |

The v8 numbers are pointer-compressed, the build the rest of the workspace uses.
`openworkers-v8` fetches its own prebuilt at build time
(`librusty_v8_ptrcomp_release_aarch64-apple-darwin.a.gz` from the
`openworkers/rusty-v8` release matching its version), so no `RUSTY_V8_*`
variable has to be set for any of this.

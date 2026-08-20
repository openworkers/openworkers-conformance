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

Each probe runs in its own thread, with its own runtime and its own deadline,
because two of the five backends deadlock somewhere in this corpus. A probe
that hangs costs one line; the binary prints the report and exits without
waiting for the stuck thread.

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

Measured 2026-08-20. `n/a` means the probe belongs to the other corpus: wasm
takes a component, not a script, so it runs four probes of its own.

| probe                             | v8      | jsc     | quickjs | boa     | wasm    |
| --------------------------------- | ------- | ------- | ------- | ------- | ------- |
| **request streaming**             |         |         |         |         |         |
| `request_text`                    | ok      | wrong   | rejects | wrong   | n/a     |
| `request_reader`                  | streams | rejects | rejects | wrong   | n/a     |
| `request_json`                    | ok      | rejects | rejects | wrong   | n/a     |
| `request_binary`                  | ok      | wrong   | rejects | wrong   | n/a     |
| `request_utf8_boundary`           | ok      | wrong   | rejects | wrong   | n/a     |
| `request_large`                   | streams | rejects | rejects | wrong   | n/a     |
| `request_empty`                   | ok      | ok      | rejects | ok      | n/a     |
| `request_double_consume`          | ok      | wrong   | rejects | wrong   | n/a     |
| `request_never_consumed`          | ok      | ok      | rejects | ok      | n/a     |
| `request_partial_read`            | ok      | rejects | rejects | wrong   | n/a     |
| **response streaming**            |         |         |         |         |         |
| `response_ttfc`                   | streams | hangs   | buffers | wrong   | n/a     |
| `response_buffered_shape`         | ok      | ok      | ok      | ok      | buffers |
| `response_echo`                   | streams | wrong   | rejects | wrong   | n/a     |
| `response_pipeline_multiple`      | hangs   | hangs   | ok      | wrong   | n/a     |
| **bidirectional**                 |         |         |         |         |         |
| `bidirectional_transform`         | streams | wrong   | rejects | wrong   | n/a     |
| **mid-stream error**              |         |         |         |         |         |
| `request_error_midstream`         | ok      | rejects | rejects | wrong   | wrong   |
| `response_error_midstream`        | wrong   | wrong   | rejects | rejects | n/a     |
| **cancellation**                  |         |         |         |         |         |
| `response_cancel_midstream`       | hangs   | hangs   | ok      | skipped | n/a     |
| **backpressure**                  |         |         |         |         |         |
| `backpressure_slow_reader`        | buffers | rejects | rejects | wrong   | n/a     |
| `backpressure_minimal_buffer`     | buffers | rejects | rejects | wrong   | n/a     |
| `backpressure_no_data_loss`       | buffers | rejects | rejects | wrong   | n/a     |
| `backpressure_slow_drain`         | hangs   | hangs   | buffers | wrong   | n/a     |
| **integrity**                     |         |         |         |         |         |
| `integrity_response_32mib`        | hangs   | hangs   | ok      | wrong   | n/a     |
| `integrity_echo_32mib`            | ok      | wrong   | rejects | wrong   | n/a     |
| `integrity_echo_32mib_capped`     | ok      | wrong   | rejects | wrong   | n/a     |
| `integrity_response_32mib_capped` | n/a     | n/a     | n/a     | n/a     | buffers |
| `integrity_request_32mib_capped`  | n/a     | n/a     | n/a     | n/a     | ok      |

| backend | totals                                          |
| ------- | ----------------------------------------------- |
| v8      | ok 12, streams 5, buffers 3, hangs 4, wrong 1    |
| jsc     | wrong 9, rejects 8, hangs 5, ok 3                |
| quickjs | rejects 19, buffers 2, ok 4                      |
| boa     | wrong 20, ok 3, rejects 1, skipped 1             |
| wasm    | buffers 2, ok 1, wrong 1                         |

## Measurements

Time to the first chunk against time to the last, both from the start of
`exec`, on a guest that emits 8 chunks 40ms apart and stamps each one with its
own clock.

| backend | first chunk | last chunk | chunks | guest produced over |
| ------- | ----------- | ---------- | -----: | ------------------- |
| v8      | 43ms        | 380ms      |      8 | 295ms               |
| quickjs | 378ms       | 378ms      |      8 | 294ms               |
| jsc     | never       | never      |      0 | -                   |
| boa     | never       | never      |      0 | -                   |

quickjs is the interesting line: its guest paced itself correctly over 294ms,
and all eight chunks still reached the host in the same millisecond, 378ms in.
That is what a replay of an already-collected body looks like from the outside.

The rest, where a body arrived at all:

| probe                       | backend | first  | last   | chunks | bytes    |
| --------------------------- | ------- | ------ | ------ | -----: | -------: |
| `response_echo`             | v8      | 0ms    | 26ms   |      4 |       12 |
| `bidirectional_transform`   | v8      | 0ms    | 32ms   |      5 |        6 |
| `integrity_echo_32mib`      | v8      | 0ms    | 0ms    |    512 | 33554432 |
| `backpressure_slow_drain`   | quickjs | 1ms    | 1307ms |     60 |     1010 |
| `integrity_response_32mib`  | quickjs | 34ms   | 34ms   |    512 | 33554432 |
| `response_ttfc`             | quickjs | 378ms  | 378ms  |      8 |      128 |
| `integrity_response_32mib_capped` | wasm | 9ms | 9ms |      1 | 33554432 |
| `integrity_request_32mib_capped`  | wasm | 43ms | 43ms |      1 |        8 |

`integrity_echo_32mib` on v8 reads 0ms for both because the host had already
queued all 32 MiB into the request channel before `exec` started; the 512 chunks
crossing back with their boundaries intact is the measurement there, not the
clock.

## What the numbers say

**One backend streams a response.** v8 is the only runtime where a chunk
reaches the host while the guest is still producing: 43ms to the first of eight
chunks paced 40ms apart, 380ms to the last. jsc sends the response head early
but never delivers the end of the stream. quickjs reports `ResponseBody::Stream`
for every body, including a plain string, and fills it from a `Vec<Vec<u8>>` it
has already materialised. boa has no `ResponseBody::Stream` branch at all, and
extracts a body by concatenating whatever is already sitting in the stream
controller's queue, so a pull-driven producer yields nothing.

**A guest `ReadableStream` that closes on a full buffer deadlocks v8.**
`response_pipeline_multiple` isolates it: a pull-driven producer closing after
exactly 32 chunks, read as fast as the host can, never delivers end-of-stream.
The trigger is the state of the 16-slot hop at `controller.close()`, not the
count:

| chunks | reader | result |
| -----: | ------ | ------ |
| 8      | fast   | ends   |
| 16     | fast   | hangs  |
| 17     | fast   | ends   |
| 32     | fast   | hangs  |
| 47     | fast   | ends   |
| 48     | fast   | hangs  |
| 24     | 20ms   | ends   |
| 60     | 20ms   | hangs  |

Below 32 chunks the two-hop pipeline swallows everything and nothing ever
blocks. Above it, a reader slow enough to keep the buffer full deadlocks at any
count. This takes `integrity_response_32mib` (512 chunks) and
`backpressure_slow_drain` (60 chunks, 20ms reader) with it, so v8 response
backpressure has no measurement here: it deadlocks as soon as it is exercised.
The path that forwards a request body straight back out is not affected, which
is why `integrity_echo_32mib` moves the same 512 chunks without trouble.

**A client that leaves takes the v8 worker with it.** After the host drops the
response receiver three chunks into a 200-chunk stream, `exec` returns `Ok`,
but only after 5001ms, and the next fetch on the same worker never returns. The
disconnect is detected; the recovery is not. jsc never gets that far, because
its paced stream delivers no chunks to cancel. quickjs passes for the wrong
reason: it had already run the guest to completion, so hanging up costs
nothing, and the three chunks the host asked for took 2414ms to appear.

**A guest that errors its own stream is indistinguishable from one that
finished.** `controller.error(...)` after two chunks arrives at the host as two
chunks and a clean end of channel on both v8 and jsc. Nothing ever puts an
`Err` on the response channel, so a truncated response reads as complete
downstream. quickjs turns the same case into
`Exception("Failed to read chunk")` and sends no response at all, which loses
the two good chunks but at least says something happened.

**An `Err` chunk in a request body is dropped on the way in.** The wasm probe
shows it most plainly, because its guest counts bytes: 64 KiB, then
`Err("connection reset")`, then another 64 KiB, and the guest reports 131072 as
a success. `RequestBody::collect` in `openworkers-core` keeps the `Ok` arm and
has no `else`, so every runtime that collects inherits it. v8, which does not
collect, is the only backend that surfaces it: the guest's `reader.read()`
rejects with the upstream message.

**Request streaming exists on v8 only, and it has no backpressure.** v8 hands
the guest a real `ReadableStream` with the chunk boundaries intact: 3 chunks in,
3 seen; 100 chunks in, 100 seen. But the host channel is drained as fast as it
fills, whatever its capacity. 20 chunks through a 4-slot channel are accepted in
0ms while the guest is still sleeping 2ms per chunk, 10 chunks through a
1-slot channel likewise, and a guest that never reads the body still lets all 5
chunks through. `reader.cancel()` does not close the channel either, so the
producer uploads all 10 chunks to a guest that stopped listening after the
first.

The other three refuse the shape rather than mishandle it, with varying
honesty. quickjs returns `Err(Other("Streaming request bodies are not
supported"))`, the only answer of the four a caller can act on. jsc turns
`RequestBody::Stream` into an empty body and answers 200: `text()` gives `""`,
`arrayBuffer()` gives 0 bytes, and `request.body` is null, so every reader
fixture throws and the worker burns its whole wall-clock budget before
returning `WallClockTimeout`. boa does the same and answers 500.

**32 MiB survives the round trip where a round trip exists.** v8 echoes 32 MiB
back byte for byte in 512 chunks, and does it again under a 16 MB heap cap.
wasm moves the same 32 MiB in both directions under an 8 MB cap with a matching
digest, but hands the host one `Bytes`: the p3 boundary streams inside the
guest and buffers at `HttpResponse`.

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
| `response_cancel_midstream`   | new                                                  |
| `backpressure_slow_drain`     | new                                                  |
| `integrity_response_32mib`    | new                                                  |
| `integrity_echo_32mib`        | new                                                  |
| `integrity_echo_32mib_capped` | new                                                  |

The ported ones asserted; here they measure, which is what makes them run on
five backends instead of one. `bidirectional_transform` keeps the runner's
cadence, the one that matters: a two-slot request channel fed every 5ms while
the host reads the doubled values back with 2ms between chunks.

The four wasm probes are their own corpus, driven by the `fetch-worker-v3`
example in `openworkers-runtime-wasm`, because that backend takes a component
rather than a script. They ask the one question the p3 tests in
`openworkers-worker` deliberately do not: what shape the body has once it
reaches `HttpResponse`.

## Reproducing

Measured 2026-08-20, against these checkouts:

| backend | branch                         | commit    |
| ------- | ------------------------------ | --------- |
| v8      | `feat/upstream-v8`             | `bdf102f` |
| jsc     | `feat/core-0.14`               | `b3a61b9` |
| quickjs | `feat/core-0.14`               | `7ca14d3` |
| boa     | `feat/core-0.14`               | `e19da53` |
| wasm    | `feat/core-0.14-wasmtime-bump` | `e385bae` |

The v8 numbers come from a build without pointer compression, because no
pointer-compressed v8 152 archive exists on this machine and `Cargo.toml` keeps
the `ptrcomp` feature the rest of the workspace uses. Heap layout has no
bearing on a body crossing a channel, but the two builds are not the same
binary.

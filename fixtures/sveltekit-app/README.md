# SvelteKit SSR fixture

A small but real SvelteKit app, built with `@openworkers/adapter-sveltekit`, plus
the responses `openworkers-runtime-v8` produces for it. Every other runtime
(deno, quickjs, jsc, nova, boa, wasm) runs the same bundle and diffs against
`oracle.json` and `oracle/`.

Nothing here is prerendered, so every scenario reaches `server.respond()`.

## What it exercises

| Route | Path through SvelteKit |
|---|---|
| `/` | page render, no server load |
| `/items/[id]` | route params, `+page.server.ts` load, `url.searchParams`, cookies in and out, `devalue.uneval` inlined in the HTML |
| `/items/[id]/__data.json` | the data-request path: `devalue.stringify` of `Date`, `Map`, `Set`, `BigInt`, `RegExp`, `undefined`, `NaN`, `-0`, `Infinity` and a repeated reference |
| `/items/[id]` POST | form actions, `x-www-form-urlencoded` body, `formData()`, `fail()`, `Set-Cookie`, CSRF origin check |
| `/items/999` | `error(404)` and the `+error.svelte` render |
| `/escape` | `escape_html` in text and attribute position, `{@html}` passthrough |
| `/api/echo` | `+server.ts` `GET`/`POST`, `json()`, `error(418)`, 405 from the router |
| `/go` | `redirect(302)` thrown from a load |
| `hooks.server.ts` | `locals` and a response header set in `handle` |

The item payload carries an accented latin letter, a CJK ideograph and an
astral emoji, so the UTF-8 round trip and SvelteKit's surrogate-pair escape
regex are both on the hot path.

## Rebuild

```bash
bun install
bun run build     # -> build/_worker.js, byte-reproducible
```

Reproducible because `svelte.config.js` pins `kit.version.name` (the default is
`Date.now()`, which would move every asset hash) and `normalize-bundle.ts`
strips the build timestamp the adapter stamps into the banner.

## Re-record the oracle

From the crate root, with the V8 prebuilts in the environment:

```bash
export RUSTY_V8_ARCHIVE=~/rusty-v8-prebuilt/librusty_v8_ptrcomp_release_aarch64-apple-darwin.a
export RUSTY_V8_SRC_BINDING_PATH=~/rusty-v8-prebuilt/src_binding_ptrcomp_release_aarch64-apple-darwin.rs
cargo run --release --features v8 --example ssr_oracle            # record
cargo run --release --features v8 --example ssr_oracle -- --check # verify
```

Each scenario runs on its own fresh worker, then repeats on the same worker;
`warm_identical` reports whether the second answer was byte-identical.

## Running the bundle

`build/_worker.js` is ESM. Lower it to a classic script with
`openworkers-transform` v0.1.0 (`parse_worker_code`), the same pass every
runtime uses, then create a worker with an `ASSETS` binding whose fetches all
answer 404. The handler is `globalThis.default.fetch(request, env, ctx)`.

## Manifest format

`scenarios.json` is the input: `name`, `exercises`, and a request of `method`,
`path`, `headers` (`name: value` strings, ordered, duplicates allowed) and a
`body` string or null. `oracle.json` repeats each scenario with the recorded
`response`: `status`, `headers` in the order the runtime emitted them, and
`body_file` / `body_bytes` / `body_sha256`. The bodies live in `oracle/`.

To compare a runtime, run the same requests and diff status, header list and
body bytes. 16 of the 17 scenarios are already byte-identical between
`openworkers-runtime-v8` and Bun, so a difference is a runtime bug, not a
SvelteKit non-determinism.

## Known deviation baked into the oracle

`urlencoded-plus` records `+` surviving as a literal `+` in both a query string
and a form body. WHATWG says the urlencoded parser decodes it to a space, and
Bun does; `openworkers-runtime-v8` does not, in `URLSearchParams`, in
`URL.searchParams` and in `Request.formData()`. A runtime that renders
`hello world` there is right and the oracle is wrong.

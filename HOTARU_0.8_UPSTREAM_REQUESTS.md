# hotaru 0.8 — Upstream Re-exports / Bug Reports

Notes collected while migrating the SFX project from `hotaru = 0.7.7` to
`hotaru = 0.8.x` (`hotaru_lib`, `htmstd`).

**Status update (after running against the local `hotaru = 0.8.2` /
`htmstd = 0.8.1` workspace at `~/FDS/hotaru`):** most of the items below
are already addressed in that build. The two remaining items are #6 and
#7, plus a small follow-up (#8) about prelude completeness.

Per-item status is in the table at the bottom.

---

## 1. `hotaru_trans` 0.8.1 — `<**path>` literal check rejects every valid catch-all URL ✅ FIXED in 0.8.2

**Severity:** breaking; blocks all uses of the `<**path>` segment in the
`endpoint!` / `outpoint!` macros (both `trans` and `attr` / `semi-trans`
flavors).

**Where (0.8.1):** `hotaru_trans-0.8.1/src/url/urlexpr.rs:104`

```rust
fn check_url_literal_format(lit: &Literal) -> Result<(), TokenStream> {
    parse_check_url(&lit.to_string())   // <-- includes surrounding quotes
        .map_err(...)
}
```

`proc_macro::Literal::to_string()` returns the *source representation* of
the literal, including the surrounding `"` characters. The new 0.8 URL
parser correctly enforces "`<**path>` must be the only content of its
segment." With the trailing `"` still attached, the final segment becomes
`<**path>"` and the parser returns `AnyPathMixedWithOtherContent`, which
the macro reports as:

```
error: Invalid URL literal format: <**path> must be the only content within
       its segment at index 7
```

The runtime `Server::url(...)` parser receives the actual string value
(without quotes) and would handle `<**path>` correctly — only the
macro-time check was broken.

**Fix in 0.8.2:** `urlexpr.rs` now strips the surrounding `"` (and the
`r"..."` raw form) before calling `parse_check_url`. SFX's
`/static/<**path>` endpoint compiles again with no workaround.

---

## 2. Removed: `htmstd::PrintLog` (no replacement) ✅ RESTORED in htmstd 0.8.1

**Severity:** breaking; affects any project that used the standard
logging middleware.

In `htmstd-0.8.0` the `PrintLog` re-export disappeared and the underlying
`LoggingMiddleware` was commented out of `hotaru_core`.

**Fix in htmstd 0.8.1:** `pub use log::print_log::PrintLog;` is back in
`htmstd::lib`. SFX dropped its hand-rolled `middleware! PrintLog` stub.

---

## 3. Removed: `HttpResCtx::send_request(host, request, safety)` ✅ REPLACED in 0.8.2

**Severity:** breaking; this was *the* one-liner for any code that
needed to talk to another HTTP server from inside a handler.

`hotaru_core-0.7.7/src/http/context.rs:532` provided a static method
that accepted `("http://..." | "https://..." | "host:port", request,
safety)` and handled scheme parsing, default ports, optional explicit
ports, auto-populating the `Host` header, TLS upgrade for `https://`,
framing, and parsing — everything a handler-side caller needs.

0.8.1 removed it without a comparable replacement; the only options were
`Client::request_fn` (requires building a `Client` and registering a
named outpoint), the `outpoint!` macro (same shape), or ~20 lines of
manual `TcpOutbound`/`Http1Channel` plumbing per call site.

**Fix in 0.8.2:** `hotaru_http::send_request(outbound, request, safety)`
ships as a public one-shot helper, re-exported through
`hotaru::http::*`. Caller picks the right `Outbound` (`TcpOutbound` or
`TlsOutbound`) for the URL.

The split (caller chooses the transport, helper does the framing) is the
right separation — but for the common case of "I have an
`http://host[:port]` URL, give me back a response," SFX still wraps it
in a small adapter that:

1. parses scheme/host/port out of the URL,
2. builds the `TcpOutbound`,
3. fills in the `Host` header if absent,
4. calls `send_request(&outbound, ...)`.

Worth considering for upstream: a tiny convenience layer like
`hotaru::http::send_url(url: &str, request, safety)` that does the
scheme dispatch. Optional — the current API is clean, just slightly
verbose for the bulk case.

---

## 4. `HttpStartLine` not re-exported through `hotaru::http::*` ✅ FIXED in 0.8.2

**Severity:** papercut.

In 0.8.1, `hotaru::http::*` re-exported `hotaru_http::meta::*` but not
`hotaru_http::message::start_line`, so building a request via
`HttpStartLine::request_post(...)` required a deep `use
hotaru::hotaru_http::message::start_line::HttpStartLine;`.

**Fix in 0.8.2:** `hotaru::http` adds `pub use
hotaru_http::start_line::*;` (and `hotaru_http::lib.rs` re-exports the
module itself). The deep import is gone.

---

## 5. `TimeoutSetting` not in any prelude ✅ FIXED in 0.8.2

**Severity:** papercut.

`AppBuilder::max_connection_time(...)` takes a `TimeoutSetting`. In
0.8.1 it lived at
`hotaru_core::app::common::operational_config::TimeoutSetting` with no
prelude re-export, so the canonical builder snippet ended up with a
20-character module path inline.

**Fix in 0.8.2:** `hotaru::prelude` exports `TimeoutSetting` directly
(via `pub use crate::{... TimeoutSetting};` in `lib.rs` and `pub use
crate::{... TimeoutSetting};` in `prelude.rs`). SFX now writes
`max_connection_time(TimeoutSetting::Seconds(10))`.

---

## 6. `App::binding_address: String` renamed to `Server::binding` with no compat alias ❌ STILL OPEN

**Severity:** papercut; trivial fix in user code, but still an
unannounced rename.

0.7:

```rust
pub struct App { pub binding_address: String, ... }
```

0.8:

```rust
pub struct Server<TS: TransportSpec> {
    pub binding: <TS::Inbound as Inbound>::BindTarget,  // String for TCP
    ...
}
```

Code that read `APP.binding_address` silently breaks. The 0.8 changelog
mentions `Server` replacing `App` but doesn't list field renames.

**Suggested fix:** either keep a transitional `pub fn
binding_address(&self) -> &str` accessor, or add an explicit "field
renames" subsection to the 0.8 changelog. The current `Server::binding`
shape is fine on its own — the issue is purely the silent rename.

---

## 7. Middleware return shape changed from `C` to `Result<C, C::Error>` ❌ DOCS ONLY

**Severity:** breaking; not a re-export issue but worth a changelog
entry.

In 0.7, a `middleware!` body returned the context directly:

```rust
middleware! {
    pub RedirectGuest <HTTP> {
        if user.is_guest() {
            req.response = redirect_response("/user/login");
            return req
        }
        next(req).await
    }
}
```

In 0.8 the generated `handle` returns `Result<C, <C as
RequestContext>::Error>`, and so does `next(req).await`. Every early
`return req` becomes `return Ok(req)`. The macro error is clear
(`expected ... but found HttpContext`), but the readme example only
shows the happy path; an early-return example would have caught this in
one pass instead of one-per-middleware.

**Suggested fix:** add a one-line note to the `middleware!` section of
the `hotaru` readme:

> Early returns wrap in `Ok(...)`: `return Ok(req)` (not `return req`).

Optionally a short "migrating from 0.7" section pointing this out.

---

## 8. `TcpOutbound` / `TlsOutbound` not in `hotaru::prelude` ❌ NEW (0.8.2)

**Severity:** papercut, surfaces once #3 is in use.

With the new `hotaru_http::send_request(&outbound, ...)` helper, the
caller writes `TcpOutbound::build(...).await?`. `TcpOutbound` is
re-exported through `hotaru::*` (from `hotaru_core::connection::{...,
TcpOutbound, ...}`) but **not** through `hotaru::prelude`. The current
prelude has `TcpTransport` (the type-spec marker) but not the matching
`TcpOutbound` / `TcpInbound` instances, which is what users actually
construct.

For the SFX adapter around `send_request`, the imports look like:

```rust
use hotaru::prelude::*;
use hotaru::http::*;
use hotaru::TcpOutbound;   // <-- the extra line
```

**Suggested fix:** add `TcpInbound`, `TcpOutbound` (and the TLS
counterparts under `#[cfg(feature = "https")]`) to
`hotaru::prelude::*`. They're the natural pair to `TcpTransport` and
`Server` / `Client`.

---

## Summary table

| # | Crate | Severity | Status in 0.8.2 |
|---|---|---|---|
| 1 | `hotaru_trans` | Breaking (blocked `<**path>`) | ✅ FIXED — quotes stripped |
| 2 | `htmstd` | Breaking (silent removal) | ✅ RESTORED in htmstd 0.8.1 |
| 3 | `hotaru_http` | Breaking (no one-shot client) | ✅ REPLACED — `send_request(outbound, req, safety)` |
| 4 | `hotaru` / `hotaru_http` | Papercut | ✅ FIXED — `http::*` re-exports `start_line::*` |
| 5 | `hotaru` | Papercut | ✅ FIXED — `TimeoutSetting` in prelude |
| 6 | `hotaru` | Papercut | ❌ Open — changelog / `binding_address` shim |
| 7 | `hotaru` / docs | Breaking, well-signaled | ❌ Open — one-line readme note |
| 8 | `hotaru` | Papercut | ❌ New — add `TcpOutbound` to prelude |

Items **1**, **2**, and **3** were the ones that turned the original
migration from "edit a few imports" into "rewrite handler plumbing."
0.8.2 closes all three, plus the two papercuts in **4** and **5**.
What's left (**6**, **7**, **8**) is a small documentation pass plus
one prelude line.

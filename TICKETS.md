# SFX internal tickets

Open follow-ups for the SFX CLI / scaffolding. Surfaced by the 0.1.2
publish-readiness pass; deferred since none are regressions and none block
the 0.1.2 release.

---

## Ticket 1 — `sfx init` hardcodes the project name to `my_project`

### Where

`src/main.rs` (around line 48 per earlier inspection). The `init`
subcommand writes `name = "my_project"` into `Cargo.toml` regardless of
the directory it runs in.

### Repro

```bash
mkdir foo && cd foo
sfx init
grep '^name' Cargo.toml
# name = "my_project"
```

### Why it's a problem

Users expect `init` to mirror Cargo's behavior — derive the package name
from the containing directory. Today every `sfx init` produces an
identically-named project until the user manually edits `Cargo.toml`.

### Suggested fix

In the `init` handler, default the crate name to
`env::current_dir()?.file_name()`, sanitized the same way `sfx new` already
sanitizes (`-` → `_`, lowercase). Allow an optional `--name <name>`
override.

### Severity

Cosmetic / UX. Not a blocker.

---

## Ticket 2 — `sfx new` accepts non-Rust-identifier names and emits a broken `Cargo.toml`

### Where

`src/main.rs`, the `new` subcommand argument handling. There is no upfront
validation on the `<program_name>` argument.

### Repro

```bash
sfx new 123app /tmp/x
cd /tmp/x/123app && cargo check
# error: invalid character '1' in package name: '123app', the name cannot start with a digit
```

`my-app` is handled correctly (normalized to `my_app`), but leading digits
and other invalid identifier shapes slip through.

### Why it's a problem

The CLI exits 0 and prints "Project created" but the resulting project
won't compile. The user discovers the problem one step later, with a
confusing error pointing at the scaffolded `Cargo.toml`.

### Suggested fix

Validate `<program_name>` before doing any filesystem work:

- must start with `[A-Za-z_]`
- must contain only `[A-Za-z0-9_-]`
- must not be a Rust keyword

Reject early with a clear error. Same rule applies to the implicit name
that `sfx init` will pick up after Ticket 1.

### Severity

Cosmetic / UX. Not a blocker.

---

## Ticket 3 — `sfx new` silently no-ops when the target dir already contains a project

### Where

`src/main.rs`, the `new` subcommand's copy step.

### Repro

```bash
sfx new demo_app /tmp/x      # first time — creates /tmp/x/demo_app
echo "marker" >> /tmp/x/demo_app/src/main.rs
sfx new demo_app /tmp/x      # second time — exits 0, prints "Project created"
grep marker /tmp/x/demo_app/src/main.rs
# marker  ← still there. Files were NOT overwritten.
```

### Why it's a problem

The success message is a lie when the target dir already exists. There's
no `--force` flag on `new` (unlike `init`, which has `-f/--force`), and no
warning. Users who run `sfx new` twice — either by accident, or thinking
they're regenerating after a config update — silently get the old project
with the misleading "Project created" message.

### Suggested fix

Pick one:

- (a) Default behavior: refuse with an error if the target dir exists and
  is non-empty. Add `--force` to allow overwrite. Symmetric with `init`.
- (b) Always overwrite, but warn loudly.

(a) is safer.

### Severity

Cosmetic / UX, but with a real footgun: a user could think they've
re-scaffolded when they haven't. Not a blocker for 0.1.2, but the
"silently misleading success message" character of it makes it the
highest-priority item among these three for a follow-up.

# SFX Development Notes

## Project Structure

```
sfx/
├── Cargo.toml          # Workspace + library + binaries
├── src/
│   ├── lib.rs          # Library entry (exports APP, prelude, modules)
│   ├── main.rs         # Binary: runs APP directly
│   └── main-cli.rs     # Binary: sfx-cli scaffolding tool
├── default/            # Template files for sfx-cli new
│   ├── Cargo.toml.template
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs
│   ├── templates/      # HTML templates
│   └── programfiles/   # Config files
└── test-app/           # Workspace member for testing
```

## Binary

| Binary | Command | Purpose |
|--------|---------|---------|
| `sfx` | `cargo run --bin sfx` | Project scaffolding CLI |

## CLI Usage

```bash
# Create new project
sfx new my_app_name /path/to/folder

# Initialize in current directory
sfx init
```

## Template Placeholders

In `default/` template files, use:
- `{{crate_name}}` - Replaced with the crate name (underscores, valid Rust identifier)

## Framework Migration: Starberry → Hotaru

| Starberry | Hotaru |
|-----------|--------|
| `starberry::prelude::*` | `hotaru::prelude::*` + `hotaru::http::*` |
| `sbmstd` | `htmstd` |
| `#[url(APP.lit_url("/path"))]` | `endpoint! { APP.url("/path"), pub fn <HTTP> { } }` |
| `#[middleware]` | `middleware! { pub Name <HTTP> { } }` |
| `req.get_url_args("x")` | `req.query("x")` |
| `req.get_arg("x")` | `req.param("x")` |
| `req.meta().method()` | `req.method()` |
| `ProtocolBuilder::<HttpReqCtx>::new()` | `ProtocolBuilder::new(HTTP::server(HttpSafety::default()))` |

## Generated Project Structure

When running `sfx-cli new my_app /path`:

```
my_app/
├── Cargo.toml          # Depends on sfx = "0.1.0"
├── src/
│   ├── main.rs         # use my_app::APP; APP.clone().run().await;
│   └── lib.rs          # Custom endpoints using sfx::APP
├── templates/          # HTML templates (copied from default/)
└── programfiles/       # Config files (copied from default/)
```

## Future Work: Modular APP Builder

Current: APP is a pre-built singleton with all routes.

Planned: Allow opt-in modules:
```rust
use sfx::{AppBuilder, auth_routes, admin_routes};

fn main() {
    AppBuilder::new()
        .with_auth()           // opt-in
        .with_admin()          // opt-in
        .add_routes(my_routes) // custom
        .run();
}
```

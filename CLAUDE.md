# Claude Code Guidelines for uv

This document provides guidance for Claude Code when working on the uv codebase.

## Project Overview

uv is an extremely fast Python package and project manager written in Rust. The codebase consists of
67+ crates in a Cargo workspace.

## Philosophy

### Performance is Non-Negotiable

uv is 10-100x faster than pip. Every code path should be optimized. Avoid unnecessary allocations,
prefer streaming over buffering, and use efficient data structures. When in doubt, benchmark.

### One Tool to Replace Many

uv replaces pip, pip-tools, pipx, poetry, pyenv, twine, virtualenv, and more. New features should
integrate cohesively rather than feeling bolted-on. Maintain consistency in CLI design, error
messages, and behavior.

### Correctness Over Convenience

Don't work around problems — fix them properly. If something is broken, understand why. If an API is
misused, correct the usage. If a dependency has issues, address them upstream or document the
limitation clearly.

### Safety Through Types and Linting

The codebase uses comprehensive linting (clippy pedantic), type-safe wrappers (like `uv-fs` for file
operations), and error types with proper source chains. These aren't optional — they catch bugs
before users do.

### Comprehensive Testing

Use snapshot testing for CLI output, property-based testing where appropriate, and integration tests
that cover real-world scenarios. Tests should be deterministic — filter timestamps, paths, and
versions in snapshots.

### User-Facing Quality

Error messages should be actionable. Warnings should be meaningful. Output should be readable
without colors (accessibility). Follow the style guide in STYLE.md for all user-facing text.

## Critical Rules

### File System Operations

**Never use stdlib fs operations directly.** All file operations must go through `uv-fs`:

```rust
// WRONG
use std::fs::File;
std::fs::create_dir_all(path)?;

// CORRECT
use uv_fs::Simplified;
fs_err::create_dir_all(path)?;
```

The `clippy.toml` enforces this - builds will fail if you use disallowed fs types/methods.

### Output and Printing

**Never use `println!()` or `eprintln!()` directly:**

```rust
// WRONG
println!("Output");
eprintln!("Error");

// CORRECT
use anstream::eprintln;
eprintln!("Error message");

// For user warnings
use uv_warnings::warn_user;
warn_user!("Deprecation warning");
```

The main binary has `#![deny(clippy::print_stdout, clippy::print_stderr)]`.

### Error Handling

Use `thiserror` for error types with proper source chaining:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid configuration")]
    Config(#[source] ConfigError),
}
```

Implement `Display` for all custom types that users might see.

## Code Style

### Import Organization

```rust
// 1. Standard library
use std::path::PathBuf;
use std::str::FromStr;

// 2. Workspace crates
use uv_normalize::PackageName;
use uv_fs::Simplified;

// 3. External crates
use serde::{Deserialize, Serialize};
use thiserror::Error;
```

### Derive Macros

Follow the standard derive pattern:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct MyType {
    // ...
}
```

### Visibility

- Use `pub(crate)` for internal items, not `pub`
- The linter warns on `unreachable_pub`

### Naming

- Package names: lowercase with dashes (PEP compliant normalization)
- Enum variants: PascalCase
- Constants: SCREAMING_SNAKE_CASE
- Functions/methods: snake_case

## Testing

### Snapshot Testing

Use `uv_snapshot!` macro with filters for stable output:

```rust
#[test]
fn test_command() {
    let context = TestContext::new("3.12");

    uv_snapshot!(context.filters(), context.command()
        .arg("--flag"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Expected output here
    "###
    );
}
```

### Test Context

Use `TestContext` for isolated test environments:

```rust
let context = TestContext::new("3.12");
// or for multiple Python versions:
let context = TestContext::new_with_versions(&["3.11", "3.12"]);
```

### Running Tests

```bash
cargo test                           # Run all tests
cargo test -p uv-gui                 # Run specific crate tests
cargo insta review                   # Review snapshot changes
```

## Workspace Structure

### Cargo.toml Pattern

```toml
[package]
name = "uv-my-crate"
version = "0.0.7"
edition = { workspace = true }
rust-version = { workspace = true }
license = { workspace = true }

[lib]
doctest = false

[lints]
workspace = true

[dependencies]
uv-fs = { workspace = true }
serde = { workspace = true, features = ["derive"] }

[dev-dependencies]
insta = { workspace = true }
```

Always use `{ workspace = true }` for shared dependencies.

### Crate Categories

- **Type crates**: `uv-normalize`, `uv-pep440`, `uv-pep508`
- **Core functionality**: `uv-resolver`, `uv-installer`, `uv-cache`
- **CLI/Config**: `uv-cli`, `uv-configuration`, `uv-settings`
- **Utilities**: `uv-fs`, `uv-shell`, `uv-extract`

## CLI Commands

Commands are async functions returning `Result<ExitStatus>`:

```rust
pub(crate) async fn my_command(
    config: Config,
    printer: Printer,
) -> Result<ExitStatus> {
    // Implementation
    Ok(ExitStatus::Success)
}
```

## Linting

Key clippy rules enabled:

- `pedantic` (with many allows)
- `unsafe_code = "warn"`
- `unreachable_pub = "warn"`
- `print_stdout = "warn"`, `print_stderr = "warn"`
- `use_self = "warn"`

Run before committing:

```bash
cargo fmt
cargo clippy --all-targets
cargo dev generate-all  # If CLI/settings changed
```

## User-Facing Messages

From STYLE.md:

- Use backticks for commands, code, packages, file paths
- Wrap at 100 characters in markdown
- Use em-dashes with spaces: "hello — world"
- Hyphenate compound words: "platform-specific"
- Colors: green=success, red=error, yellow=warning, cyan=hints/paths

## Pre-commit Checklist

1. `cargo fmt` - Format code
2. `cargo clippy --all-targets` - Lint check
3. `cargo test` - Run tests
4. `cargo dev generate-all` - If CLI/settings changed
5. `cargo insta review` - If snapshots changed

## What NOT to Do

1. Don't start features without prior discussion
2. Don't use direct stdlib fs operations
3. Don't use `println!()` / `eprintln!()`
4. Don't ignore clippy warnings without explicit `#[allow()]`
5. Don't create circular crate dependencies
6. Don't hardcode paths
7. Don't use `.unwrap()` without considering errors

## Contribution Guidelines

- Bug fixes are the best contributions
- Look for `good first issue` and `help wanted` labels
- Don't work on unlabeled issues without checking first
- New feature PRs without prior discussion will be closed
- Always make sure the PR description is up to date if working on a PR.  Ensure it reflects the current state of the PR, do not use phrases like "Recent Changes" or such based on specific commits in the PR.  Also make sure you don't reference screenshots or something in the description.
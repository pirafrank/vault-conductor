# Agent Guidelines for this repository

This document provides guidelines for AI agents working on the vault-conductor codebase.

## Build System & Commands

### Build Commands

```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Build with all features
cargo build --all-features

# Clean build artifacts
cargo clean
```

### Test Commands

```bash
# Run all tests
cargo test

# Run all tests with output
cargo test -- --nocapture

# Run a specific test
cargo test <test_name>

# Run tests in a specific module
cargo test <module_name>::

# Run tests with verbose output
cargo test -vv

# Run tests in release mode
cargo test --release
```

### Lint & Format Commands

```bash
# Run clippy linter
cargo clippy

# Run clippy with warnings as errors
cargo clippy -- --no-deps -D warnings

# Auto-fix clippy warnings
cargo fix --allow-dirty

# Format code
cargo fmt

# Check formatting without applying changes
cargo fmt -- --check

# Run both formatting and linting
cargo fmt && cargo clippy -- --no-deps -D warnings
```

### CI Commands (via just)

```bash
# Install just if not available: cargo install just

# Run full CI checks (clean, format, lint, build, test)
just ci

# Run pre-commit checks (format + lint)
just pre-commit

# Run pre-push checks (build + test)
just pre-push

# Run only formatting check
just fmt-check

# Run only linting
just lint

# Run only tests
just test
```

## Code Style Guidelines

### Imports

```rust
// Standard library imports first
use std::path::PathBuf;
use std::fs;

// External crate imports next
use anyhow::{Context, Result};
use serde::Deserialize;
use log::{debug, info, error};

// Local module imports last
use crate::bitwarden::client_wrapper::start_agent_foreground;
use crate::logging::setup_logging;
use crate::process_manager::{restart_agent, start_agent_background, stop_agent};
```

**Rules:**
- Group imports: std, external crates, local modules
- Use absolute paths for local imports (`crate::module::submodule`)
- Use curly braces for multiple imports from same module
- Keep imports sorted within each group

### Formatting

- **Line length**: 100 characters max (configured in rustfmt.toml)
- **Indentation**: 4 spaces (no tabs)
- **Newlines**: Unix style (`\n`)
- **Braces**: Always use braces for control structures
- **Spacing**: Use spaces around operators, after commas
- **Reordering**: Enable `reorder_imports` and `reorder_modules` in rustfmt

### Naming Conventions

```rust
// Types (PascalCase)
struct Config {}
struct SshAgent {}
enum Command {}

// Functions and methods (snake_case)
fn setup_logging() {}
fn get_config_path() -> Result<PathBuf> {}

// Variables (snake_case)
let config_path = PathBuf::new();
let bws_access_token = "token".to_string();

// Constants (SCREAMING_SNAKE_CASE)
const CONFIG_FILE: &str = ".config/vault-conductor/config.yaml";

// Traits (PascalCase)
trait AgentTrait {}

// Modules (snake_case)
mod bitwarden;
mod config;
mod logging;
```

**Disallowed names** (from clippy.toml):
- `foo`, `baz`, `pippo`, `pluto`

### Error Handling

```rust
use anyhow::{Context, Result};

// Preferred error handling pattern
fn load_config() -> Result<Config> {
    let config_content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    let config: Config = serde_yaml::from_str(&config_content)
        .context("Failed to parse config file as YAML")?;

    Ok(config)
}

// Use anyhow for application errors
// Use thiserror for library errors (not applicable here)
// Always provide context for errors
// Use ? operator for error propagation
```

### Type Usage

```rust
// Prefer strong typing
fn process_data(data: &[u8]) -> Result<Vec<String>> {}

// Use Result<T, E> for fallible operations
// Use Option<T> for optional values
// Use anyhow::Result for application code
// Use explicit error types for library code

// Type aliases for complex types
pub type AgentResult<T> = Result<T, AgentError>;
```

### Documentation

```rust
/// A Rust CLI boilerplate application
#[derive(Parser)]
struct Cli {}

/// Available subcommands
#[derive(Subcommand)]
enum Commands {}

/// Start the SSH Agent in the background
#[command(name = "start-agent")]
Start(StartArgs),

// Document public APIs
// Use /// for documentation comments
// Use // for implementation comments
// Keep documentation up to date
```

### Async Code

```rust
// Use tokio for async runtime
#[tokio::main]
async fn main() -> Result<()> {}

// Proper async function signature
async fn start_agent_foreground() -> Result<()> {}

// Use .await for async calls
let result = some_async_function().await?;

// Avoid blocking calls in async context
// Use tokio::spawn_blocking for CPU-intensive tasks
```

### Logging

```rust
use log::{debug, info, error, warn};

// Log levels:
// - error: Critical failures
// - warn: Potential issues
// - info: Important runtime information
// - debug: Detailed debugging information
// - trace: Very verbose debugging

// Initialize logging
env_logger::init();

// Log messages
info!("Starting application");
debug!("Processing request: {}", request_id);
error!("Failed to connect: {}", error);
```

### Testing

```rust
// Test structure (if tests exist)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        // Test implementation
    }

    #[test]
    fn test_error_handling() {
        // Test implementation  
    }
}

// Test naming: test_<function>_<scenario>
// Use descriptive test names
// Keep tests focused on single functionality
// Use setup/teardown where appropriate
```

## Project Structure

```
src/
├── main.rs              # Main application entry point
├── lib.rs              # Library code (if exists)
├── bitwarden/          # Bitwarden integration module
│   ├── mod.rs          # Module declaration
│   ├── agent.rs        # Agent implementation
│   └── client_wrapper.rs # Client wrapper
├── config.rs           # Configuration handling
├── logging.rs          # Logging setup
└── process_manager.rs  # Process management

tests/                 # Integration tests (if exists)
```

## Git & Workflow

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new feature
fix: fix bug in module
refactor: improve code structure
doc: update documentation
perf: improve performance
test: add missing tests
chore: update dependencies
```

### Branching

- `main`: Production branch
- `feat/*`: Feature branches
- `fix/*`: Bug fix branches
- `refactor/*`: Refactoring branches

### Pull Requests

- Reference related issues
- Include screenshots for UI changes
- Update documentation
- Add tests for new functionality
- Keep PRs focused and small

## Toolchain

- **Rust version**: 1.91 (from rust-toolchain.toml)
- **Edition**: 2021
- **Components**: rustfmt, clippy, llvm-tools-preview
- **Targets**: Linux (gnu/musl), macOS, FreeBSD

## Dependencies

### Key Dependencies

- `clap`: Command line parsing
- `anyhow`: Error handling
- `tokio`: Async runtime
- `log`/`env_logger`: Logging
- `serde`/`serde_yaml`: Serialization
- `bitwarden`: Bitwarden SDK
- `ssh-agent-lib`: SSH agent library

### Dependency Management

```bash
# Update dependencies
cargo update

# Check for outdated dependencies
cargo outdated --root-deps-only

# Check for security vulnerabilities
cargo audit

# Check for license issues
cargo deny check licenses
```

## Best Practices

1. **Error Handling**: Always use proper error handling with context
2. **Logging**: Use appropriate log levels
3. **Testing**: Write tests for new functionality
4. **Documentation**: Keep docs up to date
5. **Formatting**: Run `cargo fmt` before committing
6. **Linting**: Run `cargo clippy` before committing
7. **CI**: Ensure all CI checks pass
8. **Security**: Follow security best practices
9. **Performance**: Consider performance implications
10. **Compatibility**: Maintain backward compatibility whenever possible

## Agent-Specific Instructions

When working on this codebase:

1. **Always run** `just pre-commit` AND `just pre-push` before making changes
2. **Follow** the existing code style and patterns. Ask the user consent before making breaking changes. Ask the user the consent ONLY if a breaking change is REALLY needed.
3. **Use** the existing module structure. DO NOT change module structure unless you are told so.
4. **Write** tests for new functionality. Write less tests but make them sensible in the codebase.
5. **Update** documentation when making changes
6. **Keep** changes focused and small
7. **Run** linting and formatting after making changes. Auto-fix errors and warning if any.
8. **Check** for security vulnerabilities
9. **NEVER** ever commit. Let the human developer do it.
10. **Be** consistent with existing code

## Common Issues & Solutions

### Build Issues

```bash
# Missing toolchain component
rustup component add rustfmt clippy

# Missing target
rustup target add x86_64-unknown-linux-musl

# Clean and rebuild
cargo clean && cargo build
```

### Test Issues

```bash
# Test failures due to environment
# Set required environment variables

# Test failures due to missing dependencies
# Install system dependencies as needed
```

### Formatting Issues

```bash
# Auto-format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting Issues

```bash
# Fix clippy warnings
cargo clippy --fix --allow-dirty

# Run clippy with warnings as errors
cargo clippy -- --no-deps -D warnings
```

This guide should help AI agents work effectively on the vault-conductor codebase while maintaining consistency and quality.

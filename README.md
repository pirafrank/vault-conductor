[![Licenses](https://github.com/pirafrank/poof/actions/workflows/licenses.yml/badge.svg)](https://github.com/pirafrank/poof/actions/workflows/licenses.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

# Rust CLI boilerplate

A template for cross-platform binary CLI projects written in Rust.

## Table of Contents

- [Features](#features)
  - [CLI Framework](#cli-framework)
  - [Build System](#build-system)
  - [Code Quality & Standards](#code-quality--standards)
  - [Git Workflow](#git-workflow)
  - [Changelog & Release Management](#changelog--release-management)
  - [Development Workflow](#development-workflow-just)
  - [Project Structure](#project-structure)
- [Requirements](#requirements)
  - [Core Requirements](#core-requirements)
  - [Optional Requirements](#optional-requirements)
  - [Additional Tools (Optional, yet recommended)](#additional-tools-optional-yet-recommended)
- [Getting Started](#getting-started)
  - [Run examples](#run-examples)
- [Development tasks](#development-tasks)
- [License](#license)


## Features

### CLI Framework

- **Command-line argument parsing** with [clap](https://github.com/clap-rs/clap)
- **Subcommand architecture** ready to use (includes example `greet` and `echo` commands)
- **Flexible verbosity control** using `clap-verbosity-flag` (supports `-v`, `-vv`, `-vvv`, `-vvvv`)
- **Structured logging** with `log` and `env_logger` crates
- **Error handling** with `anyhow` for ergonomic error propagation

### Build System

- **Custom build script** (`build.rs`) that:
  - Detects and embeds glibc version for GNU targets
  - Identifies C library linkage (glibc, musl, libSystem, libc)
  - Detects static vs dynamic linking configuration
  - Embeds Git commit hash at build time
  - Embeds build timestamp in UTC
- **Multi-platform support** configured via `rust-toolchain.toml`:
  - Linux: x86_64 and aarch64 (GNU and musl)
  - macOS: x86_64 and aarch64 (Darwin)
  - FreeBSD: x86_64
- **Build matrix** configuration (`matrix.jsonc`) for cross-platform CI builds

### Code Quality & Standards

- **Rustfmt configuration** (`rustfmt.toml`) with:
  - Edition 2021 style
  - 100 character line width
  - Unix line endings
  - Auto-reordering of imports and modules
- **Clippy configuration** (`clippy.toml`) with custom linting rules
- **Code coverage** setup with codecov integration (`codecov.yml`)
- **Dependency management** and security:
  - `cargo-deny` configuration (`deny.toml`) for:
    - License compliance checking (MIT, Apache-2.0, BSD-3-Clause, etc.)
    - Security vulnerability scanning
    - Dependency source validation
    - Duplicate dependency detection

### Git Workflow

- **Git hooks** ready to use (in `hooks/` directory):
  - **Pre-commit hook**: Runs formatting checks and linting on staged Rust files
  - **Pre-push hook**: Runs full build and test suite before pushing
  - **Pre-push tag hook**: Validates version consistency across Cargo.toml, Cargo.lock, and CHANGELOG.md
  - **Commit-msg hook**: Enforces [Conventional Commits](https://www.conventionalcommits.org/) format
- **Auto-formatting** of staged files via `fmt_staged.sh` helper script

### Changelog & Release Management

- **Automated changelog generation** with [git-cliff](https://github.com/orhun/git-cliff)
- **Conventional commits** parser configuration (`cliff.toml`) with custom groups:
  - 🚀 Features
  - 🐛 Bug Fixes
  - 🚜 Refactor
  - 📚 Documentation
  - ⚡ Performance
  - 🧪 Testing
  - 🔧 Setup & Quality
  - 🛡️ Security
  - ◀️ Revert
  - ⚙️ Miscellaneous Tasks

### Development Workflow

Integrated task runner via [just](https://github.com/casey/just) with comprehensive recipes:

- **Building**: `just build`, `just release`
- **Testing**: `just test`, `just test-integration`
- **Code quality**: `just fmt`, `just lint`, `just fix`, `just better`
- **Pre-commit/push**: `just pre-commit`, `just pre-push`
- **Documentation**: `just docs`
- **Dependencies**: `just deps`, `just update-deps`, `just outdated-deps`
- **Security**: `just audit`, `just licenses`, `just compliance`
- **Coverage**: `just coverage`
- **Release**: Run `just prepare-release VERSION`, then `just make-release VERSION`
- **Cleaning**: `just clean`, `just clean-all`
- **CI**: `just ci` (runs full CI pipeline locally)

### Project Structure

- **Clean package manifest** with comprehensive metadata (authors, repository, documentation, license)
- **Minimum Rust version**: 1.85.0 (specified in `Cargo.toml`). You can change this based on your dependencies.
- **Stable toolchain** with required components: `rustfmt`, `clippy`, `llvm-tools-preview`
- **Proper exclusions** for version control artifacts and IDE files

## Requirements

### Core Requirements

- **Rust** 1.85.0 or newer (toolchain will be installed automatically via `rust-toolchain.toml`)
  - Components: `rustfmt`, `clippy`, `llvm-tools-preview` (which are installed automatically)
- **git** (required for version control and build script that embeds commit hash)
- **just** - Command runner for task automation ([installation guide](https://github.com/casey/just#installation))

### Optional Requirements

These cargo plugins enhance the development workflow but are not strictly required to build and run the project. Install them with `just install-cargo-plugins`:

- **cargo-binstall** - Fast cargo plugin installer
- **cargo-edit** - Manage Cargo.toml dependencies from CLI
- **cargo-deny** - Check licenses, security vulnerabilities, and dependencies
- **cargo-semver-checks** - Lint breaking changes in public API
- **cargo-outdated** - Check for outdated dependencies
- **cargo-audit** - Security vulnerability scanner
- **cargo-llvm-cov** - Code coverage tool
- **git-cliff** - Changelog generator for conventional commits

### Additional Tools (Optional, yet recommended)

- **glow** - Markdown renderer for viewing changelog (used in `just changelog` recipe)
- **vim/nvim** or any text editor (for `just prepare-release` recipe). You can change `nvim` to the editor of your choice in `justfile` recipe.

## Getting Started

1. Click [here](https://github.com/new?template_name=rust-cli-boilerplate) use the template

2. Clone the newly created repo locally or in a Codespace

3. Setup the enviroment and make the first build:

```bash
# Install git hooks
just install-hooks

# Install required cargo plugins
just install-cargo-plugins

# Make the first build
just build
```

All done!

### Run examples

**Run the example commands**:

```bash
cargo run -- greet "Rust Developer"
cargo run -- echo "Hello, World!"
```

**Run with verbose logging**

```bash
cargo run -- -vvv greet World
```

Now write some code and make it yours!

## Development tasks

Use the following during development to make life easier:

- `just build` to build the project
- `just better` to format and lint your code
- `just ci` to run the full CI pipeline locally
- `just compliance` to check for security issues and license compliance

You can always run `just` to see all available commands.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE.md) file for details.


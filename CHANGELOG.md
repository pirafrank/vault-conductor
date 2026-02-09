# Changelog

All notable changes to the project will be documented in this file.

## [0.3.0] - 2026-02-09

### 🚀 Features

- Additional installation methods
- [**breaking**] Removed `-agent` string from commands
- Added support for multiple users
- Support receiving signals for graceful shutdown
- [**breaking**] Removed `restart` command, added `--config` option to optionally load config from file
- Show BWS secret name as the comment of SSH key
- New `logs` command to show logs in the terminal
- [**breaking**] Support multiple SSH keys (changed config)
- Do not fail for missing keys and provide proper output to the user
- Support for custom server endpoint and self-hosted instances

### 🐛 Bug Fixes

- Override using env var

### 🚜 Refactor

- Better process clean, moved file functions

### 🔧 Setup & Quality

- Group dependabot PRs
- Crates.io release pipeline

### ⚙️ Miscellaneous Tasks

- Updated Bitwarden Rust SDK to [v2.0.0](https://github.com/bitwarden/sdk-sm/releases/tag/rust-v2.0.0)
- Tell cursor to ignore secrets
- Better error logs + add vscode debug settings
- Update readme and service install instructions

## [0.2.1] - 2026-01-06

### 🚀 Features

- First implementation
- Load secret tokens from config file instead of env var
- Better user logs for bitwarden sdk failures
- Fallback to env vars if config file is not found
- Log to file
- Run in bg, added stop-agent and restart-agent commands

### 🐛 Bug Fixes

- It builds!
- It works for SSH!
- It works for git commit signing
- Bg mode
- Set 0600 permissions for socket and pid file

### 🚜 Refactor

- Variable rename
- Logs
- Kept run in fg mode

### 🔧 Setup & Quality

- Removed publish as crate due to bitwarden-sdk build from source
- Fix release pipeline after file rename

### ⚙️ Miscellaneous Tasks

- License fix
- Lint code
- Deleted leftovers


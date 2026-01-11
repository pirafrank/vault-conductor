# Documentation

Files in this folder document how the codebase is structured and how the code works.

## Architecture & Design

- [ARCHITECTURE](ARCHITECTURE.md) - Architecture overview with visual diagrams covering module structure, SSH agent workflow, process lifecycle, command flow, and more.

## Implementation Details

- [CONFIG_LOAD](CONFIG_LOAD.md) - Configuration loading flow and fallback mechanisms
- [DATA_STRUCTURES](DATA_STRUCTURES.md) - Key data structures, classes, traits, and their relationships
- [ERROR_HANDLING](ERROR_HANDLING.md) - Error handling strategy using `anyhow` with context propagation
- [FILE_SYSTEM](FILE_SYSTEM.md) - Runtime file system layout for configuration, logs, PID files, and Unix sockets

## Deployment

- [SERVICE](SERVICE.md) - How to install and manage `vault-conductor` as a systemd service in userspace

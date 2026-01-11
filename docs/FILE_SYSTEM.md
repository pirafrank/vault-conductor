# File System Layout

File system structure used by the application at runtime.

```txt
$HOME
│
├── .config/vault-conductor/
│   └── config.yaml                # Configuration
│
├── Library/Logs/vault-conductor/
│   └── vault-conductor.log        # Logs (macOS)
│
└── .local/state/vault-conductor/logs/
    └── vault-conductor.log        # Logs (Linux)

/tmp/
│
├── vc-USERNAME-ssh-agent.pid      # PID info
└── vc-USERNAME-ssh-agent.sock     # Agent socket
```

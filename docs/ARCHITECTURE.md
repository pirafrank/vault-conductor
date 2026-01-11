# Architecture Overview

This document provides a visual representation of the vault-conductor codebase and architecture.

The architecture follows these principles:

- **Separation of Concerns**: Each module has a single, well-defined responsibility
- **Trait-based Design**: `SecretFetcher` trait allows for testability and flexibility
- **Error Handling**: Comprehensive error handling with context using `anyhow`
- **Async/Await**: Tokio-based async runtime for efficient I/O
- **Security**: Proper file permissions, signal handling, and graceful shutdown
- **Caching**: Lazy loading and caching of secrets to minimize API calls

## Table of Contents

- [Module Structure](#module-structure)
- [SSH Agent Architecture](#ssh-agent-architecture)
- [Process Lifecycle](#process-lifecycle)
- [Command Flow](#command-flow)
- [Concurrency Model](#concurrency-model)

## Module Structure

The codebase is organized into several key components:

1. **CLI Layer** (`main.rs`): Entry point that parses commands and coordinates between modules
2. **Configuration** (`config.rs`): Handles loading configuration from files or environment variables
3. **Process Management** (`process_manager.rs`, `file_manager.rs`): Manages daemon lifecycle, PID files, and socket files
4. **Bitwarden Integration** (`bitwarden/`): Implements SSH agent protocol backed by Bitwarden Secrets Manager
5. **Logging** (`logging.rs`): Platform-specific logging configuration

```mermaid
graph TB
    subgraph "Vault Conductor Application"
        Main[main.rs<br/>CLI Entry Point]

        subgraph "Configuration"
            Config[config.rs<br/>Config Loading]
            ConfigFile[config.yaml<br/>User Configuration]
        end

        subgraph "Process Management"
            ProcessMgr[process_manager.rs<br/>Agent Lifecycle]
            FileMgr[file_manager.rs<br/>PID/Socket Files]
        end

        subgraph "Bitwarden Integration"
            BWMod[bitwarden/mod.rs<br/>Module Declaration]
            Agent[bitwarden/agent.rs<br/>SSH Agent Logic]
            ClientWrap[bitwarden/client_wrapper.rs<br/>SDK Integration]
        end

        subgraph "Logging"
            Logger[logging.rs<br/>Log Configuration]
            LogFile[vault-conductor.log<br/>Log Output]
        end

        Main --> Config
        Main --> ProcessMgr
        Main --> Logger
        Main --> ClientWrap

        ProcessMgr --> FileMgr
        ProcessMgr --> Logger

        ClientWrap --> Agent
        ClientWrap --> Config
        ClientWrap --> FileMgr

        Config -.reads.-> ConfigFile
        Logger -.writes.-> LogFile
        FileMgr -.manages.-> PIDFile[PID File<br/>/tmp/vc-USERNAME-ssh-agent.pid]
        FileMgr -.manages.-> Socket[Unix Socket<br/>/tmp/vc-USERNAME-ssh-agent.sock]
    end

    subgraph "External Dependencies"
        Clap[clap<br/>CLI Parsing]
        BWSDk[bitwarden SDK<br/>Secrets Manager]
        SSHLib[ssh-agent-lib<br/>SSH Protocol]
        Tokio[tokio<br/>Async Runtime]
        Serde[serde/serde_yaml<br/>Serialization]
    end

    Main --> Clap
    ClientWrap --> BWSDk
    Agent --> SSHLib
    Main --> Tokio
    Config --> Serde

    style Main fill:#D4D4D4,stroke:#333333,stroke-width:2px,color:#000
    style Config fill:#ffcc80,stroke:#e65100,stroke-width:2px,color:#000
    style ProcessMgr fill:#ce93d8,stroke:#4a148c,stroke-width:2px,color:#000
    style Agent fill:#a5d6a7,stroke:#1b5e20,stroke-width:2px,color:#000
    style ClientWrap fill:#a5d6a7,stroke:#1b5e20,stroke-width:2px,color:#000
    style Logger fill:#f48fb1,stroke:#880e4f,stroke-width:2px,color:#000
```

## SSH Agent Architecture

This diagram shows how the SSH agent handles authentication requests and signing operations (e.g., git commit signing).

```mermaid
sequenceDiagram
    participant SSH as SSH Client<br/>(ssh user@host)
    participant Git as Git<br/>(git commit -S)
    participant Socket as Unix Socket<br/>(/tmp/vc-USERNAME-ssh-agent.sock)
    participant Listener as ssh-agent-lib<br/>listen()
    participant Agent as BitwardenAgent<br/>(Session trait)
    participant Fetcher as SecretFetcher<br/>(get_private_key)
    participant Cache as Key Cache<br/>(Arc<Mutex<Vec>>)
    participant BW as Bitwarden SDK<br/>(get_secret)

    Note over SSH,BW: Flow 1: SSH Authentication (list identities)

    SSH->>Socket: SSH_AGENTC_REQUEST_IDENTITIES
    Socket->>Listener: Forward protocol message
    Listener->>Agent: request_identities()
    
    loop For each secret_id (0..n)
        Agent->>Fetcher: get_private_key(index)
        Fetcher->>Cache: Check if key cached
        
        alt Key in cache
            Cache-->>Fetcher: Return cached PrivateKey
        else Key not in cache
            Fetcher->>BW: get_secret(uuid)
            BW-->>Fetcher: SecretData (name, value)
            Fetcher->>Fetcher: PrivateKey::from_openssh(value)
            Fetcher->>Cache: Store PrivateKey + name
            Cache-->>Fetcher: Return PrivateKey
        end
        
        Fetcher-->>Agent: PrivateKey
        Agent->>Agent: Extract public_key()
        Agent->>Agent: Build Identity (pubkey + comment)
    end
    
    Agent-->>Listener: Vec<Identity> (public keys)
    Listener-->>Socket: SSH_AGENT_IDENTITIES_ANSWER
    Socket-->>SSH: List of public keys

    Note over SSH,BW: Flow 2: Git Commit Signing (sign data)

    Git->>Socket: SSH_AGENTC_SIGN_REQUEST<br/>(pubkey, commit data, flags)
    Socket->>Listener: Forward protocol message
    Listener->>Agent: sign(SignRequest)
    
    loop For each secret_id (find matching key)
        Agent->>Fetcher: get_private_key(index)
        Fetcher->>Cache: Check if key cached
        
        alt Key in cache
            Cache-->>Fetcher: Return cached PrivateKey
        else Key not in cache
            Fetcher->>BW: get_secret(uuid)
            BW-->>Fetcher: SecretData (name, value)
            Fetcher->>Fetcher: PrivateKey::from_openssh(value)
            Fetcher->>Cache: Store PrivateKey + name
            Cache-->>Fetcher: Return PrivateKey
        end
        
        Fetcher-->>Agent: PrivateKey
        Agent->>Agent: Compare pubkey with request.pubkey
        
        alt Pubkey matches
            Agent->>Agent: key.try_sign(request.data)
            Agent-->>Listener: Signature bytes
            Listener-->>Socket: SSH_AGENT_SIGN_RESPONSE
            Socket-->>Git: Signature
            Note over Git: Git verifies signature<br/>and commits
        else Pubkey doesn't match
            Agent->>Agent: Try next key
        end
    end

    Note over SSH,BW: Both flows use the same caching mechanism<br/>Keys are fetched lazily and cached for performance
```

## Process Lifecycle

A complete lifecycle of the agent process is shown below.

```mermaid
stateDiagram-v2
    [*] --> Stopped: Initial state

    Stopped --> Starting: User runs 'start'
    Stopped --> ShowLogs: User runs 'logs'

    Starting --> CheckExisting: Check PID file

    CheckExisting --> CleanStale: PID exists but process dead
    CheckExisting --> AlreadyRunning: Process already running
    CheckExisting --> LoadConfig: No PID file

    CleanStale --> LoadConfig: Remove stale files

    LoadConfig --> SpawnDaemon: Config valid
    LoadConfig --> ConfigError: Config invalid

    SpawnDaemon --> WritePID: Fork process with --fg
    WritePID --> Running: Save PID to file

    state Running {
        [*] --> Authenticate: Initialize
        Authenticate --> BindSocket: Login to Bitwarden
        BindSocket --> Listen: Create Unix socket
        Listen --> Idle: Wait for connections

        Idle --> HandleRequest: SSH client connects
        HandleRequest --> FetchSecret: Request identities
        FetchSecret --> CacheKey: Parse private key
        CacheKey --> Idle: Return public keys

        Idle --> SignData: Sign request
        SignData --> Idle: Return signature

        Idle --> ListenSignal: No activity
        ListenSignal --> InternalShutdown: SIGTERM/SIGINT received
    }

    Running --> ExternalShutdown: User runs 'stop' sends SIGTERM

    InternalShutdown --> CleanupFiles
    ExternalShutdown --> CleanupFiles

    CleanupFiles --> Stopped: Files removed

    AlreadyRunning --> [*]: Error
    ConfigError --> [*]: Error

    ShowLogs --> Stopped: Display log file

    note right of Running
        Daemon process runs with VC_DAEMON_CHILD=1
        Stdout and stderr redirected
        Logging to file
    end note

    note left of CleanupFiles
        Graceful shutdown steps
        1 Close socket listener
        2 Finish pending requests
        3 Remove PID file
        4 Remove socket file
    end note
```

## Command Flow

Below, how different CLI commands are processed.

```mermaid
sequenceDiagram
    participant User
    participant CLI as main.rs<br/>(CLI Parser)
    participant Logger as logging.rs
    participant ProcMgr as process_manager.rs
    participant Client as client_wrapper.rs
    participant Agent as agent.rs
    participant BW as Bitwarden SDK

    User->>CLI: Execute command
    CLI->>Logger: setup_logging()

    alt Start Command (Background)
        CLI->>ProcMgr: start_agent_background()
        ProcMgr->>ProcMgr: Check if already running
        ProcMgr->>ProcMgr: Spawn new process with --fg
        ProcMgr->>ProcMgr: Write PID file
        ProcMgr-->>CLI: Return success
    else Start Command (Foreground)
        CLI->>Client: start_agent_foreground()
        Client->>BW: Authenticate with access token
        BW-->>Client: Authentication success
        Client->>Agent: Create BitwardenAgent
        Client->>Client: Bind Unix socket
        Client->>Agent: listen() - Start accepting connections
        Agent->>BW: Fetch secrets on-demand
        Agent-->>Agent: Cache private keys
    else Stop Command
        CLI->>ProcMgr: stop_agent()
        ProcMgr->>ProcMgr: Read PID file
        ProcMgr->>ProcMgr: Send SIGTERM
        ProcMgr->>ProcMgr: Wait and send SIGKILL if needed
        ProcMgr->>ProcMgr: Cleanup PID and socket files
        ProcMgr-->>CLI: Return success
    else Logs Command
        CLI->>ProcMgr: show_log_file()
        ProcMgr->>ProcMgr: Open with 'less'
        ProcMgr-->>CLI: Return success
    end

    CLI-->>User: Command complete
```

## Concurrency Model

The application's concurrency and synchronization strategy:

```mermaid
graph TB
    subgraph MainThread["Main Thread"]
        MainFn[main function<br/>#40;tokio main macro#41;]
    end

    subgraph TokioRuntime["Tokio Runtime"]
        Runtime[Multi-threaded Runtime<br/>#40;rt-multi-thread feature#41;]

        subgraph AgentProcess["Agent Process"]
            Listen[listen task<br/>Accept connections]

            subgraph PerConnection["Per Connection"]
                Conn1[Connection Handler 1]
                Conn2[Connection Handler 2]
                ConnN[Connection Handler N]
            end

            SignalTerm[SIGTERM Handler]
            SignalInt[SIGINT Handler]
        end
    end

    subgraph SharedState["Shared State"]
        KeyCache[Arc Mutex Vec Option PrivateKey<br/>Cached Keys]
        NameCache[Arc Mutex Vec Option String<br/>Cached Key Names]
        ClientArc[Arc Client<br/>Bitwarden Client]
    end

    MainFn -->|Spawns| Runtime
    Runtime --> Listen

    Listen -->|tokio spawn| Conn1
    Listen -->|tokio spawn| Conn2
    Listen -->|tokio spawn| ConnN

    Listen -->|tokio select| SignalTerm
    Listen -->|tokio select| SignalInt

    Conn1 <-->|Lock for read/write| KeyCache
    Conn2 <-->|Lock for read/write| KeyCache
    ConnN <-->|Lock for read/write| KeyCache

    Conn1 <-->|Lock for read/write| NameCache
    Conn2 <-->|Lock for read/write| NameCache
    ConnN <-->|Lock for read/write| NameCache

    Conn1 -->|Shared reference| ClientArc
    Conn2 -->|Shared reference| ClientArc
    ConnN -->|Shared reference| ClientArc

    SignalTerm -->|Triggers| Cleanup[cleanup_files function]
    SignalInt -->|Triggers| Cleanup

    %%style MainFn fill:#D4D4D4,stroke:#333333,stroke-width:2px,color:#000
    %%style Runtime fill:#ce93d8,stroke:#4a148c,stroke-width:2px,color:#000
    style KeyCache fill:#ffcc80,stroke:#e65100,stroke-width:2px,color:#000
    style NameCache fill:#ffcc80,stroke:#e65100,stroke-width:2px,color:#000
    style ClientArc fill:#a5d6a7,stroke:#1b5e20,stroke-width:2px,color:#000
    %%style Cleanup fill:#ef5350,stroke:#b71c1c,stroke-width:2px,color:#fff

    Note1[Arc: Atomic Reference Counting<br/>Mutex: Mutual Exclusion<br/>tokio select: Concurrent await]
    Note1 -.-> Runtime
```

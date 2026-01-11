[![CI](https://github.com/pirafrank/vault-conductor/actions/workflows/ci.yml/badge.svg)](https://github.com/pirafrank/vault-conductor/actions/workflows/ci.yml)
[![CI Cross](https://github.com/pirafrank/vault-conductor/actions/workflows/ci_cross.yml/badge.svg)](https://github.com/pirafrank/vault-conductor/actions/workflows/ci_cross.yml)
[![Release](https://img.shields.io/github/release/pirafrank/vault-conductor.svg)](https://github.com/pirafrank/vault-conductor/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

# vault-conductor

An SSH Agent to provide SSH keys stored in Bitwarden Secret Manager as secrets.

## Features

It implements the SSH Agent Protocol as a Unix domain socket server, acting as a secure bridge between your SSH clients and Bitwarden Secrets Manager.

It features:

- **Agent lifecycle**: Runs as a daemon (background) or foreground process, listening on a Unix socket at `/tmp/vc-$(whoami)-ssh-agent.sock`
- **Lazy loading of keys**: SSH keys are fetched from Bitwarden via their official Rust SDK only when requested, then cached in memory
- **Safe SSH operations**: When SSH clients query identities or request signatures, the agent handles requests using the `ssh-agent-lib` crate without ever exposing private keys to disk
- **Process management**: Background mode spawns a detached child process, tracks PID, and supports graceful shutdown via SIGTERM/SIGINT
- **Security**: Socket permissions are locked to `0600` (owner-only), keys live only in process memory, and Bitwarden APIs are called using a scoped machine token which you can configure with granular secret access.

Under the hood, it's built with Tokio for async I/O, uses `ssh-key` crate for cryptographic operations, and supports both Ed25519 and RSA keys in OpenSSH format.

## Why

It was born out of a necessity of mine. [Bitwarden SSH Agent](https://bitwarden.com/help/ssh-agent/) feature in Bitwarden GUI client is handy, but what to use if you're running your devbox CLI only? How to securely bring your SSH key in a CI/CD pipeline to sign git commits? What if you need to open an SSH connection from an ephimeral container or VM without copying any private key?

So I wrote a tiny CLI tool to retrieve SSH keys and make them available without exposing their private counterpart.

And to avoid bringing your whole Bitwarden vault to the environment, it uses [Bitwarden Secrets Manager](https://bitwarden.com/products/secrets-manager/) so you can choose which machine can access to which secret and set granular token permissions.

## Requirements

- A Bitwarden account with configured [Bitwarden Secret Manager](https://bitwarden.com/help/secrets-manager-quick-start/) (which you can create and setup for free) (support for self-hosted Bitwarden is planned)
- An Ed25519 or RSA SSH key in OpenSSH new format saved as secret value in BWS
  - It needs to be saved including  `-----BEGIN OPENSSH PRIVATE KEY-----` and `-----END OPENSSH PRIVATE KEY-----` strings.
  - Note: new OpenSSH private key format was introduced with OpenSSH 7.8 in 2018.
- macOS or Linux released in the last 5 years

## Installation

Either by using [poof](https://github.com/pirafrank/poof):

```sh
poof install pirafrank/vault-conductor
```

with a quick one-liner:

```sh
curl -fsSL https://raw.githubusercontent.com/pirafrank/vault-conductor/main/install.sh | sh
```

or by manually download the [latest stable](https://github.com/pirafrank/vault-conductor/releases/latest) release and put it to `$PATH`.

## Configuration

You have to provide:

- `BWS_ACCESS_TOKEN`, the machine token you have set up above. The environment variable has the same name as the `bws` CLI tool [by Bitwarden](https://github.com/bitwarden/sdk-sm/releases/tag/bws-v1.0.0)
- `BW_SECRET_IDS`, comma-separated list of UUIDs of secrets where each private key is stored. You can read the UUID of each secret in the BWS web app (check under the secret name).

You can either pass them as the above environment variables (good for CI and DevOps setups) or via config file:

```sh
# download the example config file at the default path, then customize to your needs
mkdir ~/.config/vault-conductor
curl -sSL https://github.com/pirafrank/vault-conductor/raw/refs/heads/main/config.yaml.example > ~/.config/vault-conductor/config.yaml
chmod 0660 ~/.config/vault-conductor/config.yaml
```

## Usage

```sh
# set SSH Agent env var to vault-conductor socket
export SSH_AUTH_SOCK="/tmp/vc-$(whoami)-ssh-agent.sock"

# Start in foreground
# (recommended for first time users to verify config is ok)
vault-conductor start --fg

# Start the agent in background
vault-conductor start

# Stop the background agent
vault-conductor stop
```

The `start` command also supports `--config` option to provide a custom configuration path. **Environment variables always take precedence over config file.**

## Debug

Sometimes you may need to debug a weird situation and need as much log as possible. Execute the following to run in foreground and get verbose sysout logs:

```sh
vault-conductor start --fg -vv
```

## Install as a service

You can install it as a Systemd service in userspace. Read more [here](docs/SERVICE.md).

## Documentation

Check the [docs](docs/README.md) directory to find diagrams about how the code works and is organized.

## What's next

- [x] Support multiple SSH keys
- [ ] Support self-hosted Bitwarden setups
- [ ] Better testing
- [ ] Offer more ways to install (Homebrew, AUR, nix, .deb, .rpm)
- [ ] Support providers other than Bitwarden?

## About the name

*vault*, /voːlt/ - *an underground room, especially for storing valuables*

*conductor*, /kənˈdʌk·tər/ - *a director, a thing that conducts heat or electricity*

by extension, something that conducts your valuable SSH key from a Bitwarden vault to your dev environment.

## License

This project is licensed under the MIT License.

See the [LICENSE](LICENSE) file for details.

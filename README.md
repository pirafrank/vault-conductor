[![CI](https://github.com/pirafrank/vault-conductor/actions/workflows/ci.yml/badge.svg)](https://github.com/pirafrank/vault-conductor/actions/workflows/ci.yml)
[![CI Cross](https://github.com/pirafrank/vault-conductor/actions/workflows/ci_cross.yml/badge.svg)](https://github.com/pirafrank/vault-conductor/actions/workflows/ci_cross.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

# vault-conductor

An SSH Agent to provide an SSH key stored in Bitwarden Secret Manager.

## Why

It was born out of a necessity of mine. [Bitwarden SSH Agent]() feature in Bitwarden GUI client is handy, but what to use if you're running your devbox CLI only? How to securely bring your SSH key to connect and git commit sign on an ephimeral container or VM securely?

Meet `vault-conductor`: a tiny CLI tool to securely retrieve your SSH key and make it available without exposing its private counterpart.

And to avoid bringing your whole vault to the environment, it uses [Bitwarden Secrets Manager](https://bitwarden.com/products/secrets-manager/) so you can choose which machine can access to which secret and set granular token permissions for them.

## Requirements

- A Bitwarden account with configured [Bitwarden Secret Manager](https://bitwarden.com/help/secrets-manager-quick-start/) (which you can create and setup for free)
- An Ed25519 or RSA SSH key in OpenSSH new format saved as secret value in BWS
  - It needs to be saved including  `-----BEGIN OPENSSH PRIVATE KEY-----` and `-----END OPENSSH PRIVATE KEY-----` strings.
  - Note: new private key format was introduced in OpenSSH 7.8 in 2018.
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

- `BWS_ACCESS_TOKEN`, the machine token you have set up above. The variable has the same name as the `bws` CLI tool [by Bitwarden](https://github.com/bitwarden/sdk-sm/releases/tag/bws-v1.0.0)
- `BW_SECRET_ID`, the `id` of the secret where the private key is stored. It's a UUID and you can read it in the BWS web app under the secret name.

You can either pass them via config file:

```sh
mkdir ~/.config/vault-conductor
curl -sSL https://github.com/pirafrank/vault-conductor/raw/refs/heads/main/config.yaml.example > ~/.config/vault-conductor/config.yaml
chmod 0660 ~/.config/vault-conductor/config.yaml
```

or by setting `BWS_ACCESS_TOKEN` and `BW_SECRET_ID` as environment variables (good for CI and DevOps setups).

## Usage

```sh
# set SSH Agent env var to vault-conductor socket
export SSH_AUTH_SOCK="/tmp/vc-ssh-agent.sock"

# Start the agent in background
vault-conductor start-agent

# Stop the background agent
vault-conductor stop-agent

# Restart the agent
vault-conductor restart-agent
```

## Debug

Sometimes you may need to debug a weird situation and need as much log as possible. Run the following to get verbose sysout log:

```sh
vault-conductor start-agent --fg -vv
```

## What's next

- [ ] Better testing
- [ ] Support multiple SSH keys
- [ ] Offer more ways to install (Homebrew, AUR, nix, .deb, .rpm)
- [ ] Support self-hosted Bitwarden setups
- [ ] Support providers other than Bitwarden?

## About the name

*vault*, /voːlt/ - *an underground room, especially for storing valuables*

*conductor*, /kənˈdʌk·tər/ - *a director, a thing that conducts heat or electricity*

by extension, something that conducts your valuable SSH key from a Bitwarden vault to your dev environment.

## License

This project is licensed under the MIT License.

See the [LICENSE](LICENSE) file for details.

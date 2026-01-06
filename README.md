[![Licenses](https://github.com/pirafrank/vault-conductor/actions/workflows/licenses.yml/badge.svg)](https://github.com/pirafrank/vault-conductor/actions/workflows/licenses.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

# vault-conductor

An SSH Agent to provide an SSH key stored in Bitwarden Secret Manager.

## Installation

TODO

## Requirements

- A Bitwarden account with configured [Bitwarden Secret Manager](https://bitwarden.com/products/secrets-manager/) (which you can create and setup for free)
- An SSH key in OpenSSH new format saved as a secret value (including `-----BEGIN OPENSSH PRIVATE KEY-----` and `-----END OPENSSH PRIVATE KEY-----`). New private key format was introduced in OpenSSH 7.8 in 2018.
- macOS or Linux released in the last 5 years

## Getting started

TODO

## Usage

```sh
# Start the agent in background
vault-conductor start-agent

# Stop the background agent
vault-conductor stop-agent

# Restart the agent
vault-conductor restart-agent
```

## About the name

*vault*, /voːlt/ - *an underground room, especially for storing valuables*
*conductor*, /kənˈdʌk·tər/ - *a director, a thing that conducts heat or electricity*

by extension, something can conducts your valuable SSH key from Bitwarden vault to your dev environment.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE.md) file for details.


# Security Policy

Vault Conductor is an SSH agent that retrieves SSH keys from Bitwarden Secrets
Manager and serves them to local SSH clients over a Unix domain socket. We take
the protection of access tokens, private keys, and the agent interface
seriously.

## Supported Versions

Security fixes are currently provided for the latest released `0.x` version of
Vault Conductor. Users should upgrade to the latest stable release whenever
possible. Older releases may not receive security updates.

We aim to minimize breaking changes and configuration changes, but a security
fix may require them when necessary to protect users.

## Reporting a Vulnerability

**Please do not open a public GitHub issue, discussion, or pull request for a
security vulnerability.** Do not include access tokens, private keys, or other
secrets in a report.

The preferred reporting channel is GitHub's private vulnerability reporting:

**[Report a vulnerability privately](https://github.com/pirafrank/vault-conductor/security/advisories/new)**

If private vulnerability reporting is unavailable, contact the maintainer
through the [repository owner's GitHub profile](https://github.com/pirafrank)
and request a private security contact. Please do not disclose sensitive
details until a private channel has been established.

Please include, where possible:

- A concise description of the vulnerability and its potential impact.
- The affected version, commit, operating system, and installation method.
- Reproduction steps or a minimal proof of concept.
- Relevant configuration, logs, or stack traces after removing all secrets.
- Any suggested mitigation or fix.

Reports involving local privilege escalation, unauthorized access to the Unix
socket, access-token exposure, private-key leakage, or unsafe Bitwarden API
behavior are especially important. If a report is time-sensitive, say so in
the submission.

We will try to acknowledge a report as fast as possible. We will investigate the
issue, keep the reporter informed when practical, and coordinate remediation
and disclosure timing. We may credit reporters in the security advisory unless
they request anonymity.

## Security Model and User Responsibilities

- The SSH agent socket defaults to `/tmp/vc-$(whoami)-ssh-agent.sock` and is
  created with owner-only (`0600`) permissions. Verify the permissions and
  parent-directory security on your system. Never expose the socket through a
  network service or make it accessible to other local users.
- Treat `BWS_ACCESS_TOKEN` as a credential with access to the configured
  Bitwarden Secrets Manager resources. Store it only in a protected secret
  store or configuration file, restrict configuration files to the owning user
  (for example, mode `0600`), and rotate the token if exposure is suspected.
- Environment variables can be visible to other processes or users depending
  on the operating system and process-monitoring permissions. Avoid them when a
  suitably protected configuration or secret-management mechanism is available.
- Retrieved private keys are held in the agent process memory while in use.
  Protect the host and process from unauthorized access, and do not run the
  agent under a shared account.
- Keep the operating system, Vault Conductor, Rust dependencies, and Bitwarden
  components up to date. Avoid forwarding the agent through an untrusted SSH
  host.
- Rust reduces the risk of memory-safety defects such as buffer overflows, but
  it does not eliminate application, dependency, configuration, or host-level
  security risks.

## Dependency Security Monitoring

Vault Conductor uses [GitHub Dependabot](https://github.com/pirafrank/vault-conductor/network/updates)
to check Rust dependencies weekly and propose grouped updates, including
security-related updates where supported. The repository also runs a scheduled
weekly [`cargo audit`](https://github.com/RustSec/cargo-audit) check through
GitHub Actions and audits dependency changes in pull requests and selected
pushes.

These automated checks reduce risk but do not replace keeping installations
current or privately reporting vulnerabilities discovered by users.

## Known Dependency Advisory

`cargo audit` reports [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html),
the Marvin Attack, for two RSA versions in the dependency tree:

- `rsa 0.9.10` through `ssh-key`, which the SSH agent uses to parse and sign
  with configured private keys.
- `rsa 0.10.0-rc.18` through Bitwarden's cryptographic dependencies. This is a
  transitive dependency; Vault Conductor's Secrets Manager flow does not
  directly expose an RSA decryption operation.

The advisory currently has no fixed version. It concerns timing leakage during
RSA private-key operations and requires an attacker to observe many operations.
The default owner-only socket permissions reduce local exposure, but do not
remove the risk entirely. Prefer Ed25519 keys, avoid reusing RSA keys with
remotely observable RSA services, and rerun `cargo audit` after dependency
updates. We will revisit this advisory when a fixed RSA release or compatible
dependency change becomes available.

Track the project-specific assessment and remediation work in
[issue #13](https://github.com/pirafrank/vault-conductor/issues/13).

## Disclosure Process

For a confirmed vulnerability, we will assess severity and affected versions,
develop and test a fix privately where practical, and publish an updated
release and GitHub security advisory. We will request a CVE when appropriate.
Disclosure timing will be coordinated with the reporter and may be adjusted
to account for exploitability, available mitigations, and the time users need
to update.

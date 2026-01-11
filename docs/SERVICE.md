# Systemd service

You can install `vault-conductor` as a Systemd service in userspace to make it run at boot.

## Install

Run the following:

```sh
mkdir -p ~/.config/systemd/user
vim ~/.config/systemd/user/vault-conductor.service
```

and paste the following customized for your environment (replace text in UPPERCASE):

```txt
[Unit]
Description=Vault Conductor SSH Agent
Documentation=https://github.com/pirafrank/vault-conductor
After=network.target

[Service]
Type=simple
ExecStart=/PATH/TO/vault-conductor start --fg
Restart=on-failure
RestartSec=5s
Environment="SSH_AUTH_SOCK=/tmp/USERNAME/vc-ssh-agent.sock"

[Install]
WantedBy=default.target
```

Then install with:

```sh
systemctl --user enable vault-conductor.service
systemctl --user daemon-reload
```

## Manage or disable

Manage with standard `systemctl` commands:

```sh
systemctl --user status vault-conductor.service
systemctl --user start vault-conductor.service
systemctl --user stop vault-conductor.service
```

## Check logs

```sh
journalctl --user -u vault-conductor.service -f
```

## Uninstall

```sh
systemctl --user disable vault-conductor.service
systemctl --user daemon-reload
rm -f ~/.config/systemd/user/vault-conductor.service
```
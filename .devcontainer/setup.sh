#!/bin/bash
set -e  # Exit on any error

echo "🔧 Setting up development environment..."

# Update package lists
sudo apt-get update

# Install system packages
sudo apt-get install -y \
    build-essential \
    curl \
    git \
    vim \
    htop \
    tree \
    wget \
    bat fd-find ripgrep zip unzip fuse libfuse2 mosh \
    qemu-user-static binfmt-support

# Install Rust (if not already installed)
#
# Note: Rust should already be installed in devcontainer if
#       "ghcr.io/devcontainers/features/rust:1" option or
#       mcr.microsoft.com/devcontainers/rust:1 dockerimage are used
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Install additional cargo packages
cargo install cross --git https://github.com/cross-rs/cross

# Store shell history to persisted /workspaces directory
echo 'export HISTFILE=/workspaces/.zsh_history' >> ~/.zshrc

# Any other custom setup
echo "✅ Setup complete!"

#!/usr/bin/env bash

set -e

latest_tag=$(git tag -l | grep -E '^v?[0-9]+\.[0-9]+\.[0-9]+$' | sort -rV | head -n1)

# Check if tag exists
if [[ -z "$latest_tag" ]]; then
  echo "No version tags found"
  exit 0
fi

# Check if the version in Cargo.toml matches the latest tag
cargo_version=$(grep -m 1 "version = " Cargo.toml | cut -d'"' -f2)
if [[ "$cargo_version" != "${latest_tag#v}" ]]; then
  echo "Error: Version in Cargo.toml ($cargo_version) doesn't match latest tag (${latest_tag#v})"
  exit 1
fi

# Check if the version in Cargo.lock matches the latest tag
lock_version=$(grep -m 1 "name = \"poof\"" -A 1 Cargo.lock | grep "version" | cut -d'"' -f2)
if [[ "$lock_version" != "${latest_tag#v}" ]]; then
  echo "Error: Version in Cargo.lock ($lock_version) doesn't match latest tag (${latest_tag#v})"
  exit 1
fi

# Check if the tag appears in CHANGELOG.md
if ! grep -q "\[${latest_tag#v}\]" CHANGELOG.md; then
  echo "Error: Latest tag $latest_tag not found in CHANGELOG.md"
  exit 1
fi

echo "All version checks passed for $latest_tag"

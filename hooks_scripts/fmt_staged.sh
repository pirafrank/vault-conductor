#!/usr/bin/env bash

set -e

staged_rust_files=$(git diff --cached --name-only --diff-filter=ACMR | grep '\.rs$' || true)

if [ -z "$staged_rust_files" ]; then
    echo "✅ No staged Rust files. Skipping."
    exit 0
fi

# Auto-format (writes changes to files)
echo "🧹 Auto-formatting staged Rust files..."
echo "$staged_rust_files" | xargs -r cargo fmt --

# Re-stage formatted files (cargo fmt doesn't auto-add)
echo "$staged_rust_files" | xargs -r git add

echo "✅ Formatting complete! All changes have been staged."
exit 0

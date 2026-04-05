#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?Usage: patch-version.sh <VERSION>}"

# Validate semver: N.N.N with optional pre-release suffix
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
  echo "Error: '$VERSION' is not valid semver (expected N.N.N or N.N.N-pre.1)" >&2
  exit 1
fi

CARGO_TOML="$(git rev-parse --show-toplevel)/Cargo.toml"

# Patch version line in Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" "$CARGO_TOML"
rm -f "$CARGO_TOML.bak"

# Verify patch succeeded
if grep -q "^version = \"$VERSION\"" "$CARGO_TOML"; then
  echo "Patched Cargo.toml to version $VERSION"
else
  echo "Error: failed to patch version in $CARGO_TOML" >&2
  exit 1
fi

#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?Usage: patch-version.sh <VERSION>}"

# --- package.json ---
npm version "$VERSION" --no-git-tag-version
ACTUAL=$(npm pkg get version | tr -d '"')
if [[ "$ACTUAL" != "$VERSION" ]]; then
  echo "::error::package.json version mismatch: expected $VERSION, got $ACTUAL"
  exit 1
fi
echo "✓ package.json → $ACTUAL"

# --- src-tauri/Cargo.toml ---
# Replace only the first ^version line (the package version, not a dependency)
sed -i.bak '0,/^version = ".*"/{s//version = "'"$VERSION"'"/}' src-tauri/Cargo.toml
rm -f src-tauri/Cargo.toml.bak

if ! grep -q "^version = \"$VERSION\"" src-tauri/Cargo.toml; then
  echo "::error::Cargo.toml version patch failed"
  exit 1
fi
echo "✓ Cargo.toml → $VERSION"

# --- src-tauri/tauri.conf.json ---
node -e "
const fs = require('fs');
const path = 'src-tauri/tauri.conf.json';
const conf = JSON.parse(fs.readFileSync(path, 'utf8'));
conf.version = process.argv[1];
fs.writeFileSync(path, JSON.stringify(conf, null, 2) + '\n');
" "$VERSION"

TAURI_VER=$(node -e "
const fs = require('fs');
const conf = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));
console.log(conf.version);
")
if [[ "$TAURI_VER" != "$VERSION" ]]; then
  echo "::error::tauri.conf.json version mismatch: expected $VERSION, got $TAURI_VER"
  exit 1
fi
echo "✓ tauri.conf.json → $TAURI_VER"

echo "All manifests patched to $VERSION"

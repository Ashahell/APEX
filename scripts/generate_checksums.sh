#!/usr/bin/env bash
set -euo pipefail

RELEASE_DIR="./release-artifacts"
if [[ ! -d "$RELEASE_DIR" ]]; then
  echo "Creating release-artifacts dir..."; mkdir -p "$RELEASE_DIR";
fi

echo "Generating checksums for release artifacts..."
for f in "$RELEASE_DIR"/*; do
  if [[ -f "$f" ]]; then
    sha256sum "$f" | awk '{print $1"  "$f}' >> "$RELEASE_DIR/sha256sums.txt"
  fi
done
echo "Checksums written to $RELEASE_DIR/sha256sums.txt"

#!/usr/bin/env bash
set -euo pipefail
TAG=${1:-v2.0.0}
git fetch --tags
git tag -a "$TAG" -m "Release $TAG: parity baseline and Phase 2 gating in CI"
git push origin "$TAG"
echo "Released tag $TAG and pushed to origin."

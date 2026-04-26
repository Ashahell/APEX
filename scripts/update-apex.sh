#!/bin/bash
#
# APEX Update Script
# Usage: ./scripts/update-apex.sh [version]
#

set -e

VERSION=${1:-latest}
CURRENT_VERSION=$(docker images apex/apex-router:latest --format "{{.Tag}}" 2>/dev/null || echo "unknown")

echo "APEX Update Script"
echo "================"
echo "Current version: ${CURRENT_VERSION}"
echo "New version: ${VERSION}"
echo ""

# Confirm
read -p "Proceed with update? (y/N) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Update cancelled."
    exit 0
fi

# Pull new image
echo "Pulling new image..."
docker pull apex/apex-router:${VERSION}

# Stop current container
echo "Stopping current container..."
docker stop apex-router || true

# Remove old container
echo "Removing old container..."
docker rm apex-router || true

# Run new container
echo "Starting new container..."
docker run -d \
    --name apex-router \
    -p 3000:3000 \
    --restart unless-stopped \
    -e APEX_SHARED_SECRET="${APEX_SHARED_SECRET}" \
    -e APEX_DATABASE_URL="sqlite:apex.db" \
    -v apex_data:/data \
    apex/apex-router:${VERSION}

# Wait for health
echo "Waiting for health check..."
sleep 5

# Verify
if curl -sf http://localhost:3000/health > /dev/null 2>&1; then
    echo "Update complete!"
    echo "APEX is running version: ${VERSION}"
else
    echo "WARNING: Health check failed. Check logs with: docker logs apex-router"
fi
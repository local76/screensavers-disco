#!/usr/bin/env bash
set -euo pipefail

# Run test script
echo "==> Running test script..."
./scripts/test.sh

# Run build script
echo "==> Running build script..."
./scripts/build.sh

# Tag Git release and push
echo "==> Creating Git tag..."
VERSION=$(cargo metadata --format-version 1 | grep -o '"version":"[^"]*"' | head -n1 | cut -d'"' -f4)
TAG_NAME="v$VERSION"

if git rev-parse "$TAG_NAME" >/dev/null 2>&1; then
    echo "Tag $TAG_NAME already exists, skipping tag creation."
else
    git tag -a "$TAG_NAME" -m "Release $TAG_NAME"
    echo "Tagged release $TAG_NAME"
fi

echo "==> Pushing tags..."
if git remote | grep -q 'origin'; then
    git push origin "$TAG_NAME"
else
    echo "No 'origin' remote found. Tag created locally."
fi

echo "==> Release complete!"

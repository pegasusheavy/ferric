#!/bin/bash

# Install git hooks from .cargo-husky/hooks to .git/hooks
# Run this script if hooks need to be reinstalled

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
HOOKS_SOURCE="$SCRIPT_DIR/hooks"
HOOKS_DEST="$PROJECT_ROOT/.git/hooks"

echo "🔧 Installing Git hooks..."

if [ ! -d "$HOOKS_DEST" ]; then
    echo "❌ Error: .git/hooks directory not found"
    echo "Make sure you're in a git repository"
    exit 1
fi

# Copy hooks
for hook in pre-commit pre-push commit-msg; do
    if [ -f "$HOOKS_SOURCE/$hook" ]; then
        cp "$HOOKS_SOURCE/$hook" "$HOOKS_DEST/$hook"
        chmod +x "$HOOKS_DEST/$hook"
        echo "✅ Installed $hook hook"
    else
        echo "⚠️  Warning: $hook hook not found in $HOOKS_SOURCE"
    fi
done

echo "✨ Git hooks installation complete!"


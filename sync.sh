#!/usr/bin/env bash
# Aether Directory Sync Helper
#
# Since standard Windows mounts (/mnt/c/...) in WSL are slower for cargo builds,
# this script easily synchronizes code between the Windows Desktop folder
# and the native fast WSL home directory (~/aether).

WSL_DIR="$HOME/aether"
WIN_DIR="/mnt/c/Users/sharm/Desktop/Aether"

# Ensure WSL directory exists
mkdir -p "$WSL_DIR"

case "$1" in
    "to-wsl")
        echo "🔄 Syncing FROM Windows host TO WSL native (~/aether)..."
        rsync -av --delete \
            --exclude='target/' \
            --exclude='.git/' \
            --exclude='.idea/' \
            --exclude='.vscode/' \
            --exclude='*.bin' \
            --exclude='*.img' \
            "$WIN_DIR/" "$WSL_DIR/"
        echo "✅ Sync complete! WSL workspace is ready for super-fast compilation."
        ;;
    "to-win")
        echo "🔄 Syncing FROM WSL native TO Windows host (/mnt/c/...)..."
        rsync -av --delete \
            --exclude='target/' \
            --exclude='.git/' \
            --exclude='.idea/' \
            --exclude='.vscode/' \
            --exclude='*.bin' \
            --exclude='*.img' \
            "$WSL_DIR/" "$WIN_DIR/"
        echo "✅ Sync complete! Windows workspace is updated (viewable on IDE/GitHub)."
        ;;
    *)
        echo "Aether Workspace Synchronizer"
        echo "----------------------------"
        echo "Usage: ./sync.sh [to-wsl | to-win]"
        echo ""
        echo "Commands:"
        echo "  to-wsl : Copy edits from Windows Desktop folder to fast WSL home disk (~/aether)"
        echo "  to-win : Copy edits from WSL native back to Windows Desktop folder"
        echo ""
        echo "Note: Both options exclude heavy build 'target/' directories to keep syncs instantaneous."
        ;;
esac

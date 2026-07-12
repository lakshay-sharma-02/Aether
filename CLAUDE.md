# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Aether is an AI-native operating system built entirely in Rust, where the AI layer is a first-class kernel subsystem running in ring 0 with direct hardware access. The project consists of:

- **kernel/** — Bare-metal x86_64 kernel (`no_std`, bootloader 0.9)
- **vexfs/** — Next-generation filesystem with embedded AI (slab allocator, ARC cache, B+ tree, FUSE-mountable)
- **ai-core/** — Kernel-level intelligence subsystem (future)
- **compositor/** — Window manager (future, forked from Hyprland)
- **shell/** — System shell (future)

The workspace uses Cargo with panic="abort" in both dev and release profiles.

## Build Commands

### Kernel Development

```bash
# Build kernel + bootimage (from repo root)
make build

# Build and run in QEMU with VGA display (curses mode in terminal)
make run

# Build and run with serial output only (no VGA window)
make serial

# Clean build artifacts
make clean
```

**Direct cargo commands** (from repo root):
```bash
# Build bootimage directly
~/.cargo/bin/cargo bootimage --manifest-path kernel/Cargo.toml

# Run in QEMU (requires bootimage in manifest)
cd kernel && cargo run
```

### VexFS Development

```bash
cd vexfs

# Build filesystem binaries
cargo build

# Run all tests (28 tests)
cargo test

# Create and format a disk image
dd if=/dev/zero of=~/vexfs.img bs=1M count=100
./target/debug/mkfs_vexfs ~/vexfs.img

# Mount the filesystem
mkdir -p ~/mnt/vexfs
./target/debug/vexfs ~/vexfs.img ~/mnt/vexfs

# Unmount
fusermount3 -u ~/mnt/vexfs
```

### WSL-Specific Workflow

The project is developed in WSL2. Use `sync.sh` to move code between Windows Desktop and native WSL storage for faster builds:

```bash
# Sync Windows → WSL (before building)
./sync.sh to-wsl

# Sync WSL → Windows (after changes)
./sync.sh to-win
```

The Makefile and sync.sh automatically exclude `target/`, `.git/`, `*.bin`, `*.img` during sync.

## Architecture Notes

### Kernel (kernel/)

- **Entry point**: `kernel/src/main.rs` → `_start()` → `kernel_main()`
- **Target**: `x86_64-unknown-none` (bare metal, no OS)
- **Linker**: `rust-lld` via `kernel/.cargo/config.toml`
- **Bootloader**: bootloader 0.9 with `bootimage` runner
- **VGA buffer**: 0xb8000, 80x25 text mode, color code 0x5F (white on purple)
- **Current state**: Prints "Aether" at (0,0), then halts in loop

**Critical constraint**: `kernel/.cargo/config.toml` sets the bare-metal target. The root `.cargo/config.toml` is intentionally minimal to avoid forcing this target on other workspace members (VexFS, ai-core).

**Known issue from git history**: Duplicate `_start` symbols can occur if workspace-level cargo config bleeds into kernel. The fix is to keep target/runner config scoped to `kernel/.cargo/config.toml` only.

### VexFS (vexfs/)

Five-layer architecture:

1. **Allocator** (src/allocator/) — Slab allocator, fixed-size pools, O(1), zero fragmentation
2. **Cache** (src/cache/) — ARC (Adaptive Replacement Cache), hard memory ceiling
3. **Filesystem Core** (src/fs/) — Superblock (magic: 0x5645584653000001), inode table at block 1 (256 bytes/inode), COW design. B+ tree (src/fs/btree.rs) powers all lookups and directory listings with O(log n) performance.
4. **FUSE Layer** (src/fuse/) — Mounts VexFS as real filesystem, full CRUD persistence
5. **AI Subsystem** (src/ai/):
   - **logger.rs** — Access event log (bounded to 10k events)
   - **markov.rs** — Sequence predictor, learns file access patterns
   - **importance.rs** — Scores files 0.0-1.0 for HOT/WARM/COLD tiering
   - **search.rs** — TF-IDF semantic search over file content

**Design constraints**:
- Total memory target: <80MB resident RAM
- No hidden allocations — all go through the slab allocator
- AI uses classical ML (Markov chains, TF-IDF) not neural nets — stays lightweight

**Binaries**:
- `vexfs/src/bin/vexfs.rs` — Mount binary
- `vexfs/src/bin/mkfs_vexfs.rs` — Format binary

## Development Environment

- **Platform**: Linux/WSL2 (Ubuntu 24.04)
- **Rust**: nightly (see rust-toolchain.toml)
- **QEMU**: qemu-system-x86_64 for kernel testing
- **FUSE**: libfuse3-dev required for VexFS

## Design Philosophy

- Memory-first: every component has a hard RAM ceiling
- No hidden costs: explicit over implicit, measured over assumed
- AI that costs nothing: classical ML over deep learning where possible
- Bare-metal correctness: no compromise on kernel safety

## Current Status (as of 2026-06-03)

- **Kernel**: Boots in QEMU, prints "Aether" to VGA, basic panic handler
- **VexFS**: Fully functional, 28 tests passing, mountable via FUSE, AI subsystem operational
- **ai-core, compositor, shell**: Scaffolded but not yet implemented

## Visual Design Language

Minimal + sharp + fluid + organic. Dark backgrounds, purple AI accents (0x5 in CGA palette), color as semantic signal.

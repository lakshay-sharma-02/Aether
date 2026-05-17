# Aether — Session Start Template

Copy this at the beginning of every new AI conversation.
Fill in the blanks, delete the instructions, then paste.

---

## Context

I am building **Aether** — an AI-native operating system in Rust.

**Core idea**: Jarvis is a first-class kernel subsystem that sits on a system-wide AI event bus.
Every kernel subsystem (scheduler, memory, VexFS, network) emits events into this bus.
Jarvis observes, learns, and acts — optimizing the machine for the developer in real time.

**Stack**:
- Kernel: Rust `no_std`, booted via bootloader crate, targeting x86_64
- Filesystem: VexFS (our own, ported from FUSE to bare metal)
- AI Core: ring buffer event bus + Jarvis engine
- Compositor: Hyprland fork with custom animations
- Shell: WGPU-based, GPU-rendered

**Repo structure**:
```
aether/
├── kernel/       ← no_std Rust kernel
├── vexfs/        ← VexFS ported to no_std
├── ai-core/      ← event bus + Jarvis
├── compositor/   ← Hyprland fork
├── shell/        ← WGPU shell (later)
└── DEVLOG.md
```

---

## Recent DEVLOG (last 3 entries)

[PASTE YOUR LAST 3 DEVLOG ENTRIES HERE]

---

## Current state

[DESCRIBE WHAT IS CURRENTLY WORKING / WHAT THE CODE LOOKS LIKE RIGHT NOW]

Example:
> kernel boots in QEMU, prints "Aether" to VGA buffer in purple.
> VGA writer is raw buffer write, no cursor yet.
> No interrupts, no memory allocator yet.

---

## Today's goal

[ONE SPECIFIC THING YOU WANT TO BUILD THIS SESSION]

Example:
> Implement a proper VGA text buffer writer in kernel/src/vga.rs with:
> - a Writer struct with cursor tracking
> - print! and println! macros
> - color support using the ColorCode enum

---

## Relevant files

[PASTE THE FILES THAT ARE DIRECTLY RELEVANT TO TODAY'S GOAL]

```rust
// kernel/src/main.rs
[paste file content here]
```

---

## Rules for this session

- You are the engineer, I am the architect. Explain what you build.
- One goal only. Don't expand scope unless I ask.
- If something is unclear or has multiple valid approaches, ask me before deciding.
- Keep everything minimal. No over-engineering.
- After we finish, remind me to update DEVLOG.md.

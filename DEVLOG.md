# Aether — Dev Log

> *"The machine that knows itself."*
> AI-native OS in Rust. VexFS as default filesystem. Jarvis as kernel-level intelligence.

---

## 2026-05-17

- Initialized project vision: AI-native OS, Jarvis as first-class kernel subsystem, VexFS as default fs
- Decided on Linux fork approach for hardware/driver layer, custom compositor forked from Hyprland
- Visual language defined: minimal + sharp + fluid + organic. Dark bg, purple AI accents, color = meaning
- Scaffolded monorepo: kernel/, vexfs/, ai-core/, compositor/, shell/
- Next: get kernel booting in QEMU, print "Aether" in purple to VGA buffer

---

## 2026-05-17 — Session 2

- Kernel crate fully scaffolded: `no_std` entry point, raw VGA write of `Aether` at (0,0) in color 0x55 (white-on-purple), panic handler, `rust-toolchain.toml` pinned to nightly, `.cargo/config.toml` with `lld` + `bootimage runner`, repo pushed to GitHub
- Next: `cd kernel && cargo run` in WSL — see `Aether` on the QEMU screen

---

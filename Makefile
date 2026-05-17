# Aether build automation
# ─────────────────────────────────────────────────
# Run from the repo root (WSL):
#   make build    → compile kernel + create bootimage
#   make run      → build + launch QEMU in terminal (curses VGA display)
#   make serial   → build + launch QEMU with serial output only (no VGA)
#   make to-wsl   → sync Windows → WSL native dir (faster builds)
#   make to-win   → sync WSL native dir → Windows

CARGO      := $${HOME}/.cargo/bin/cargo
KERNEL_PKG := aether-kernel
WSL_DIR    := $(HOME)/aether
WIN_DIR    := /mnt/c/Users/sharm/Desktop/Aether
BOOTIMAGE  := $(WIN_DIR)/target/x86_64-unknown-none/debug/bootimage-aether-kernel.bin

.PHONY: build run serial to-wsl to-win clean

build:
	$(CARGO) bootimage --manifest-path kernel/Cargo.toml

# Full VGA text-mode display rendered inside your WSL terminal
run: build
	qemu-system-x86_64 \
		-drive format=raw,file=$(BOOTIMAGE) \
		-display curses

# Serial-only mode — output goes to stdout, no display window needed
serial: build
	qemu-system-x86_64 \
		-drive format=raw,file=$(BOOTIMAGE) \
		-display none \
		-serial stdio

to-wsl:
	rsync -av --delete \
		--exclude='target/' --exclude='.git/' \
		--exclude='*.bin'   --exclude='*.img' \
		$(WIN_DIR)/ $(WSL_DIR)/
	@echo "✅ Synced to WSL native: $(WSL_DIR)"

to-win:
	rsync -av --delete \
		--exclude='target/' --exclude='.git/' \
		--exclude='*.bin'   --exclude='*.img' \
		$(WSL_DIR)/ $(WIN_DIR)/
	@echo "✅ Synced to Windows: $(WIN_DIR)"

clean:
	$(CARGO) clean

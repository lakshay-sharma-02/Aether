# Aether build automation
# ─────────────────────────────────────────────────
# Run these from the repo root:
#   make run       → build kernel + launch QEMU (Windows GUI)
#   make build     → build kernel only
#   make headless  → build kernel + launch QEMU in terminal (no GUI)
#   make to-wsl    → sync Windows → WSL native dir (faster builds)
#   make to-win    → sync WSL native dir → Windows

CARGO       := cargo
KERNEL_PKG  := aether-kernel
WSL_DIR     := $(HOME)/aether
WIN_DIR     := /mnt/c/Users/sharm/Desktop/Aether
BOOTIMAGE   := $(WIN_DIR)/target/x86_64-unknown-none/debug/bootimage-aether-kernel.bin

.PHONY: build run headless to-wsl to-win clean

build:
	$(CARGO) build -p $(KERNEL_PKG)

run: build
	qemu-system-x86_64.exe -drive format=raw,file=$(BOOTIMAGE)

headless: build
	qemu-system-x86_64 \
		-drive format=raw,file=$(BOOTIMAGE) \
		-display curses

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

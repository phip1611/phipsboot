# Flags for the binary and all transitive compilation units (such as libcore).
ROOT = $(PWD)
BUILD_DIR = $(ROOT)/build
CARGO_BUILD_STD_FLAGS = -Z build-std=core,alloc,compiler_builtins -Z build-std-features=compiler-builtins-mem

PHIPSBOOT_SRC = $(ROOT)/phipsboot
PHIPSBOOT_RUSTFLAGS = -C relocation-model=static -C target-cpu=x86-64
PHIPSBOOT_RUST_TARGET = x86_64-unknown-none-static
PHIPSBOOT_RUST_TARGET_FILE = $(PHIPSBOOT_SRC)/bin/$(PHIPSBOOT_RUST_TARGET).json
PHIPSBOOT_CARGO_FLAGS = --verbose --target $(PHIPSBOOT_RUST_TARGET_FILE) $(CARGO_BUILD_STD_FLAGS)
PHIPSBOOT_CARGO_ARTIFACT = $(PHIPSBOOT_SRC)/target/$(PHIPSBOOT_RUST_TARGET)/release/phipsboot

.PHONY: default
default: phipsboot

.PHONY: build_dir
build_dir:
	mkdir -p ./build

.PHONY: phipsboot
phipsboot: build_dir phipsboot_cargo
	objcopy -O elf32-i386 $(PHIPSBOOT_CARGO_ARTIFACT) $(BUILD_DIR)/phipsboot.elf32
	cp $(PHIPSBOOT_CARGO_ARTIFACT) $(BUILD_DIR)/phipsboot.elf64

.PHONY: phipsboot_cargo
phipsboot_cargo:
	cd phipsboot && RUSTFLAGS="$(PHIPSBOOT_RUSTFLAGS)" cargo build $(PHIPSBOOT_CARGO_FLAGS)
	cd phipsboot && RUSTFLAGS="$(PHIPSBOOT_RUSTFLAGS)" cargo build $(PHIPSBOOT_CARGO_FLAGS) --release
	grub-file --is-x86-multiboot2 "$(PHIPSBOOT_CARGO_ARTIFACT)"

.PHONY: test
test:
	cd phipsboot && cargo nextest run --lib

.PHONY: integration-test
integration-test: phipsboot
	cd integration-test && ./build_bootable_img.sh --phipsboot="$(BUILD_DIR)/phipsboot.elf64" --out-path="$(BUILD_DIR)"
	cd integration-test && ./run_iso_in_qemu.sh --iso="$(BUILD_DIR)/phipsboot.grub-mb2.iso"


.PHONY: clean
clean:
	cd phipsboot && cargo clean
	rm -rf ./build

export PROJECT_ROOT      	= $(PWD)
export CARGO_TARGET_DIR     = $(PWD)/target

# release|debug
export RELEASE             ?= release
export COMPILE_TARGET       = x86_64-unknown-none
export CHAINLOADER_ARTIFACT = $(CARGO_TARGET_DIR)/${COMPILE_TARGET}/$(RELEASE)/bin

export CARGO_BIN_FLAGS = --target $(COMPILE_TARGET)

.PHONY: default
default: bin

.PHONY: bin
bin:
	cd chainloader && cargo build $(CARGO_BIN_FLAGS)
	cd chainloader && cargo build --release $(CARGO_BIN_FLAGS)
	grub-file --is-x86-multiboot2 "$(CHAINLOADER_ARTIFACT)"
	#objcopy -O elf32-i386 "$(BIN_ARTIFACT)" "$(BIN_ARTIFACT)_elf32"

.PHONY: test
test:
	cd chainloader && cargo nextest run --lib

.PHONY: integration-test
integration-test: bin
	cd integration-test && ./build_grub_img.sh && ./run_qemu.sh

.PHONY: clean
clean:
	cd chainloader && cargo clean
	rm -rf integration-test/.vol

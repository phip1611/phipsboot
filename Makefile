export PROJECT_ROOT      	= $(PWD)
# release|debug
export RELEASE             ?= release
export CHAINLOADER_ARTIFACT = $(PROJECT_ROOT)/chainloader/target/x86_64-unknown-none/$(RELEASE)/bin

export CARGO_BIN_FLAGS = --target x86_64-unknown-none

.PHONY: default
default: bin

.PHONY: bin
bin:
	cd chainloader && cargo build $(CARGO_BIN_FLAGS)
	cd chainloader && cargo build --release $(CARGO_BIN_FLAGS)
	grub-file --is-x86-multiboot2 "$(CHAINLOADER_ARTIFACT)"

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

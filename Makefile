export PROJECT_ROOT      	= $(PWD)
export CHAINLOADER_ARTIFACT = $(PROJECT_ROOT)/chainloader/target/x86_64-unknown-none/release/bin

.PHONY: default
default: bin

.PHONY: bin
bin:
	cd chainloader && cargo build --release --target x86_64-unknown-none.json -Z build-std=core,alloc,compiler_builtins -Z build-std-features=compiler-builtins-mem
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

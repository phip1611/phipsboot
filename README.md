Context: <https://github.com/rust-lang/rust/issues/113207>

If you have nix installed, running everything should be as easy as:

`$ nix-shell --run "make && make integration-test"`. The Rust toolchain is
pinned using a rustup-toolchain.toml file.

This starts my loader (=the rust binary I build) in a QEMU VM. GRUB2 chainloads the loader via multiboot2. The Rust binary is supposed to have only position-independent code and is linked at 2M. GRUB relocates the binary to 6M at runtime in physical memory.

To check if you have the right binary after running make, here's the sha256 hash:

```
$ sha256sum chainloader/target/x86-unknown-none/release/bin
a2881d9545a1a969b278cc663fb7225b1f04a49539cf49195f809df4866b3ad1  chainloader/target/x86-unknown-none/release/bin
```

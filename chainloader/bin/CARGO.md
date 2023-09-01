# Cargo/Rust Build Overview

This Cargo project produces a Rust binary, i.e., the loader binary. There are
a few challenges that need to be solved in a non-standard Cargo way: By default,
the built-in target `x86_64-unknown-none` uses the `kernel` code model of the
x86_64 SystemV ABI specification. `libcore` is compiled with this code model.
The loader's high-level code however is supposed to be linked at 4G and not at
`0xffffffff82000000` or so. As I can't link the code at 4G with the standard
`x86_64-unknown-none` target and it's pre-compiled libcore, I have to use a
custom target specification including a cross-compilation `libcore`, `liballoc`,
etc.

The compiler target is mostly identical to the `x86_64-unknown-none`, except
that no relocation information is used and code use the `static` relocation
model of Rust. This works, as the position-independent boot-code written in
assembly sets up paging for the actual Rust code.


Version of the default `x86_64-unknown-none` target that uses a static
relocation model for the kernel, unlike `x86_64-unknown-none`. As we need the
same compilation options for `libcore` and `liballoc`, we cross-compile the
(relevant parts) the standard library with that target as well.

Otherwise, we would have an object file with relocation information coming from
the pre-compiled `libcore` and `liballoc`, which only causes
[weird behaviour](https://github.com/rust-lang/rust/issues/114767).

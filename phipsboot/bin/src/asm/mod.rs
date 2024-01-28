//! Module that combines all assembly files.
//!
//! # Syntax
//! All files use GNU as-style assembly with AT&T syntax. Behind the scenes,
//! LLVM's internal assembler is used:
//! - https://llvm.org/doxygen/AsmParser_8cpp_source.html
//!
//! # Style
//! All files follow the GNU assembler style with the AT&T syntax. Add two
//! spaces after the instruction mnemonic or a macro invocation. All further
//! registers or parameters are only deviced by one single space. Use parameter
//! alignment across lines only where really necessary/benefitial. Macros are
//! prefixed with `M_`.

core::arch::global_asm!(include_str!("vars.S"), options(att_syntax));
core::arch::global_asm!(include_str!("macros.S"), options(att_syntax));
core::arch::global_asm!(include_str!("start.S"), options(att_syntax));
core::arch::global_asm!(include_str!("multiboot2_header.S"), options(att_syntax));

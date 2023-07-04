core::arch::global_asm!(include_str!("macros.S"), options(att_syntax));
core::arch::global_asm!(include_str!("start.S"), options(att_syntax));
core::arch::global_asm!(include_str!("multiboot2_header.S"), options(att_syntax));

use core::fmt::Write;

const QEMU_DEBUGCON_PORT: u16 = 0xe9;

#[derive(Debug, Default)]
pub struct DebugconLogger;

impl DebugconLogger {
    fn print_char(&self, c: u8) {
        unsafe { x86::io::outb(QEMU_DEBUGCON_PORT, c) }
    }
}

impl Write for DebugconLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.as_bytes() {
            self.print_char(*byte);
        }
        Ok(())
    }
}

impl lib::logger::Backend for DebugconLogger {
    fn name(&self) -> &str {
        "debugcon"
    }
}

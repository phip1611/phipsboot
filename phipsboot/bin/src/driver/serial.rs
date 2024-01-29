use core::fmt::Write;


#[derive(Debug)]
pub struct SerialLogger(uart_16550::SerialPort);

impl Default for SerialLogger {
    fn default() -> Self {
        let mut port = unsafe { uart_16550::SerialPort::new(0x3f8) };
        port.init();
        Self(port)
    }
}

impl Write for SerialLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.write_str(s)
    }
}

impl lib::logger::Backend for SerialLogger {
    fn name(&self) -> &str {
        "serial"
    }
}

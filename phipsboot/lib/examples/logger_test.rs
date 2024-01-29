use lib::logger::Backend;
use std::fmt::Write;

#[derive(Debug)]
struct StdoutBackend;

impl Write for StdoutBackend {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        print!("{}", s);
        Ok(())
    }
}

impl Backend for StdoutBackend {
    fn name(&self) -> &str {
        "stdout"
    }
}

fn main() {
    lib::logger::init();
    log::debug!("called init()");
    lib::logger::add_backend(StdoutBackend).unwrap();
    log::info!("added backend init()");
    lib::logger::flush();
    log::warn!("flushed");
}

//! This implements a logger (see [`LoggerFacade`]) that can buffer messages on
//! the heap as long as
//! 1) no backends are specified and
//! 2) the logger was not initially flushed.
//!
//! The whole code is relatively complex for the following reasons:
//! - A static is needed for the `log` crate
//! - This static needs internal mutable state
//! - Logging backends can be added dynamically
//! - I want to avoid unnecessary heap allocations for formatting, and
//! - I want to use as much of the `core::fmt`-facilities as possible.
//!
//! Therefore, we need multiple levels and intermediate types to achieve that
//! goal.

use crate::safe::Safe;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Debug, Write};
pub use log;
use log::{LevelFilter, Log, Metadata, Record};

/// Logger instance with `static` lifetime for the [`log::set_logger`]
/// interface.
static LOGGER: LoggerFacade = LoggerFacade::new();

/// Initializes the logger. At the beginning, log messages are buffered in a .
/// vector.
pub fn init() {
    log::set_max_level(LevelFilter::Trace);
    log::set_logger(&LOGGER).unwrap();
}

/// The provided backend is already specified.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct BackendAlreadySpecifiedError<B: Backend>(B);

/// Actually formats a message and writes it to the corresponding destination
/// specified by [`Write`].
fn format_and_write_log_msg(writer: &mut dyn Write, record: &Record) -> core::fmt::Result {
    writeln!(
        writer,
        "[{:>5} {}@{}]: {}",
        record.level(),
        record.file().unwrap_or("<unknown>"),
        record.line().unwrap_or(0),
        record.args()
    )
}

/// Static-safe type and [`log::Log`]-compatible logger that supports multiple
/// backends. As long as no backends are specified, log messages are buffered
/// internally. An user MUST invoke [`LoggerFacade::flush`] once after all
/// desired backends were specified via [`LoggerFacade::add_backend`]. After
/// that, the buffering is deactivated and all messages are directly written to
/// the corresponding backends.
#[derive(Debug)]
pub struct LoggerFacade(Safe<LoggerFacadeInner> /* mutable inner state */);

impl LoggerFacade {
    pub const fn new() -> Self {
        Self(Safe::new(LoggerFacadeInner::new()))
    }

    /// Adds a [`Backend`] to the logger.
    pub fn add_backend<B: Backend>(
        &self,
        backend: B,
    ) -> Result<(), BackendAlreadySpecifiedError<B>> {
        let backends = &mut self.0.borrow_mut().backends;
        let has_backend = backends.iter().any(|b| b.name() == backend.name());
        if !has_backend {
            backends.push(Box::new(backend));
            Ok(())
        } else {
            Err(BackendAlreadySpecifiedError(backend))
        }
    }

    #[cfg(test)]
    fn buffered_msg_count(&self) -> Option<usize> {
        self.0.borrow().message_buffer.as_ref().map(|v| v.len())
    }

    /// Returns a copy of the buffered messages.
    #[cfg(test)]
    fn buffered_msgs(&self) -> Vec<String> {
        self.0.borrow().message_buffer.as_ref().unwrap().clone()
    }
}

impl log::Log for LoggerFacade {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let inner = &mut *self.0.borrow_mut();
        if let Some(ref mut buffer) = inner.message_buffer {
            let mut writer = String::new();
            let _ = format_and_write_log_msg(&mut writer, record);
            buffer.push(writer);
        } else {
            let mut writer: &mut LoggerFacadeInner = inner;
            let _ = format_and_write_log_msg(&mut writer, record);
        }
    }

    fn flush(&self) {
        let inner: &mut LoggerFacadeInner = &mut *self.0.borrow_mut();
        if let Some(messages) = inner.message_buffer.take() {
            for msg in messages {
                inner.write_to_all_backends(&msg);
            }
        } else {
            log::debug!("flushing multiple times is a no-op for this type");
        }
    }
}

/// Logging backend abstraction for [`LoggerFacade`]. The backend is responsible
/// of sending the formatted messages a corresponding output channel/device.
pub trait Backend: Write /* for write_str*/ + Debug + 'static {
    /// Returns a unique ID of the backend implementation, such as `serial`.
    fn name(&self) -> &str;
}

/// Actual implementation and mutable state of a [`LoggerFacade`]. The type
/// takes care whether new messages are written to all backends or to the buffer
/// in its implementation of [`LoggerFacadeInner::write_str`].
#[derive(Debug, Default)]
struct LoggerFacadeInner {
    /// Logging message buffer.
    message_buffer: Option<Vec<String>>,
    /// Logging backends.
    backends: Vec<Box<dyn Backend>>,
}

impl LoggerFacadeInner {
    const fn new() -> Self {
        Self {
            message_buffer: Some(Vec::new()),
            backends: Vec::new(),
        }
    }

    /// Writes the message synchronously to all backends.
    fn write_to_all_backends(&mut self, msg: &str) {
        for backend in &mut self.backends {
            let _ = backend.write_str(&msg);
        }
    }
}

impl Write for LoggerFacadeInner {
    fn write_str(&mut self, msg: &str) -> core::fmt::Result {
        // note: here, we do not receive full formatted messages but partial
        // messages as intermediate results of the formatting library.
        self.write_to_all_backends(msg);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::rc::Rc;
    use log::Level;
    use std::cell::RefCell;

    #[derive(Debug)]
    struct StdoutBackend;

    impl Write for StdoutBackend {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            std::print!("{s}");
            Ok(())
        }
    }

    impl Backend for StdoutBackend {
        fn name(&self) -> &str {
            "stdout"
        }
    }

    #[derive(Debug, Default)]
    struct BufferingBackend(Rc<RefCell<String>> /* last written log line */);

    impl Write for BufferingBackend {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            // This simplification is only valid for the test where I write
            // exactly one message to it. Due to the nature of `core::fmt`,
            // the string will be builds from components, such as:
            // - `[`
            // - `DEBUG`
            // - ` `
            // - `demo.rs`
            // - `@`
            // - `...`,
            // hence, composed in multiple steps.
            self.0.borrow_mut().push_str(s);
            Ok(())
        }
    }

    impl Backend for BufferingBackend {
        fn name(&self) -> &str {
            "buffering"
        }
    }

    #[test]
    fn add_backend() {
        let logger = LoggerFacade::new();
        assert_eq!(logger.0.borrow().backends.len(), 0);
        logger.add_backend(StdoutBackend).unwrap();
        assert_eq!(logger.0.borrow().backends.len(), 1);
    }

    #[test]
    fn add_same_backend_twice_fails() {
        let logger = LoggerFacade::new();
        logger.add_backend(StdoutBackend).unwrap();
        logger.add_backend(StdoutBackend).unwrap_err();
        assert_eq!(logger.0.borrow().backends.len(), 1);
    }

    #[test]
    fn add_multiple_backends() {
        let logger = LoggerFacade::new();
        logger.add_backend(StdoutBackend).unwrap();
        logger.add_backend(BufferingBackend::default()).unwrap();
        assert_eq!(logger.0.borrow().backends.len(), 2);
    }

    #[test]
    fn log_msgs_are_buffered() {
        let logger = LoggerFacade::new();
        logger.add_backend(StdoutBackend).unwrap();

        assert_eq!(logger.buffered_msg_count(), Some(0));
        logger.log(
            &Record::builder()
                .target("target")
                .level(Level::Debug)
                .line(Some(42))
                .file(Some("demo.rs"))
                .args(format_args!("a={}, b={}", 13, 73))
                .build(),
        );
        assert_eq!(logger.buffered_msg_count(), Some(1));
    }

    #[test]
    fn msgs_are_in_right_format() {
        let logger = LoggerFacade::new();
        logger.add_backend(StdoutBackend).unwrap();

        assert_eq!(logger.buffered_msg_count(), Some(0));
        logger.log(
            &Record::builder()
                .target("target")
                .level(Level::Debug)
                .line(Some(42))
                .file(Some("demo.rs"))
                .args(format_args!("a={}, b={}", 13, 73))
                .build(),
        );
        assert_eq!(
            logger.buffered_msgs()[0],
            "[DEBUG demo.rs@42]: a=13, b=73\n"
        );
    }

    #[test]
    fn log_msgs_arent_buffered_after_initial_flush() {
        let logger = LoggerFacade::new();
        logger.add_backend(StdoutBackend).unwrap();
        logger.flush();

        assert_eq!(logger.buffered_msg_count(), None);
        logger.log(
            &Record::builder()
                .target("target")
                .level(Level::Debug)
                .line(Some(42))
                .file(Some("demo.rs"))
                .args(format_args!("a={}, b={}", 13, 73))
                .build(),
        );
        assert_eq!(logger.buffered_msg_count(), None);
    }

    #[test]
    fn buffered_log_msgs_are_forwarded_to_backend_after_initial_lush() {
        let logger = LoggerFacade::new();
        let backend = BufferingBackend::default();
        let backend_received_buffered_line = backend.0.clone();
        logger.add_backend(backend).unwrap();

        logger.log(
            &Record::builder()
                .target("target")
                .level(Level::Debug)
                .line(Some(42))
                .file(Some("demo.rs"))
                .args(format_args!("a={}, b={}", 13, 73))
                .build(),
        );

        logger.flush();

        let backend_received_buffered_line = backend_received_buffered_line.borrow();
        assert_eq!(
            backend_received_buffered_line.as_str(),
            "[DEBUG demo.rs@42]: a=13, b=73\n"
        );
    }

    #[test]
    fn new_log_msgs_are_forwarded_to_backend_after_initial_flush() {
        let logger = LoggerFacade::new();
        let backend = BufferingBackend::default();
        let backend_received_buffered_line = backend.0.clone();
        logger.add_backend(backend).unwrap();
        logger.flush();

        logger.log(
            &Record::builder()
                .target("target")
                .level(Level::Debug)
                .line(Some(42))
                .file(Some("demo.rs"))
                .args(format_args!("a={}, b={}", 13, 73))
                .build(),
        );

        let backend_received_buffered_line = backend_received_buffered_line.borrow();
        assert_eq!(
            backend_received_buffered_line.as_str(),
            "[DEBUG demo.rs@42]: a=13, b=73\n"
        )
    }
}

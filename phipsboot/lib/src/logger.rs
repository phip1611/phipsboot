//! This implements a logger (see [`LoggerFacade`]) that can buffer messages on
//! the heap as long as
//! 1) no backends are specified and
//! 2) the logger was not initially flushed.
//!
//! The whole code is relatively complex for the following reasons:
//! - A static is needed for the `log` crate
//! - This static needs internal mutable state
//! - Logging backends can be added dynamically (at the beginning, it is unclear
//!   whether serial or other backends are enabled (chicken-egg problem).
//! - I want log messages in the crate from the very begin of the Rust code
//! - I want to avoid unnecessary heap allocations for formatting, and
//! - I want to use as much of the `core::fmt`-facilities as possible.

use crate::safe::Safe;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt::{Debug, Write};
use log::{LevelFilter, Log, Metadata, Record};

/// Logger instance with `static` lifetime for the [`log::set_logger`] interface.
static LOGGER: Safe<RefCell<LoggerFacade>> = Safe::new(RefCell::new(LoggerFacade::new()));

/// Initializes the logger. The logger depends on the heap being available!
///
/// At the beginning, log messages are buffered in a vector. A user must call
/// eventually [`flush`] after adding corresponding
/// [`Backend`]s via [`add_backend`].
pub fn init() {
    log::set_max_level(LevelFilter::Trace);
    log::set_logger(&LOGGER).unwrap();
}

/// Flushes all messages that have been buffered so far. Once this has been
/// called, all further messages are directly send to the backends and are not
/// longer buffered.
pub fn flush() {
    LOGGER.flush();
}

/// Adds a [`Backend`] to the logger. As long as [`flush`] hasn't been called,
/// all new log messages are still buffered.
pub fn add_backend<B: Backend>(backend: B) -> Result<(), BackendAlreadySpecifiedError<B>> {
    LOGGER.borrow_mut().add_backend(backend)
}

/// The provided backend is already specified.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct BackendAlreadySpecifiedError<B: Backend>(B);

/// Actually formats a log message properly and writes it to the corresponding
/// destination specified by [`Write`]. This works completely on the stack.
fn format_and_write_log_msg(writer: &mut dyn Write, record: &Record) -> core::fmt::Result {
    writeln!(
        writer,
        "[{:>5} {}@{:03}]: {}",
        record.level(),
        record.file().unwrap_or("<unknown>"),
        record.line().unwrap_or(0),
        record.args()
    )
}
impl Log for Safe<RefCell<LoggerFacade>> {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        self.borrow_mut().log_or_buffer_record(record)
    }

    fn flush(&self) {
        self.borrow_mut().do_handover_to_backends()
    }
}

/// Logging backend abstraction for [`LoggerFacade`]. The backend is responsible
/// of sending the formatted messages to a corresponding output channel/device.
pub trait Backend: Write /* for write_str*/ + Debug + 'static {
    /// Returns a unique ID of the backend implementation, such as `serial`, to
    /// identify the backend.
    fn name(&self) -> &str;
}

/// Logger implementation that holds the message buffer and all [`Backend`]s.
/// Its implementation of [`LoggerFacade::write_str`] connects the formatting of
/// [`core::fmt`] with the corresponding [`Backend`]s.
#[derive(Debug, Default)]
struct LoggerFacade {
    /// Logging message buffer.
    message_buffer: Option<Vec<String>>,
    /// Logging backends.
    backends: Vec<Box<dyn Backend>>,
}

impl LoggerFacade {
    /// Constructor.
    const fn new() -> Self {
        Self {
            message_buffer: Some(Vec::new()),
            backends: Vec::new(),
        }
    }

    /// Adds a [`Backend`] to the logger.
    fn add_backend<B: Backend>(
        &mut self,
        backend: B,
    ) -> Result<(), BackendAlreadySpecifiedError<B>> {
        let backends = &mut self.backends;
        let has_backend = backends.iter().any(|b| b.name() == backend.name());
        if !has_backend {
            backends.push(Box::new(backend));
            Ok(())
        } else {
            Err(BackendAlreadySpecifiedError(backend))
        }
    }

    /// Depending on the state of the logger, formats a logging message and puts
    /// it into the buffer, or writes it directly to the backends.
    fn log_or_buffer_record(&mut self, record: &Record) {
        if let Some(ref mut buffer) = self.message_buffer {
            let mut writer = String::new();
            let _ = format_and_write_log_msg(&mut writer, record);
            buffer.push(writer);
        } else {
            let _ = format_and_write_log_msg(self, record);
        }
    }

    /// One time operation that performs the handover from the buffering of all
    /// log messages to the actual [`Backend`]s. This also flushes all buffered
    /// messages. Further invocations are no-ops.
    fn do_handover_to_backends(&mut self) {
        if let Some(messages) = self.message_buffer.take() {
            for msg in messages {
                self.write_to_all_backends(&msg);
            }
        } else {
            log::debug!("flushing multiple times is a no-op for this type");
        }
    }

    #[cfg(test)]
    fn buffered_msg_count(&self) -> Option<usize> {
        self.message_buffer.as_ref().map(|v| v.len())
    }

    /// Returns a copy of the buffered messages.
    #[cfg(test)]
    fn buffered_msgs(&self) -> Vec<String> {
        self.message_buffer.as_ref().unwrap().clone()
    }

    /// Writes the message synchronously to all backends.
    fn write_to_all_backends(&mut self, msg: &str) {
        for backend in &mut self.backends {
            // Ignore error. We can't do much about it here anyway.
            let _ = backend.write_str(&msg);
        }
    }
}

impl Write for LoggerFacade {
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
        let mut logger = LoggerFacade::new();
        assert_eq!(logger.backends.len(), 0);
        logger.add_backend(StdoutBackend).unwrap();
        assert_eq!(logger.backends.len(), 1);
    }

    #[test]
    fn add_same_backend_twice_fails() {
        let mut logger = LoggerFacade::new();
        logger.add_backend(StdoutBackend).unwrap();
        logger.add_backend(StdoutBackend).unwrap_err();
        assert_eq!(logger.backends.len(), 1);
    }

    #[test]
    fn add_multiple_backends() {
        let mut logger = LoggerFacade::new();
        logger.add_backend(StdoutBackend).unwrap();
        logger.add_backend(BufferingBackend::default()).unwrap();
        assert_eq!(logger.backends.len(), 2);
    }

    #[test]
    fn log_msgs_are_buffered() {
        let mut logger = LoggerFacade::new();
        logger.add_backend(StdoutBackend).unwrap();

        assert_eq!(logger.buffered_msg_count(), Some(0));
        logger.log_or_buffer_record(
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
        let mut logger = LoggerFacade::new();
        logger.add_backend(StdoutBackend).unwrap();

        assert_eq!(logger.buffered_msg_count(), Some(0));
        logger.log_or_buffer_record(
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
        let mut logger = LoggerFacade::new();
        logger.add_backend(StdoutBackend).unwrap();
        logger.do_handover_to_backends();

        assert_eq!(logger.buffered_msg_count(), None);
        logger.log_or_buffer_record(
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
        let mut logger = LoggerFacade::new();
        let backend = BufferingBackend::default();
        let backend_received_buffered_line = backend.0.clone();
        logger.add_backend(backend).unwrap();

        logger.log_or_buffer_record(
            &Record::builder()
                .target("target")
                .level(Level::Debug)
                .line(Some(42))
                .file(Some("demo.rs"))
                .args(format_args!("a={}, b={}", 13, 73))
                .build(),
        );

        logger.do_handover_to_backends();

        let backend_received_buffered_line = backend_received_buffered_line.borrow();
        assert_eq!(
            backend_received_buffered_line.as_str(),
            "[DEBUG demo.rs@42]: a=13, b=73\n"
        );
    }

    #[test]
    fn new_log_msgs_are_forwarded_to_backend_after_initial_flush() {
        let mut logger = LoggerFacade::new();
        let backend = BufferingBackend::default();
        let backend_received_buffered_line = backend.0.clone();
        logger.add_backend(backend).unwrap();
        logger.do_handover_to_backends();

        logger.log_or_buffer_record(
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

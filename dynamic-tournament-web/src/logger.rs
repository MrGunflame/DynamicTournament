use gloo_console::{debug, error, info, trace, warn};
use log::{set_logger_racy, Level, Log, Metadata, Record};

/// Initializes the logger.
///
/// # Safety
///
/// This function is only safe to call when there are no other threads calling it
/// at the same time.
#[inline]
pub unsafe fn init() {
    let _ = set_logger_racy(&Logger);
}

#[derive(Copy, Clone, Debug)]
pub struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let message = record.args().to_string();

        match record.level() {
            Level::Error => error!(message),
            Level::Warn => warn!(message),
            Level::Info => info!(message),
            Level::Debug => debug!(message),
            Level::Trace => trace!(message),
        }
    }

    fn flush(&self) {}
}

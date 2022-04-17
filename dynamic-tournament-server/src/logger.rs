use chrono::Local;
use log::{set_logger, set_max_level, Level, LevelFilter, Log, Metadata, Record};

pub fn init(level: LevelFilter) {
    set_logger(&Logger).unwrap();
    set_max_level(level);
}

#[derive(Copy, Clone, Debug)]
pub struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S");

        let level = match record.level() {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        };

        println!(
            "[{}] [{}:{}] [{}] {}",
            now,
            record.file().unwrap_or("???"),
            record.line().unwrap_or(0),
            level,
            record.args()
        );
    }

    fn flush(&self) {}
}

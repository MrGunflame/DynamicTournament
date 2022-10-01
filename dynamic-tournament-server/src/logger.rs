use std::io::{self, ErrorKind, Write};

use chrono::Local;
use log::{set_logger, set_max_level, Level, Log, Metadata, Record};

use crate::config::Log as Config;

const ERROR: &str = "ERROR";
const WARN: &str = "WARN";
const INFO: &str = "INFO";
const DEBUG: &str = "DEBUG";
const TRACE: &str = "TRACE";

const ERROR_COLOR: &str = "\x1b[31mERROR\x1b[0m";
const WARN_COLOR: &str = "\x1b[33mWARN\x1b[0m";
const INFO_COLOR: &str = INFO;
const DEBUG_COLOR: &str = "\x1b[90mDEBUG\x1b[0m";
const TRACE_COLOR: &str = "\x1b[90mTRACE\x1b[0m";

pub fn init(config: Config) {
    let _ = set_logger(Box::leak(Box::new(Logger {
        color: config.color,
    })));

    set_max_level(config.level);
}

#[derive(Copy, Clone, Debug)]
pub struct Logger {
    color: bool,
}

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S");

        let level = if self.color {
            match record.level() {
                Level::Error => ERROR_COLOR,
                Level::Warn => WARN_COLOR,
                Level::Info => INFO_COLOR,
                Level::Debug => DEBUG_COLOR,
                Level::Trace => TRACE_COLOR,
            }
        } else {
            match record.level() {
                Level::Error => ERROR,
                Level::Warn => WARN,
                Level::Info => INFO,
                Level::Debug => DEBUG,
                Level::Trace => TRACE,
            }
        };

        let mut stdout = io::stdout();

        let res = writeln!(
            stdout,
            "[{}] [{}] [{}:{}] {}",
            now,
            level,
            record.file().unwrap_or("???"),
            record.line().unwrap_or(0),
            record.args()
        );

        drop(stdout);

        if let Err(err) = res {
            if err.kind() != ErrorKind::BrokenPipe {
                panic!("Failed to write to stdout: {}", err);
            }
        }
    }

    fn flush(&self) {}
}

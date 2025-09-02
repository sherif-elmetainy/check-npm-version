use std::sync::atomic::{AtomicU8, Ordering};
use clap::ValueEnum;

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn to_u8(self) -> u8 {
        match self {
            LogLevel::Off => 0,
            LogLevel::Error => 1,
            LogLevel::Warn => 2,
            LogLevel::Info => 3,
            LogLevel::Debug => 4,
            LogLevel::Trace => 5,
        }
    }

    fn from_u8(v: u8) -> LogLevel {
        match v {
            0 => LogLevel::Off,
            1 => LogLevel::Error,
            2 => LogLevel::Warn,
            3 => LogLevel::Info,
            4 => LogLevel::Debug,
            5 => LogLevel::Trace,
            _ => LogLevel::Warn,
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Warn
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Off => write!(f, "off"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Trace => write!(f, "trace"),
        }
    }
}

static GLOBAL_LOG_LEVEL: AtomicU8 = AtomicU8::new(2); // default: Warn

pub fn set_log_level(level: LogLevel) {
    GLOBAL_LOG_LEVEL.store(level.to_u8(), Ordering::Relaxed);
}

pub fn get_log_level() -> LogLevel {
    LogLevel::from_u8(GLOBAL_LOG_LEVEL.load(Ordering::Relaxed))
}

pub fn log_enabled(level: LogLevel) -> bool {
    level.to_u8() <= GLOBAL_LOG_LEVEL.load(Ordering::Relaxed)
}

// pub fn log(level: LogLevel, msg: &str) {
//     if log_enabled(level) {
//         println!("{}", msg);
//     }
// }

// Macro exports
#[macro_export]
macro_rules! log_at {
    ($level:expr, $($arg:tt)*) => {{
        if $crate::util::log_enabled($level) {
            println!("{}", format!($($arg)*));
        }
    }};
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::log_at!($crate::util::LogLevel::Error, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::log_at!($crate::util::LogLevel::Warn, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::log_at!($crate::util::LogLevel::Info, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::log_at!($crate::util::LogLevel::Debug, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        $crate::log_at!($crate::util::LogLevel::Trace, $($arg)*)
    };
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    None,
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        if $crate::config::CONFIG.get_or_init($crate::config::Config::new).log_level <= $crate::logger::LogLevel::Trace {
            println!("[PID: {}] {}", std::process::id(), format_args!($($arg)*));
        }
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if $crate::config::CONFIG.get_or_init($crate::config::Config::new).log_level <= $crate::logger::LogLevel::Debug {
            println!("[PID: {}] {}", std::process::id(), format_args!($($arg)*));
        }
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        if $crate::config::CONFIG.get_or_init($crate::config::Config::new).log_level <= $crate::logger::LogLevel::Info {
            println!("[PID: {}] {}", std::process::id(), format_args!($($arg)*));
        }
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        if $crate::config::CONFIG.get_or_init($crate::config::Config::new).log_level <= $crate::logger::LogLevel::Warn {
            println!("[PID: {}] {}", std::process::id(), format_args!($($arg)*));
        }
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        if $crate::config::CONFIG.get_or_init($crate::config::Config::new).log_level <= $crate::logger::LogLevel::Error {
            eprintln!("[PID: {}] {}", std::process::id(), format_args!($($arg)*));
        }
    }
}

// alias for info
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::info!($($arg)*)
    }
}

// alias for error
#[macro_export]
macro_rules! elog {
    ($($arg:tt)*) => {
        $crate::error!($($arg)*)
    }
}

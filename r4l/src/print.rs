// SPDX-License-Identifier: GPL-2.0

//! Defines the R4L print.
//!
//!
//!Every OS should provide log_print! macro
//!
//!
/// Log Level
pub enum LogLevel {
    /// The "EMERG" level
    Emerge,
    /// The "Alert" level
    Alert,
    /// The "Crit" level
    Crit,
    /// The "error" level
    Error,
    /// The "warn" level.
    Warn,
    /// The "info" level.
    Info,
    /// The "debug" level.
    Debug,
    /// The "debug" level.
    Cont,
}

#[cfg(feature = "starry")] 
pub mod log {
    pub use axlog;
    #[doc(hidden)]
    #[allow(clippy::crate_in_macro_def)]
    #[macro_export]
    macro_rules! log_print (
        ($crate::print::LogLevel::Emerge, $($arg:tt)*) => (
            $crate::print::log::axlog::error!($($arg)*)
        );
        ($crate::print::LogLevel::Alert, $($arg:tt)*) => (
            $crate::axlog::error!($($arg)*)
        );
        ($crate::print::LogLevel::Crit, $($arg:tt)*) => (
            $crate::axlog::error!($($arg)*)
        );
        ($crate::print::LogLevel::Error, $($arg:tt)*) => (
            $crate::axlog::error!($($arg)*)
        );
        ($crate::print::LogLevel::Warn, $($arg:tt)*) => (
            $crate::axlog::warn!($($arg)*)
        );
        ($crate::print::LogLevel::Info, $($arg:tt)*) => (
            $crate::print::log::axlog::info!($($arg)*)
        );
        ($crate::print::LogLevel::Debug, $($arg:tt)*) => (
            $crate::axlog::debug!($($arg)*)
        );
        ($crate::print::LogLevel::Cont, $($arg:tt)*) => (
            $crate::axlog::info!($($arg)*)
        );
    );
}

/// Prints an emergency-level message (level 0).
///
/// Use this level if the system is unusable.
///
/// Equivalent to the kernel's [`pr_emerg`] macro.
///
/// Mimics the interface of [`std::print!`]. See [`core::fmt`] and
/// `alloc::format!` for information about the formatting syntax.
///
/// [`pr_emerg`]: https://www.kernel.org/doc/html/latest/core-api/printk-basics.html#c.pr_emerg
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// pr_emerg!("hello {}\n", "there");
/// ```
#[macro_export]
macro_rules! pr_emerg (
    ($($arg:tt)*) => (
        $crate::log_print!($crate::print::LogLevel::Emerge, $($arg)*)
    )
);

/// Prints an alert-level message (level 1).
///
/// Use this level if action must be taken immediately.
///
/// Equivalent to the kernel's [`pr_alert`] macro.
///
/// Mimics the interface of [`std::print!`]. See [`core::fmt`] and
/// `alloc::format!` for information about the formatting syntax.
///
/// [`pr_alert`]: https://www.kernel.org/doc/html/latest/core-api/printk-basics.html#c.pr_alert
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// pr_alert!("hello {}\n", "there");
/// ```
#[macro_export]
macro_rules! pr_alert (
    ($($arg:tt)*) => (
        $crate::log_print!($crate::print::LogLevel::Alert, $($arg)*)
    )
);

/// Prints a critical-level message (level 2).
///
/// Use this level for critical conditions.
///
/// Equivalent to the kernel's [`pr_crit`] macro.
///
/// Mimics the interface of [`std::print!`]. See [`core::fmt`] and
/// `alloc::format!` for information about the formatting syntax.
///
/// [`pr_crit`]: https://www.kernel.org/doc/html/latest/core-api/printk-basics.html#c.pr_crit
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// pr_crit!("hello {}\n", "there");
/// ```
#[macro_export]
macro_rules! pr_crit (
    ($($arg:tt)*) => (
        $crate::log_print!($crate::print::LogLevel::Crit, $($arg)*)
    )
);

/// Prints an error-level message (level 3).
///
/// Use this level for error conditions.
///
/// Equivalent to the kernel's [`pr_err`] macro.
///
/// Mimics the interface of [`std::print!`]. See [`core::fmt`] and
/// `alloc::format!` for information about the formatting syntax.
///
/// [`pr_err`]: https://www.kernel.org/doc/html/latest/core-api/printk-basics.html#c.pr_err
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// pr_err!("hello {}\n", "there");
/// ```
#[macro_export]
macro_rules! pr_err (
    ($($arg:tt)*) => (
        $crate::log_print!($crate::print::LogLevel::Error, $($arg)*)
    )
);

/// Prints a warning-level message (level 4).
///
/// Use this level for warning conditions.
///
/// Equivalent to the kernel's [`pr_warn`] macro.
///
/// Mimics the interface of [`std::print!`]. See [`core::fmt`] and
/// `alloc::format!` for information about the formatting syntax.
///
/// [`pr_warn`]: https://www.kernel.org/doc/html/latest/core-api/printk-basics.html#c.pr_warn
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// pr_warn!("hello {}\n", "there");
/// ```
#[macro_export]
macro_rules! pr_warn (
    ($($arg:tt)*) => (
        $crate::log_print!($crate::print::LogLevel::Warn, $($arg)*)
    )
);

/// Prints a notice-level message (level 5).
///
/// Use this level for normal but significant conditions.
///
/// Equivalent to the kernel's [`pr_notice`] macro.
///
/// Mimics the interface of [`std::print!`]. See [`core::fmt`] and
/// `alloc::format!` for information about the formatting syntax.
///
/// [`pr_notice`]: https://www.kernel.org/doc/html/latest/core-api/printk-basics.html#c.pr_notice
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// pr_notice!("hello {}\n", "there");
/// ```
#[macro_export]
macro_rules! pr_notice (
    ($($arg:tt)*) => (
        $crate::log_print!($crate::print::LogLevel::Notice, $($arg)*)
    )
);

/// Prints an info-level message (level 6).
///
/// Use this level for informational messages.
///
/// Equivalent to the kernel's [`pr_info`] macro.
///
/// Mimics the interface of [`std::print!`]. See [`core::fmt`] and
/// `alloc::format!` for information about the formatting syntax.
///
/// [`pr_info`]: https://www.kernel.org/doc/html/latest/core-api/printk-basics.html#c.pr_info
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// pr_info!("hello {}\n", "there");
/// ```
#[macro_export]
#[doc(alias = "print")]
macro_rules! pr_info (
    ($($arg:tt)*) => (
        $crate::log_print!($crate::print::LogLevel::Info, $($arg)*)
    )
);

/// Prints a debug-level message (level 7).
///
/// Use this level for debug messages.
///
/// Equivalent to the kernel's [`pr_debug`] macro, except that it doesn't support dynamic debug
/// yet.
///
/// Mimics the interface of [`std::print!`]. See [`core::fmt`] and
/// `alloc::format!` for information about the formatting syntax.
///
/// [`pr_debug`]: https://www.kernel.org/doc/html/latest/core-api/printk-basics.html#c.pr_debug
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// pr_debug!("hello {}\n", "there");
/// ```
#[macro_export]
#[doc(alias = "print")]
macro_rules! pr_debug (
    ($($arg:tt)*) => (
        $crate::log_print!($crate::print::LogLevel::Debug, $($arg)*)
    )
);

/// Continues a previous log message in the same line.
///
/// Use only when continuing a previous `pr_*!` macro (e.g. [`pr_info!`]).
///
/// Equivalent to the kernel's [`pr_cont`] macro.
///
/// Mimics the interface of [`std::print!`]. See [`core::fmt`] and
/// `alloc::format!` for information about the formatting syntax.
///
/// [`pr_info!`]: crate::pr_info!
/// [`pr_cont`]: https://www.kernel.org/doc/html/latest/core-api/printk-basics.html#c.pr_cont
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// # use kernel::pr_cont;
/// pr_info!("hello");
/// pr_cont!(" {}\n", "there");
/// ```
#[macro_export]
macro_rules! pr_cont (
    ($($arg:tt)*) => (
        $crate::log_print!($crate::print::LogLevel::Cont, $($arg)*)
    )
);


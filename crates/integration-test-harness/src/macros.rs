/// Locates the cloudtruth binary to test
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! cli_bin_path {
    () => {
        $crate::command::cli_bin_path(option_env!("CARGO_BIN_NAME").unwrap_or("cloudtruth"))
    };
}

/// Creates a cloudtruth command.
/// Example:
///     cloudtruth!("projects list")
///
/// Also accepts format! syntax:
///     cloudtruth!("projects set {name}", name=expr)
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! cloudtruth {
    ($($fmt:tt)*) => (
        $crate::command::run_cloudtruth_cmd($crate::cli_bin_path!(), format!($($fmt)*)).unwrap()
    )
}

/// Helper macro equivalent to contains(format!(...))
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! contains {
    ($($fmt:tt)*) => (
        $crate::predicates::contains(format!($($fmt)*))
    )
}

/// Helper macro equivalent to diff(format!(...))
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! diff {
    ($($fmt:tt)*) => (
        $crate::predicates::diff(format!($($fmt)*))
    )
}

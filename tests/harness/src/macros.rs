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
        $crate::command::from_cmd_args($crate::cli_bin_path!(), format!($($fmt)*)).unwrap()
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

#[cfg(feature = "macros")]
#[macro_export]
/// Helper macro to and() a list of predicates.
macro_rules! all {
    ($p:expr $(, $ps:expr)* $(,)?) => { $p$(.and($ps))* }
}

/// Checks that variable contains all strings from an iterator
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! contains_all {
    ($e:expr $(, $es:expr)* $(,)?) => ( $crate::all!($crate::predicates::contains($e) $(, $crate::predicates::contains($es))*,) )
}

#[cfg(feature = "macros")]
#[macro_export]
/// Helper macro to or() a list of predicates.
macro_rules! any {
    ($p:expr $(, $ps:expr)* $(,)?) => { $p$(.or($ps))* }
}

/// Checks that variable contains all strings from an iterator
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! contains_any {
    ($e:expr $(, $es:expr)* $(,)?) => ( $crate::any!($crate::predicates::contains($e) $(, $crate::predicates::contains($es))*,) )
}

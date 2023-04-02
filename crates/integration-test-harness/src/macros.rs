#[cfg(feature = "macros")]
#[macro_export]
macro_rules! cli_bin_path {
    () => {
        env!("CARGO_BIN_EXE_cloudtruth")
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
    ($fmt:expr) => ( cloudtruth!($fmt,) );
    ($fmt:expr, $( $id:ident = $value:expr ),* $(,)*) => (
        $crate::command::commandify(format!(concat!("{} ", $fmt), cli_bin_path!(), $( $id = $crate::command::command_arg(&$value) ),*))
            .wrap_err_with(|| format!(concat!("Invalid command: cloudtruth ", $fmt), $( $id = $crate::command::command_arg(&$value) ),*))
            .unwrap()
    );
}

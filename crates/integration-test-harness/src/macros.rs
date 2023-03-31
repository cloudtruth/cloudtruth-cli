#[macro_export]
macro_rules! cloudtruth_bin_path {
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
#[macro_export]
macro_rules! cloudtruth {
    ($fmt:expr) => ( cloudtruth!($fmt,) );
    ($fmt:expr, $( $id:ident = $value:expr ),* $(,)*) => (
        commandspec::command!(concat!(cloudtruth_bin_path!(), " ", $fmt), $( $id = $crate::command_arg(&$value) ),*)
            .map($crate::Command::from_std)
            .map_err(|e| e.compat())
            .into_diagnostic()
            .wrap_err_with(|| format!(concat!("Invalid command: cloudtruth ", $fmt), $( $id = $crate::command_arg(&$value) ),*))
            .unwrap()
    );
}

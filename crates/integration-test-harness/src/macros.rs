/// Locates the cloudtruth binary to test
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! bin_path {
    () => {
        std::env::var("NEXTEST_BIN_EXE_cloudtruth")
            .ok()
            .or(option_env!("CARGO_BIN_EXE_cloudtruth").map(String::from))
            .or_else(|| {
                option_env!("CARGO_BIN_NAME").and_then(|name| {
                    std::env::current_exe()
                        .ok()
                        .map(|mut path| {
                            path.pop();
                            if path.ends_with("deps") {
                                path.pop();
                            }
                            path
                        })
                        .unwrap()
                        .join(format!("{}{}", name, std::env::consts::EXE_SUFFIX))
                        .into_os_string()
                        .into_string()
                        .ok()
                })
            })
            .expect("Could not find cloudtruth binary")
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
        $crate::command::run_cloudtruth_cmd(bin_path!(), format!($($fmt)*)).unwrap()
    )
}

/// Helper macro equivalent to contains(format!(...))
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! contains {
    ($($fmt:tt)*) => (
        contains(format!($($fmt)*))
    )
}

/// Helper macro equivalent to diff(format!(...))
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! diff {
    ($($fmt:tt)*) => (
        contains(format!($($fmt)*))
    )
}

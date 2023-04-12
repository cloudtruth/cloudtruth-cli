#[cfg(not(target_os = "windows"))]
pub const DISPLAY_ENV_CMD: &str = "printenv";
#[cfg(target_os = "windows")]
pub const DISPLAY_ENV_CMD: &str = "SET";

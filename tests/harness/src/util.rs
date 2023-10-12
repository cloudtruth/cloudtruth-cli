use core::time;
use std::{
    process::{Command, ExitStatus},
    thread,
};

/// Platform-specific shell command used to print environment variables with `cloudtruth run`
#[cfg(not(target_os = "windows"))]
pub const DISPLAY_ENV_CMD: &str = "printenv";
#[cfg(target_os = "windows")]
pub const DISPLAY_ENV_CMD: &str = "SET";

pub fn retry_cmd_with_backoff(cmd: &mut Command) -> std::io::Result<ExitStatus> {
    const MAX_FAILURES: u32 = 3;
    const BASE_DELAY_SECS: u64 = 2;
    let mut failure_count: u32 = 0;
    let mut result;
    loop {
        result = cmd.spawn().and_then(|mut proc| proc.wait());
        match result {
            Ok(status) if status.success() => break,
            _ => {
                failure_count += 1;
                if failure_count >= MAX_FAILURES {
                    break;
                }
                let delay_secs = BASE_DELAY_SECS.pow(failure_count);
                thread::sleep(time::Duration::from_secs(delay_secs))
            }
        }
    }
    result
}

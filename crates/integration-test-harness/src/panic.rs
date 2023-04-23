use std::panic::Location;

/// Custom panic handler
/// This code is heavily based on miette::set_panic_hook (https://github.com/zkat/miette/blob/main/src/panic.rs)
use crate::backtrace;
use thiserror::Error;

use miette::{Context, Diagnostic, Result};

use crate::source_span::TestSourceSpan;

const HELP_TEXT: &str = "set the `RUST_BACKTRACE=1` environment variable to display a backtrace.";

#[track_caller]
pub fn set_panic_hook() {
    set_panic_hook_with_caller(Location::caller())
}

pub fn set_panic_hook_with_caller(caller: &'static Location) {
    std::panic::set_hook(Box::new(move |info| {
        let payload = info.payload();
        let message = if let Some(msg) = payload.downcast_ref::<&str>() {
            msg.to_string()
        } else if let Some(msg) = payload.downcast_ref::<String>() {
            msg.to_string()
        } else {
            "Something went wrong".to_string()
        };

        let mut report: Result<()> = Err(PanicReport::new(message).into());
        if let Some(loc) = info.location() {
            report = report.with_context(|| {
                format!("Panic at {}:{}:{}", loc.file(), loc.line(), loc.column())
            });
        }
        if let Ok(Some(mut src_span)) = TestSourceSpan::from_backtrace(caller) {
            src_span.add_related(report.unwrap_err());
            report = Err(src_span.into());
        }
        eprintln!("{:?}", report.unwrap_err());
    }));
}

#[derive(Debug, Error, Diagnostic)]
#[error("{message}{}", .backtrace.clone().unwrap_or_default())]
#[diagnostic()]
pub struct PanicReport {
    message: String,
    #[help]
    help: Option<String>,
    backtrace: Option<String>,
}

impl PanicReport {
    pub fn new(message: String) -> Self {
        let (help, backtrace) = if backtrace::is_rust_backtrace_enabled() {
            (None, Some(backtrace::format_backtrace()))
        } else {
            (Some(HELP_TEXT.into()), None)
        };
        PanicReport {
            message,
            help,
            backtrace,
        }
    }
}

use std::panic::Location;

/// Custom panic handler
/// This code is heavily based on miette::set_panic_hook (https://github.com/zkat/miette/blob/main/src/panic.rs)
use crate::backtrace;
use thiserror::Error;

use miette::{Diagnostic, Report, Result};

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
        let panic = Panic::new(message);
        let mut report: Result<(), Report> = if let Some(loc) = info.location() {
            Err(PanicLocation::new(panic, loc).into())
        } else {
            Err(panic.into())
        };
        if let Ok(Some(mut src_span)) = TestSourceSpan::from_backtrace(caller) {
            src_span.add_related(report.unwrap_err());
            report = Err(src_span.into());
        }
        eprintln!("{:?}", report.unwrap_err());
    }));
}

#[derive(Clone, Debug, Error, Diagnostic)]
#[error("{message}{}", .backtrace.clone().unwrap_or_default())]
#[diagnostic()]
pub struct Panic {
    message: String,
    backtrace: Option<String>,
    #[help]
    help: Option<&'static str>,
}

impl Panic {
    pub fn new(message: String) -> Self {
        let (help, backtrace) = if backtrace::is_rust_backtrace_enabled() {
            (None, Some(backtrace::format_backtrace()))
        } else {
            (Some(HELP_TEXT), None)
        };
        Self {
            message,
            help,
            backtrace,
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Panic at {}:{}:{}", filename, line, col)]
#[diagnostic()]
pub struct PanicLocation {
    #[source]
    #[diagnostic_source]
    panic: Panic,
    filename: String,
    line: u32,
    col: u32,
    #[help]
    help: Option<&'static str>,
}

impl PanicLocation {
    fn new(panic: Panic, location: &Location) -> Self {
        Self {
            panic,
            filename: location.file().to_string(),
            line: location.line(),
            col: location.column(),
            help: if backtrace::is_rust_backtrace_enabled() {
                None
            } else {
                Some(HELP_TEXT)
            },
        }
    }
}

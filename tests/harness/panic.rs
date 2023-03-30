/// Custom panic handler
/// This code is heavily based on miette::set_panic_hook (https://github.com/zkat/miette/blob/main/src/panic.rs)
use std::fmt::Write;

use backtrace::Backtrace;
use thiserror::Error;

use miette::{Context, Diagnostic, GraphicalTheme, Result, ThemeCharacters, ThemeStyles};
use owo_colors::Style;

use crate::harness::report::TestSourceSpan;

/// A substring of test source file paths
const TEST_FILE_SUBSTRING: &str = "/cloudtruth-cli/tests/";
/// A substring of test harness source file paths
const HARNESS_FILE_SUBSTRING: &str = "/cloudtruth-cli/tests/harness";

const HELP_TEXT: &str = "set the `RUST_BACKTRACE=1` environment variable to display a backtrace.";

pub fn set_panic_hook() {
    miette::set_hook(Box::new(|_| {
        let theme = GraphicalTheme {
            characters: ThemeCharacters::unicode(),
            styles: ThemeStyles {
                highlights: vec![Style::new().red().bold()],
                ..ThemeStyles::ansi()
            },
        };
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .context_lines(3)
                .tab_width(4)
                .with_cause_chain()
                .graphical_theme(theme)
                .build(),
        )
    }))
    .unwrap();
    std::panic::set_hook(Box::new(move |info| {
        let payload = info.payload();
        let message = if let Some(msg) = payload.downcast_ref::<&str>() {
            msg.to_string()
        } else if let Some(msg) = payload.downcast_ref::<String>() {
            msg.to_string()
        } else {
            "Something went wrong".to_string()
        };
        let (help, backtrace) = if is_rust_backtrace_enabled() {
            (None, Some(format_backtrace()))
        } else {
            (Some(HELP_TEXT.into()), None)
        };
        let mut report: Result<()> = Err(Panic {
            message,
            help,
            backtrace,
        }
        .into());
        if let Some(loc) = info.location() {
            report = report.with_context(|| {
                format!("Panic at {}:{}:{}", loc.file(), loc.line(), loc.column())
            });
        }
        if let Some((filename, line, col)) = get_test_location_from_backtrace() {
            report = report.with_context(|| format!("Test failure at {filename}:{line}:{col}"));
            if let Ok(mut src_span) = TestSourceSpan::from_location(filename, line, col) {
                src_span.add_related(report.unwrap_err());
                report = Err(src_span.into());
            }
        }
        eprintln!("{:?}", report.unwrap_err());
    }));
}

#[derive(Debug, Error, Diagnostic)]
#[error("{message}{}", .backtrace.clone().unwrap_or_default())]
#[diagnostic()]
pub struct Panic {
    message: String,
    #[help]
    help: Option<String>,
    backtrace: Option<String>,
}

fn is_rust_backtrace_enabled() -> bool {
    if let Ok(var) = std::env::var("RUST_BACKTRACE") {
        !var.is_empty() && var != "0"
    } else {
        false
    }
}

/// Tries to collect source information from backtrace
fn get_test_location_from_backtrace() -> Option<(String, usize, usize)> {
    for frame in Backtrace::new().frames().iter() {
        for symbol in frame.symbols().iter() {
            if let Some(filename) = symbol.filename().and_then(|f| f.to_str()) {
                if filename.contains(TEST_FILE_SUBSTRING)
                    && !filename.contains(HARNESS_FILE_SUBSTRING)
                {
                    if let (Some(line), Some(col)) = (symbol.lineno(), symbol.colno()) {
                        return Some((filename.into(), line as usize, col as usize));
                    }
                }
            }
        }
    }
    None
}

/// Code adapted from https://github.com/zkat/miette/blob/main/src/panic.rs
fn format_backtrace() -> String {
    // This is all taken from human-panic: https://github.com/rust-cli/human-panic/blob/master/src/report.rs#L55-L107
    const HEX_WIDTH: usize = std::mem::size_of::<usize>() + 2;
    //Padding for next lines after frame's address
    const NEXT_SYMBOL_PADDING: usize = HEX_WIDTH + 6;
    let mut backtrace = String::from("\n==== Backtrace ====\n");
    for (idx, frame) in Backtrace::new().frames().iter().skip(9).enumerate() {
        let ip = frame.ip();
        let _ = write!(backtrace, "\n{:4}: {:2$?}", idx, ip, HEX_WIDTH);

        let symbols = frame.symbols();
        if symbols.is_empty() {
            let _ = write!(backtrace, " - <unresolved>");
            continue;
        }

        for (idx, symbol) in symbols.iter().enumerate() {
            //Print symbols from this address,
            //if there are several addresses
            //we need to put it on next line
            if idx != 0 {
                let _ = write!(backtrace, "\n{:1$}", "", NEXT_SYMBOL_PADDING);
            }

            if let Some(name) = symbol.name() {
                let _ = write!(backtrace, " - {}", name);
            } else {
                let _ = write!(backtrace, " - <unknown>");
            }

            //See if there is debug information with file name and line
            if let (Some(file), Some(line)) = (symbol.filename(), symbol.lineno()) {
                let _ = write!(
                    backtrace,
                    "\n{:3$}at {}:{}",
                    "",
                    file.display(),
                    line,
                    NEXT_SYMBOL_PADDING
                );
            }
        }
    }
    backtrace
}

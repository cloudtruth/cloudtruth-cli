use std::fmt::Write;

use backtrace::Backtrace;

pub fn is_rust_backtrace_enabled() -> bool {
    if let Ok(var) = std::env::var("RUST_BACKTRACE") {
        !var.is_empty() && var != "0"
    } else {
        false
    }
}

/// Code adapted from https://github.com/zkat/miette/blob/main/src/panic.rs
pub fn format_backtrace() -> String {
    // This is all taken from human-panic: https://github.com/rust-cli/human-panic/blob/master/src/report.rs#L55-L107
    const HEX_WIDTH: usize = std::mem::size_of::<usize>() + 2;
    //Padding for next lines after frame's address
    const NEXT_SYMBOL_PADDING: usize = HEX_WIDTH + 6;
    let mut backtrace = String::from("\n==== Backtrace ====\n");
    for (idx, frame) in Backtrace::new().frames().iter().skip(10).enumerate() {
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

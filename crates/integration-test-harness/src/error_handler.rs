use std::sync::Once;

use miette::{GraphicalTheme, ThemeCharacters, ThemeStyles};
use owo_colors::Style;

// global init handler
static INSTALLER: Once = Once::new();

pub fn install_miette_error_handler() {
    // set custom miette reporter handler options
    INSTALLER.call_once(|| {
        miette::set_hook(Box::new(|_| {
            Box::new(
                miette::MietteHandlerOpts::new()
                    .context_lines(3)
                    .tab_width(4)
                    .width(200)
                    .with_cause_chain()
                    .graphical_theme(GraphicalTheme {
                        characters: ThemeCharacters::unicode(),
                        styles: ThemeStyles {
                            highlights: vec![Style::new().red().bold()],
                            ..ThemeStyles::ansi()
                        },
                    })
                    .build(),
            )
        }))
        .expect("Error installing miette handler")
    })
}

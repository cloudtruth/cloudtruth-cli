use miette::{GraphicalTheme, ThemeCharacters, ThemeStyles};
use owo_colors::Style;

pub fn install_miette_error_handler() {
    // set custom miette reporter handler options
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .context_lines(3)
                .tab_width(4)
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
    .expect("Could not install miette error handler");
}

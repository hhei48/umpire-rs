use std::io::{self, IsTerminal, Write};

#[derive(Clone, Copy)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Orange,
}

impl Color {
    fn fg(self) -> &'static str {
        match self {
            Color::Red => "31",
            Color::Green => "32",
            Color::Yellow => "33",
            Color::Orange => "38;5;208",
        }
    }

    fn bg(self) -> &'static str {
        match self {
            Color::Red => "41;97",
            Color::Green => "42;97",
            Color::Yellow => "43;30",
            Color::Orange => "48;5;208;30",
        }
    }
}

/// Decides whether ANSI escape codes may be written to a stream.
#[derive(Clone, Copy)]
pub struct Style {
    enabled: bool,
}

impl Style {
    pub fn stdout() -> Self {
        Self::detect(io::stdout().is_terminal())
    }

    pub fn stderr() -> Self {
        Self::detect(io::stderr().is_terminal())
    }

    fn detect(is_terminal: bool) -> Self {
        let no_color = std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty());
        Self {
            enabled: is_terminal && !no_color,
        }
    }

    pub fn enabled(self) -> bool {
        self.enabled
    }

    pub fn paint(self, text: &str, color: Color) -> String {
        self.wrap(text, color.fg())
    }

    pub fn badge(self, text: &str, color: Color) -> String {
        self.wrap(text, color.bg())
    }

    pub fn bold(self, text: &str) -> String {
        self.wrap(text, "1")
    }

    fn wrap(self, text: &str, code: &str) -> String {
        if self.enabled {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}

/// A single line on stderr that is replaced while work is in progress.
/// It prints nothing when stderr is not a terminal.
pub struct Status {
    live: bool,
}

impl Status {
    pub fn new() -> Self {
        Self {
            live: io::stderr().is_terminal(),
        }
    }

    pub fn show(&self, message: &str) {
        if self.live {
            eprint!("\r\x1b[K{message}");
            let _ = io::stderr().flush();
        }
    }

    pub fn clear(&self) {
        if self.live {
            eprint!("\r\x1b[K");
            let _ = io::stderr().flush();
        }
    }
}

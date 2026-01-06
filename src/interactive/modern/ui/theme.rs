use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Theme {
    pub mode: ThemeMode,

    pub background: Color,
    pub foreground: Color,

    pub border_focused: Color,
    pub border_unfocused: Color,

    pub status_bar_bg: Color,
    pub status_bar_fg: Color,

    pub help_bar_bg: Color,
    pub help_bar_fg: Color,

    pub command_prompt: Color,
    pub command_text: Color,

    pub output_command: Color,
    pub output_success: Color,
    pub output_error: Color,

    pub json_key: Color,
    pub json_string: Color,
    pub json_number: Color,
    pub json_boolean: Color,
    pub json_null: Color,

    pub yaml_key: Color,
    pub yaml_string: Color,
    pub yaml_number: Color,

    pub sidebar_title: Color,
    pub sidebar_text: Color,

    pub highlight: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,

            background: Color::Reset,
            foreground: Color::Rgb(212, 212, 212),

            border_focused: Color::Green,
            border_unfocused: Color::DarkGray,

            status_bar_bg: Color::Blue,
            status_bar_fg: Color::White,

            help_bar_bg: Color::DarkGray,
            help_bar_fg: Color::White,

            command_prompt: Color::Yellow,
            command_text: Color::White,

            output_command: Color::Cyan,
            output_success: Color::White,
            output_error: Color::Red,

            json_key: Color::Rgb(156, 220, 254),
            json_string: Color::Rgb(206, 145, 120),
            json_number: Color::Rgb(181, 206, 168),
            json_boolean: Color::Rgb(86, 156, 214),
            json_null: Color::DarkGray,

            yaml_key: Color::Rgb(156, 220, 254),
            yaml_string: Color::Rgb(206, 145, 120),
            yaml_number: Color::Rgb(181, 206, 168),

            sidebar_title: Color::Yellow,
            sidebar_text: Color::White,

            highlight: Color::Rgb(78, 201, 176),
        }
    }

    #[allow(dead_code)]
    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,

            background: Color::Rgb(255, 255, 255),
            foreground: Color::Rgb(30, 30, 30),

            border_focused: Color::Rgb(0, 122, 204),
            border_unfocused: Color::Rgb(200, 200, 200),

            status_bar_bg: Color::Rgb(0, 122, 204),
            status_bar_fg: Color::White,

            help_bar_bg: Color::Rgb(230, 230, 230),
            help_bar_fg: Color::Rgb(60, 60, 60),

            command_prompt: Color::Rgb(0, 122, 204),
            command_text: Color::Rgb(30, 30, 30),

            output_command: Color::Rgb(0, 102, 204),
            output_success: Color::Rgb(30, 30, 30),
            output_error: Color::Rgb(205, 49, 49),

            json_key: Color::Rgb(0, 102, 204),
            json_string: Color::Rgb(163, 21, 21),
            json_number: Color::Rgb(9, 134, 88),
            json_boolean: Color::Rgb(0, 0, 255),
            json_null: Color::Rgb(128, 128, 128),

            yaml_key: Color::Rgb(0, 102, 204),
            yaml_string: Color::Rgb(163, 21, 21),
            yaml_number: Color::Rgb(9, 134, 88),

            sidebar_title: Color::Rgb(0, 102, 204),
            sidebar_text: Color::Rgb(30, 30, 30),

            highlight: Color::Rgb(0, 122, 204),
        }
    }

    #[allow(dead_code)]
    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
        }
    }

    pub fn border_style(&self, focused: bool) -> Style {
        if focused {
            Style::default().fg(self.border_focused)
        } else {
            Style::default().fg(self.border_unfocused)
        }
    }

    pub fn status_bar_style(&self) -> Style {
        Style::default()
            .bg(self.status_bar_bg)
            .fg(self.status_bar_fg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn help_bar_style(&self) -> Style {
        Style::default().bg(self.help_bar_bg).fg(self.help_bar_fg)
    }

    pub fn command_style(&self) -> Style {
        Style::default()
            .fg(self.output_command)
            .add_modifier(Modifier::BOLD)
    }

    pub fn prompt_style(&self) -> Style {
        Style::default().fg(self.command_prompt)
    }

    #[allow(dead_code)]
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.output_success)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.output_error)
    }

    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.sidebar_title)
            .add_modifier(Modifier::BOLD)
    }
}

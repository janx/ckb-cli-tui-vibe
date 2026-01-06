use ratatui::{
    style::Style,
    text::{Line, Span},
};

use super::theme::Theme;

pub struct OutputHighlighter<'a> {
    theme: &'a Theme,
}

impl<'a> OutputHighlighter<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }

    pub fn highlight_output(&self, text: &str) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        for line in text.lines() {
            lines.push(self.highlight_line(line));
        }

        lines
    }

    fn highlight_line(&self, line: &str) -> Line<'static> {
        let trimmed = line.trim_start();

        if trimmed.starts_with('{')
            || trimmed.starts_with('}')
            || trimmed.starts_with('[')
            || trimmed.starts_with(']')
        {
            return Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(self.theme.foreground),
            ));
        }

        if let Some(colon_pos) = trimmed.find(':') {
            let indent = line.len() - trimmed.len();
            let indent_str: String = line.chars().take(indent).collect();
            let key = &trimmed[..colon_pos];
            let rest = &trimmed[colon_pos..];

            let key_clean = key.trim_matches('"').trim_matches('\'');

            let mut spans = vec![
                Span::raw(indent_str),
                Span::styled(
                    if key.starts_with('"') {
                        format!("\"{}\"", key_clean)
                    } else {
                        key_clean.to_string()
                    },
                    Style::default().fg(self.theme.json_key),
                ),
            ];

            if rest.len() > 1 {
                let value = rest[1..].trim();
                spans.push(Span::raw(": "));
                spans.push(self.highlight_value(value));
            } else {
                spans.push(Span::raw(rest.to_string()));
            }

            Line::from(spans)
        } else {
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(self.theme.foreground),
            ))
        }
    }

    fn highlight_value(&self, value: &str) -> Span<'static> {
        let trimmed = value.trim().trim_end_matches(',');

        if trimmed == "null" || trimmed == "~" {
            Span::styled(value.to_string(), Style::default().fg(self.theme.json_null))
        } else if trimmed == "true" || trimmed == "false" {
            Span::styled(
                value.to_string(),
                Style::default().fg(self.theme.json_boolean),
            )
        } else if trimmed.starts_with('"') || trimmed.starts_with('\'') {
            Span::styled(
                value.to_string(),
                Style::default().fg(self.theme.json_string),
            )
        } else if trimmed.parse::<f64>().is_ok() || trimmed.starts_with("0x") {
            Span::styled(
                value.to_string(),
                Style::default().fg(self.theme.json_number),
            )
        } else {
            Span::styled(
                value.to_string(),
                Style::default().fg(self.theme.json_string),
            )
        }
    }
}

pub struct CommandHighlighter<'a> {
    theme: &'a Theme,
    keywords: Vec<&'static str>,
}

impl<'a> CommandHighlighter<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            theme,
            keywords: vec![
                "rpc", "wallet", "account", "dao", "tx", "mock-tx", "util", "plugin", "molecule",
                "sudt", "deploy", "config", "info", "exit", "quit",
            ],
        }
    }

    pub fn highlight_input(&self, input: &str) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return vec![Span::raw(input.to_string())];
        }

        let mut current_pos = 0;

        for (i, part) in parts.iter().enumerate() {
            if let Some(pos) = input[current_pos..].find(part) {
                let actual_pos = current_pos + pos;

                if actual_pos > current_pos {
                    spans.push(Span::raw(input[current_pos..actual_pos].to_string()));
                }

                let style = if i == 0 && self.keywords.contains(part) {
                    Style::default().fg(self.theme.highlight)
                } else if part.starts_with("--") || part.starts_with('-') {
                    Style::default().fg(self.theme.json_key)
                } else if part.starts_with("0x") {
                    Style::default().fg(self.theme.json_number)
                } else {
                    Style::default().fg(self.theme.command_text)
                };

                spans.push(Span::styled((*part).to_string(), style));
                current_pos = actual_pos + part.len();
            }
        }

        if current_pos < input.len() {
            spans.push(Span::raw(input[current_pos..].to_string()));
        }

        spans
    }
}

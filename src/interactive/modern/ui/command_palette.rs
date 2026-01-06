use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::theme::Theme;

#[derive(Debug, Clone)]
pub struct PaletteEntry {
    pub command: String,
    pub description: String,
}

pub fn extract_commands(app: &clap::App<'static>) -> Vec<PaletteEntry> {
    let mut entries = Vec::new();

    for sub in app.get_subcommands() {
        let name = sub.get_name().to_string();
        let about = sub.get_about().unwrap_or_default().to_string();

        entries.push(PaletteEntry {
            command: name.clone(),
            description: about.clone(),
        });

        for subsub in sub.get_subcommands() {
            let subname = subsub.get_name().to_string();
            let subabout = subsub.get_about().unwrap_or_default().to_string();

            entries.push(PaletteEntry {
                command: format!("{} {}", name, subname),
                description: subabout,
            });
        }
    }

    entries
}

pub fn filter_commands<'a>(entries: &'a [PaletteEntry], query: &str) -> Vec<&'a PaletteEntry> {
    if query.is_empty() {
        return entries.iter().collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches: Vec<_> = entries
        .iter()
        .filter(|e| {
            e.command.to_lowercase().contains(&query_lower)
                || fuzzy_match(&e.command.to_lowercase(), &query_lower)
                || e.description.to_lowercase().contains(&query_lower)
        })
        .collect();

    matches.sort_by(|a, b| {
        let a_starts = a.command.to_lowercase().starts_with(&query_lower);
        let b_starts = b.command.to_lowercase().starts_with(&query_lower);
        b_starts.cmp(&a_starts)
    });

    matches
}

fn fuzzy_match(text: &str, pattern: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();

    for c in text.chars() {
        if pattern_chars.peek() == Some(&c) {
            pattern_chars.next();
        }
        if pattern_chars.peek().is_none() {
            return true;
        }
    }

    pattern_chars.peek().is_none()
}

pub fn render_command_palette(
    frame: &mut Frame,
    query: &str,
    filtered: &[&PaletteEntry],
    selected_index: usize,
    theme: &Theme,
) {
    let popup_width = 60u16.min(frame.area().width - 4);
    let popup_height = 15u16.min(frame.area().height - 4);

    let popup_x = (frame.area().width - popup_width) / 2;
    let popup_y = (frame.area().height - popup_height) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let inner_height = popup_height.saturating_sub(4) as usize;
    let visible_start = if selected_index >= inner_height {
        selected_index - inner_height + 1
    } else {
        0
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("> ", Style::default().fg(theme.command_prompt)),
            Span::styled(query, Style::default().fg(theme.foreground)),
            Span::styled("█", Style::default().fg(theme.highlight)),
        ]),
        Line::from(""),
    ];

    for (i, entry) in filtered
        .iter()
        .enumerate()
        .skip(visible_start)
        .take(inner_height)
    {
        let is_selected = i == selected_index;
        let style = if is_selected {
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.foreground)
        };

        let desc_style = if is_selected {
            Style::default().fg(theme.sidebar_text)
        } else {
            Style::default().fg(theme.border_unfocused)
        };

        let prefix = if is_selected { "▶ " } else { "  " };
        let cmd_display = if entry.command.chars().count() > 25 {
            let truncated: String = entry.command.chars().take(22).collect();
            format!("{}...", truncated)
        } else {
            format!("{:25}", entry.command)
        };

        let desc_display = if entry.description.chars().count() > 28 {
            let truncated: String = entry.description.chars().take(25).collect();
            format!("{}...", truncated)
        } else {
            entry.description.clone()
        };

        lines.push(Line::from(vec![
            Span::raw(prefix),
            Span::styled(cmd_display, style),
            Span::raw(" "),
            Span::styled(desc_display, desc_style),
        ]));
    }

    let count_info = format!(" {}/{} ", filtered.len(), filtered.len());
    let title = format!(" Command Palette {} (Esc to close) ", count_info);

    let popup = Paragraph::new(lines).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.highlight)),
    );

    frame.render_widget(popup, popup_area);
}

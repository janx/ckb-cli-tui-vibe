use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::interactive::modern::state::Tab;

use super::theme::Theme;

pub fn render_tab_bar(frame: &mut Frame, area: Rect, current_tab: Tab, theme: &Theme) {
    let tabs = [(Tab::Command, "Command", "1"), (Tab::Logs, "Logs", "2")];

    let mut spans = vec![Span::raw(" ")];

    for (i, (tab, name, key)) in tabs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(
                " │ ",
                Style::default().fg(theme.border_unfocused),
            ));
        }

        let is_active = *tab == current_tab;
        let tab_style = if is_active {
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.sidebar_text)
        };

        let key_style = Style::default().fg(theme.border_unfocused);

        if is_active {
            spans.push(Span::styled("▶ ", tab_style));
        }
        spans.push(Span::styled(*name, tab_style));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(format!("[Alt+{}]", key), key_style));
    }

    let tab_bar = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.help_bar_bg));

    frame.render_widget(tab_bar, area);
}

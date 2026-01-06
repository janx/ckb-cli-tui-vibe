use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::interactive::modern::state::Completion;

const MAX_VISIBLE_COMPLETIONS: usize = 15;

pub fn render_completion_popup(
    frame: &mut Frame,
    completions: &[Completion],
    selected_index: usize,
    input_area: Rect,
) {
    if completions.is_empty() {
        return;
    }

    let visible_count = completions.len().min(MAX_VISIBLE_COMPLETIONS);
    let popup_height = (visible_count as u16) + 2;
    let popup_width = completions
        .iter()
        .map(|c| c.display.len())
        .max()
        .unwrap_or(20)
        .max(30) as u16
        + 4;

    let popup_x = input_area.x + 6;
    let available_above = input_area.y;
    let actual_height = popup_height.min(available_above).max(3);
    let popup_y = input_area.y.saturating_sub(actual_height);

    let popup_area = Rect::new(
        popup_x.min(frame.area().width.saturating_sub(popup_width)),
        popup_y,
        popup_width.min(frame.area().width.saturating_sub(popup_x)),
        actual_height,
    );

    frame.render_widget(Clear, popup_area);

    let visible_items = (actual_height.saturating_sub(2)) as usize;
    let scroll_offset = if completions.len() <= visible_items {
        0
    } else if selected_index < visible_items / 2 {
        0
    } else if selected_index >= completions.len().saturating_sub(visible_items / 2) {
        completions.len().saturating_sub(visible_items)
    } else {
        selected_index.saturating_sub(visible_items / 2)
    };

    let lines: Vec<Line> = completions
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_items)
        .map(|(i, completion)| {
            let is_selected = i == selected_index;
            let style = if is_selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if completion.is_required {
                Style::default().fg(Color::Red)
            } else if completion.replacement.starts_with("--") {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Green)
            };

            Line::from(Span::styled(format!(" {} ", completion.display), style))
        })
        .collect();

    let title = format!(
        " Completions ({}/{}) ",
        selected_index + 1,
        completions.len()
    );
    let popup = Paragraph::new(lines).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(popup, popup_area);
}

pub fn render_history_search(
    frame: &mut Frame,
    search_query: &str,
    matches: &[String],
    selected_index: usize,
) {
    let popup_width = 60u16.min(frame.area().width - 4);
    let visible_count = matches.len().min(MAX_VISIBLE_COMPLETIONS);
    let popup_height = visible_count as u16 + 4;

    let popup_x = (frame.area().width - popup_width) / 2;
    let popup_y = (frame.area().height - popup_height) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let mut lines = vec![Line::from(vec![
        Span::styled("Search: ", Style::default().fg(Color::Yellow)),
        Span::styled(search_query, Style::default().fg(Color::White)),
        Span::styled("█", Style::default().fg(Color::Gray)),
    ])];

    lines.push(Line::from(""));

    let scroll_offset = selected_index
        .saturating_sub(MAX_VISIBLE_COMPLETIONS / 2)
        .min(matches.len().saturating_sub(MAX_VISIBLE_COMPLETIONS));

    for (i, entry) in matches
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(MAX_VISIBLE_COMPLETIONS)
    {
        let is_selected = i == selected_index;
        let style = if is_selected {
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let max_len = popup_width as usize - 4;
        let display = if entry.chars().count() > max_len {
            let truncated: String = entry.chars().take(max_len.saturating_sub(3)).collect();
            format!("{}...", truncated)
        } else {
            entry.clone()
        };

        lines.push(Line::from(Span::styled(format!(" {} ", display), style)));
    }

    let popup = Paragraph::new(lines).block(
        Block::default()
            .title(" History Search (Ctrl+R) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(popup, popup_area);
}

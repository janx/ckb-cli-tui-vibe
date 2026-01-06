pub mod syntax;
pub mod theme;

use unicode_width::UnicodeWidthStr;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::app::TuiApp;
use super::state::{ChainState, Pane};
use syntax::{CommandHighlighter, OutputHighlighter};
use theme::Theme;

pub fn render(frame: &mut Frame, app: &TuiApp, chain_state: &ChainState) {
    let theme = Theme::default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_status_bar(frame, chunks[0], app, chain_state, &theme);
    render_main_area(frame, chunks[1], app, chain_state, &theme);
    render_input(frame, chunks[2], app, &theme);
    render_help_bar(frame, chunks[3], &theme);
}

fn render_status_bar(
    frame: &mut Frame,
    area: Rect,
    app: &TuiApp,
    chain_state: &ChainState,
    theme: &Theme,
) {
    let network = app
        .config
        .network()
        .map(|n| format!("{}", n))
        .unwrap_or_else(|| "Unknown".to_string());

    let sync_indicator = if chain_state.is_syncing { "⟳" } else { "✓" };

    let status = format!(
        " CKB CLI v2.0.0 │ Network: {} │ Height: {} │ Peers: {} │ {} ",
        network, chain_state.height, chain_state.peers, sync_indicator
    );

    let status_bar = Paragraph::new(status).style(theme.status_bar_style());
    frame.render_widget(status_bar, area);
}

fn render_main_area(
    frame: &mut Frame,
    area: Rect,
    app: &TuiApp,
    chain_state: &ChainState,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(40),
            Constraint::Length(app.ui_state.sidebar_width),
        ])
        .split(area);

    render_output(frame, chunks[0], app, theme);
    render_sidebar(frame, chunks[1], app, chain_state, theme);
}

fn render_output(frame: &mut Frame, area: Rect, app: &TuiApp, theme: &Theme) {
    let is_focused = app.ui_state.focused_pane == Pane::Output;
    let border_style = theme.border_style(is_focused);
    let highlighter = OutputHighlighter::new(theme);

    let mut lines: Vec<Line> = Vec::new();

    for entry in &app.command_state.output_buffer {
        lines.push(Line::from(vec![
            Span::styled("> ", theme.prompt_style()),
            Span::styled(&entry.command, theme.command_style()),
        ]));

        if entry.success {
            let highlighted = highlighter.highlight_output(&entry.result);
            lines.extend(highlighted);
        } else {
            for line in entry.result.lines() {
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    theme.error_style(),
                )));
            }
        }
        lines.push(Line::from(""));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "Welcome to CKB CLI Modern TUI!",
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Type commands below. Press Ctrl+C to exit.",
            Style::default().fg(theme.foreground),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Try: rpc get_tip_header",
            Style::default().fg(theme.sidebar_text),
        )));
    }

    let output = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Output ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(output, area);
}

fn render_sidebar(
    frame: &mut Frame,
    area: Rect,
    app: &TuiApp,
    chain_state: &ChainState,
    theme: &Theme,
) {
    let is_focused = app.ui_state.focused_pane == Pane::Sidebar;
    let border_style = theme.border_style(is_focused);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let chain_info = vec![
        Line::from(Span::styled("Chain Status", theme.title_style())),
        Line::from(""),
        Line::from(vec![
            Span::styled("Height: ", Style::default().fg(theme.sidebar_text)),
            Span::styled(
                chain_state.height.to_string(),
                Style::default().fg(theme.json_number),
            ),
        ]),
        Line::from(vec![
            Span::styled("Epoch:  ", Style::default().fg(theme.sidebar_text)),
            Span::styled(
                chain_state.epoch.to_string(),
                Style::default().fg(theme.json_number),
            ),
        ]),
        Line::from(vec![
            Span::styled("Peers:  ", Style::default().fg(theme.sidebar_text)),
            Span::styled(
                chain_state.peers.to_string(),
                Style::default().fg(theme.json_number),
            ),
        ]),
        Line::from(vec![
            Span::styled("Sync:   ", Style::default().fg(theme.sidebar_text)),
            Span::styled(
                if chain_state.is_syncing {
                    "syncing..."
                } else {
                    "✓ synced"
                },
                Style::default().fg(if chain_state.is_syncing {
                    theme.output_error
                } else {
                    theme.highlight
                }),
            ),
        ]),
    ];

    let chain_widget = Paragraph::new(chain_info).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(chain_widget, chunks[0]);

    let recent_cmds: Vec<Line> = std::iter::once(Line::from(Span::styled(
        "Recent Commands",
        theme.title_style(),
    )))
    .chain(std::iter::once(Line::from("")))
    .chain(app.command_state.history.iter().rev().take(10).map(|cmd| {
        let display = if cmd.len() > 20 {
            format!("{}...", &cmd[..17])
        } else {
            cmd.clone()
        };
        Line::from(Span::styled(
            display,
            Style::default().fg(theme.sidebar_text),
        ))
    }))
    .collect();

    let history_widget = Paragraph::new(recent_cmds).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(history_widget, chunks[1]);
}

fn render_input(frame: &mut Frame, area: Rect, app: &TuiApp, theme: &Theme) {
    let is_focused = app.ui_state.focused_pane == Pane::Input;
    let border_style = theme.border_style(is_focused);
    let highlighter = CommandHighlighter::new(theme);

    let input_spans = highlighter.highlight_input(&app.command_state.input);
    let input_line = Line::from(input_spans);

    let input = Paragraph::new(input_line).block(
        Block::default()
            .title(" CKB> ")
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(input, area);

    frame.set_cursor_position((
        area.x + app.command_state.input.width() as u16 + 1,
        area.y + 1,
    ));
}

fn render_help_bar(frame: &mut Frame, area: Rect, theme: &Theme) {
    let help_text = " Ctrl+C Exit │ Ctrl+L Clear │ ↑/↓ History │ Enter Execute │ Tab Complete ";

    let help_bar = Paragraph::new(help_text).style(theme.help_bar_style());
    frame.render_widget(help_bar, area);
}

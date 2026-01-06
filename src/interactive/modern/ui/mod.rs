use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::app::TuiApp;
use super::state::{ChainState, Pane};

pub fn render(frame: &mut Frame, app: &TuiApp, chain_state: &ChainState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_status_bar(frame, chunks[0], app, chain_state);
    render_main_area(frame, chunks[1], app, chain_state);
    render_input(frame, chunks[2], app);
    render_help_bar(frame, chunks[3]);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &TuiApp, chain_state: &ChainState) {
    let network = app
        .config
        .network()
        .map(|n| format!("{}", n))
        .unwrap_or_else(|| "Unknown".to_string());

    let status = format!(
        " CKB CLI v2.0.0 │ Network: {} │ Height: {} │ Peers: {} ",
        network, chain_state.height, chain_state.peers
    );

    let status_bar = Paragraph::new(status).style(
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

    frame.render_widget(status_bar, area);
}

fn render_main_area(frame: &mut Frame, area: Rect, app: &TuiApp, chain_state: &ChainState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(40),
            Constraint::Length(app.ui_state.sidebar_width),
        ])
        .split(area);

    render_output(frame, chunks[0], app);
    render_sidebar(frame, chunks[1], app, chain_state);
}

fn render_output(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let is_focused = app.ui_state.focused_pane == Pane::Output;
    let border_style = if is_focused {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let mut lines: Vec<Line> = Vec::new();

    for entry in &app.command_state.output_buffer {
        let cmd_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        lines.push(Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Yellow)),
            Span::styled(&entry.command, cmd_style),
        ]));

        let result_color = if entry.success {
            Color::White
        } else {
            Color::Red
        };
        for line in entry.result.lines() {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(result_color),
            )));
        }
        lines.push(Line::from(""));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "Welcome to CKB CLI Modern TUI!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from("Type commands below. Press Ctrl+C to exit."));
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

fn render_sidebar(frame: &mut Frame, area: Rect, app: &TuiApp, chain_state: &ChainState) {
    let is_focused = app.ui_state.focused_pane == Pane::Sidebar;
    let border_style = if is_focused {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let chain_info = vec![
        Line::from(Span::styled(
            "Chain Status",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Height: {}", chain_state.height)),
        Line::from(format!("Epoch:  {}", chain_state.epoch)),
        Line::from(format!("Peers:  {}", chain_state.peers)),
        Line::from(format!(
            "Sync:   {}",
            if chain_state.is_syncing {
                "syncing..."
            } else {
                "✓"
            }
        )),
    ];

    let chain_widget = Paragraph::new(chain_info).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(chain_widget, chunks[0]);

    let recent_cmds: Vec<Line> = std::iter::once(Line::from(Span::styled(
        "Recent Commands",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )))
    .chain(std::iter::once(Line::from("")))
    .chain(app.command_state.history.iter().rev().take(10).map(|cmd| {
        let display = if cmd.len() > 20 {
            format!("{}...", &cmd[..17])
        } else {
            cmd.clone()
        };
        Line::from(display)
    }))
    .collect();

    let history_widget = Paragraph::new(recent_cmds).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(history_widget, chunks[1]);
}

fn render_input(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let is_focused = app.ui_state.focused_pane == Pane::Input;
    let border_style = if is_focused {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let input = Paragraph::new(app.command_state.input.as_str())
        .block(
            Block::default()
                .title(" CKB> ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(input, area);

    frame.set_cursor_position((
        area.x + app.command_state.input.chars().count() as u16 + 1,
        area.y + 1,
    ));
}

fn render_help_bar(frame: &mut Frame, area: Rect) {
    let help_text = " Ctrl+C Exit │ Ctrl+L Clear │ ↑/↓ History │ Enter Execute ";

    let help_bar =
        Paragraph::new(help_text).style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(help_bar, area);
}

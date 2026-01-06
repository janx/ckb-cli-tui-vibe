use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::interactive::modern::app::TuiApp;
use crate::interactive::modern::state::Pane;

#[allow(dead_code)]
pub struct EventHandler;

#[allow(dead_code)]
impl EventHandler {
    pub fn handle_key(app: &mut TuiApp, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                app.should_quit = true;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                app.command_state.clear_output();
            }

            (_, KeyCode::F(1)) => {
                app.ui_state.focused_pane = Pane::Sidebar;
            }
            (_, KeyCode::F(2)) => {
                app.ui_state.focused_pane = Pane::Output;
            }
            (_, KeyCode::F(3)) => {
                app.ui_state.focused_pane = Pane::Input;
            }

            _ => match app.ui_state.focused_pane {
                Pane::Input => Self::handle_input_key(app, key),
                Pane::Output => Self::handle_output_key(app, key),
                Pane::Sidebar => Self::handle_sidebar_key(app, key),
            },
        }
    }

    fn handle_input_key(app: &mut TuiApp, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                let input = app.command_state.input.trim().to_string();
                if !input.is_empty() {
                    if input == "exit" || input == "quit" {
                        app.should_quit = true;
                    } else {
                        app.command_state.add_to_history(input.clone());
                        let output = format!("Command executed: {}", input);
                        app.command_state.add_output(input, output, true);
                    }
                    app.command_state.input.clear();
                }
            }
            KeyCode::Char(c) => {
                app.command_state.input.push(c);
            }
            KeyCode::Backspace => {
                app.command_state.input.pop();
            }
            KeyCode::Up => {
                app.command_state.navigate_history_up();
            }
            KeyCode::Down => {
                app.command_state.navigate_history_down();
            }
            KeyCode::Tab => {
                app.ui_state.show_completion = true;
            }
            KeyCode::Esc => {
                app.ui_state.show_completion = false;
            }
            _ => {}
        }
    }

    fn handle_output_key(app: &mut TuiApp, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.ui_state.output_scroll = app.ui_state.output_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.ui_state.output_scroll = app.ui_state.output_scroll.saturating_add(1);
            }
            KeyCode::PageUp => {
                app.ui_state.output_scroll = app.ui_state.output_scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                app.ui_state.output_scroll = app.ui_state.output_scroll.saturating_add(10);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                app.ui_state.output_scroll = 0;
            }
            _ => {}
        }
    }

    fn handle_sidebar_key(_app: &mut TuiApp, _key: KeyEvent) {}
}

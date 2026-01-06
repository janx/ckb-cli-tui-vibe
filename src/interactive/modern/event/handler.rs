use crossterm::event::{KeyCode, KeyEvent};

use crate::interactive::modern::state::Pane;

pub fn handle_output_scroll(focused_pane: Pane, output_scroll: &mut usize, key: KeyEvent) {
    if focused_pane != Pane::Output {
        return;
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            *output_scroll = output_scroll.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            *output_scroll = output_scroll.saturating_add(1);
        }
        KeyCode::PageUp => {
            *output_scroll = output_scroll.saturating_sub(10);
        }
        KeyCode::PageDown => {
            *output_scroll = output_scroll.saturating_add(10);
        }
        KeyCode::Home | KeyCode::Char('g') => {
            *output_scroll = 0;
        }
        KeyCode::End | KeyCode::Char('G') => {
            *output_scroll = usize::MAX;
        }
        _ => {}
    }
}

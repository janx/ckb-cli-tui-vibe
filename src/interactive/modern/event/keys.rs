use crossterm::event::{KeyCode, KeyModifiers};

#[allow(dead_code)]
pub struct KeyBindings;

#[allow(dead_code)]
impl KeyBindings {
    pub fn is_quit(modifiers: KeyModifiers, code: KeyCode) -> bool {
        matches!(
            (modifiers, code),
            (KeyModifiers::CONTROL, KeyCode::Char('c'))
                | (KeyModifiers::CONTROL, KeyCode::Char('q'))
        )
    }

    pub fn is_clear(modifiers: KeyModifiers, code: KeyCode) -> bool {
        matches!(
            (modifiers, code),
            (KeyModifiers::CONTROL, KeyCode::Char('l'))
        )
    }

    pub fn is_help(modifiers: KeyModifiers, code: KeyCode) -> bool {
        matches!(
            (modifiers, code),
            (KeyModifiers::NONE, KeyCode::Char('?')) | (KeyModifiers::NONE, KeyCode::F(1))
        )
    }
}

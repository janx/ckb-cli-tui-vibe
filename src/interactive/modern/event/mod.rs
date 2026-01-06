mod handler;
mod keys;

#[allow(unused_imports)]
pub use handler::handle_output_scroll;
#[allow(unused_imports)]
pub use keys::KeyBindings;

use crossterm::event::{KeyEvent, MouseEvent};

use super::state::ChainState;

#[allow(dead_code)]
#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    ChainUpdate(ChainState),
    Tick,
}

#[allow(dead_code)]
pub struct CommandResult {
    pub command: String,
    pub output: String,
    pub success: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pane {
    #[default]
    Input,
    Output,
    Sidebar,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UiState {
    pub focused_pane: Pane,
    pub output_scroll: usize,
    pub show_help: bool,
    pub show_completion: bool,
    pub completion_index: usize,
    pub sidebar_width: u16,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            focused_pane: Pane::Input,
            output_scroll: 0,
            show_help: false,
            show_completion: false,
            completion_index: 0,
            sidebar_width: 25,
        }
    }
}

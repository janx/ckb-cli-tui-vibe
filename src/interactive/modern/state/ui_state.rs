#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pane {
    #[default]
    Input,
    Output,
    Sidebar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Command,
    Logs,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutAreas {
    pub output: (u16, u16, u16, u16),
    pub sidebar: (u16, u16, u16, u16),
    pub input: (u16, u16, u16, u16),
}

impl LayoutAreas {
    pub fn contains(&self, pane: Pane, x: u16, y: u16) -> bool {
        let (px, py, pw, ph) = match pane {
            Pane::Output => self.output,
            Pane::Sidebar => self.sidebar,
            Pane::Input => self.input,
        };
        x >= px && x < px + pw && y >= py && y < py + ph
    }

    pub fn pane_at(&self, x: u16, y: u16) -> Option<Pane> {
        if self.contains(Pane::Input, x, y) {
            Some(Pane::Input)
        } else if self.contains(Pane::Output, x, y) {
            Some(Pane::Output)
        } else if self.contains(Pane::Sidebar, x, y) {
            Some(Pane::Sidebar)
        } else {
            None
        }
    }
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
    pub layout: LayoutAreas,
    pub max_output_scroll: usize,
    pub current_tab: Tab,
    pub logs_scroll: usize,
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
            layout: LayoutAreas::default(),
            max_output_scroll: 0,
            current_tab: Tab::Command,
            logs_scroll: 0,
        }
    }
}

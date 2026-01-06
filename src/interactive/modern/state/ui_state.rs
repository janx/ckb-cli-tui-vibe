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
    pub show_palette: bool,
    pub palette_query: String,
    pub palette_index: usize,
    pub output_search_mode: bool,
    pub output_search_query: String,
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
            show_palette: false,
            palette_query: String::new(),
            palette_index: 0,
            output_search_mode: false,
            output_search_query: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_areas_contains() {
        let layout = LayoutAreas {
            output: (0, 0, 50, 20),
            sidebar: (50, 0, 25, 20),
            input: (0, 20, 75, 5),
        };

        assert!(layout.contains(Pane::Output, 25, 10));
        assert!(!layout.contains(Pane::Output, 60, 10));

        assert!(layout.contains(Pane::Sidebar, 60, 10));
        assert!(!layout.contains(Pane::Sidebar, 25, 10));

        assert!(layout.contains(Pane::Input, 30, 22));
        assert!(!layout.contains(Pane::Input, 30, 10));
    }

    #[test]
    fn test_layout_areas_pane_at() {
        let layout = LayoutAreas {
            output: (0, 0, 50, 20),
            sidebar: (50, 0, 25, 20),
            input: (0, 20, 75, 5),
        };

        assert_eq!(layout.pane_at(25, 10), Some(Pane::Output));
        assert_eq!(layout.pane_at(60, 10), Some(Pane::Sidebar));
        assert_eq!(layout.pane_at(30, 22), Some(Pane::Input));
        assert_eq!(layout.pane_at(100, 100), None);
    }

    #[test]
    fn test_pane_default() {
        assert_eq!(Pane::default(), Pane::Input);
    }

    #[test]
    fn test_tab_default() {
        assert_eq!(Tab::default(), Tab::Command);
    }

    #[test]
    fn test_ui_state_default() {
        let state = UiState::default();
        assert_eq!(state.focused_pane, Pane::Input);
        assert_eq!(state.output_scroll, 0);
        assert!(!state.show_help);
        assert!(!state.show_completion);
        assert_eq!(state.sidebar_width, 25);
        assert_eq!(state.current_tab, Tab::Command);
        assert!(!state.show_palette);
        assert!(!state.output_search_mode);
    }
}

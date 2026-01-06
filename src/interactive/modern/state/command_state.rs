use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_HISTORY_SIZE: usize = 1000;
const MAX_OUTPUT_ENTRIES: usize = 1000;
const MAX_OUTPUT_LINES_PER_ENTRY: usize = 500;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OutputEntry {
    pub command: String,
    pub result: String,
    pub timestamp: u64,
    pub success: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct CommandState {
    pub input: String,
    pub cursor_pos: usize,
    pub history: VecDeque<String>,
    pub history_index: Option<usize>,
    pub output_buffer: VecDeque<OutputEntry>,
    pub completions: Vec<(String, String)>,
    history_file: PathBuf,
    saved_input: String,
}

impl CommandState {
    pub fn new(ckb_cli_dir: &Path) -> Result<Self, String> {
        let mut history_file = ckb_cli_dir.to_path_buf();
        history_file.push("history");

        let history = Self::load_history(&history_file);

        Ok(Self {
            input: String::new(),
            cursor_pos: 0,
            history,
            history_index: None,
            output_buffer: VecDeque::new(),
            completions: Vec::new(),
            history_file,
            saved_input: String::new(),
        })
    }

    fn load_history(path: &Path) -> VecDeque<String> {
        let mut history = VecDeque::new();

        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                if !line.is_empty() {
                    history.push_back(line);
                }
            }
            while history.len() > MAX_HISTORY_SIZE {
                history.pop_front();
            }
        }

        history
    }

    pub fn save_history(&self) -> Result<(), String> {
        let mut file = File::create(&self.history_file)
            .map_err(|e| format!("Failed to create history file: {}", e))?;

        for entry in &self.history {
            writeln!(file, "{}", entry).map_err(|e| format!("Failed to write history: {}", e))?;
        }

        Ok(())
    }

    pub fn add_to_history(&mut self, command: String) {
        if command.is_empty() {
            return;
        }

        if self.history.back().map(|s| s.as_str()) == Some(command.as_str()) {
            return;
        }

        self.history.push_back(command);
        while self.history.len() > MAX_HISTORY_SIZE {
            self.history.pop_front();
        }

        self.history_index = None;
        self.saved_input.clear();
    }

    pub fn navigate_history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.saved_input = self.input.clone();
                self.history_index = Some(self.history.len() - 1);
                self.input = self.history[self.history.len() - 1].clone();
            }
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
                self.input = self.history[idx - 1].clone();
            }
            _ => {}
        }
    }

    pub fn navigate_history_down(&mut self) {
        match self.history_index {
            Some(idx) if idx < self.history.len() - 1 => {
                self.history_index = Some(idx + 1);
                self.input = self.history[idx + 1].clone();
            }
            Some(_) => {
                self.history_index = None;
                self.input = self.saved_input.clone();
            }
            None => {}
        }
    }

    pub fn add_output(&mut self, command: String, result: String, success: bool) {
        let truncated_result = truncate_output(&result, MAX_OUTPUT_LINES_PER_ENTRY);

        let entry = OutputEntry {
            command,
            result: truncated_result,
            timestamp: current_timestamp(),
            success,
        };

        self.output_buffer.push_back(entry);
        while self.output_buffer.len() > MAX_OUTPUT_ENTRIES {
            self.output_buffer.pop_front();
        }
    }

    pub fn clear_output(&mut self) {
        self.output_buffer.clear();
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn truncate_output(output: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() <= max_lines {
        return output.to_string();
    }

    let truncated: String = lines[..max_lines].join("\n");
    format!(
        "{}\n\n... ({} more lines truncated)",
        truncated,
        lines.len() - max_lines
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_history_navigation() {
        let temp_dir = env::temp_dir();
        let mut state = CommandState::new(&temp_dir).unwrap();

        state.add_to_history("cmd1".to_string());
        state.add_to_history("cmd2".to_string());
        state.add_to_history("cmd3".to_string());

        state.input = "current".to_string();

        state.navigate_history_up();
        assert_eq!(state.input, "cmd3");

        state.navigate_history_up();
        assert_eq!(state.input, "cmd2");

        state.navigate_history_down();
        assert_eq!(state.input, "cmd3");

        state.navigate_history_down();
        assert_eq!(state.input, "current");
    }

    #[test]
    fn test_no_duplicate_history() {
        let temp_dir = env::temp_dir();
        let mut state = CommandState::new(&temp_dir).unwrap();

        state.add_to_history("cmd1".to_string());
        state.add_to_history("cmd1".to_string());

        assert_eq!(state.history.len(), 1);
    }
}

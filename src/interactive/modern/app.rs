use std::collections::VecDeque;
use std::fs::File;
use std::io::{Stdout, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};
use regex::Regex;

use ckb_signer::KeyStore;

use crate::build_interactive;
use crate::plugin::PluginManager;
use crate::subcommands::{
    AccountSubCommand, CliSubCommand, DAOSubCommand, DeploySubCommand, MockTxSubCommand,
    MoleculeSubCommand, PluginSubCommand, RpcSubCommand, SudtSubCommand, TxSubCommand,
    UtilSubCommand, WalletSubCommand,
};
use crate::utils::{
    config::GlobalConfig,
    genesis_info::GenesisInfo,
    other::{get_genesis_info, get_network_type},
    printer::{OutputFormat, Printable},
    rpc::{HttpRpcClient, RawHttpRpcClient},
};

use super::event::{handle_output_scroll, AppEvent};
use super::state::{ChainState, CommandState, Completion, Pane, Tab, TuiCompleter, UiState};
use super::ui;
use super::ui::command_palette::{extract_commands, filter_commands, PaletteEntry};

const ENV_PATTERN: &str = r"\$\{\s*(?P<key>\S+)\s*\}";
const ANSI_ESCAPE_PATTERN: &str = r"\x1b\[[0-9;]*[a-zA-Z]";
const MAX_LOG_ENTRIES: usize = 1000;

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub message: String,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            timestamp,
            level,
            message: message.into(),
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Info, message)
    }

    pub fn warn(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Warn, message)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Error, message)
    }

    pub fn debug(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Debug, message)
    }
}

#[allow(dead_code)]
pub struct TuiApp {
    pub ui_state: UiState,
    pub command_state: CommandState,
    pub chain_state: Arc<RwLock<ChainState>>,
    pub config: GlobalConfig,
    pub config_file: PathBuf,
    pub rpc_client: HttpRpcClient,
    pub raw_rpc_client: RawHttpRpcClient,
    pub plugin_mgr: PluginManager,
    pub key_store: KeyStore,
    pub should_quit: bool,
    pub ckb_cli_dir: PathBuf,
    parser: clap::App<'static>,
    genesis_info: Option<GenesisInfo>,
    env_regex: Regex,
    ansi_regex: Regex,
    completer: TuiCompleter,
    pub current_completions: Vec<Completion>,
    pub history_search_mode: bool,
    pub history_search_query: String,
    pub history_search_matches: Vec<String>,
    pub history_search_index: usize,
    pub logs: VecDeque<LogEntry>,
    pub palette_commands: Vec<PaletteEntry>,
}

impl TuiApp {
    pub fn new(
        ckb_cli_dir: PathBuf,
        config: GlobalConfig,
        plugin_mgr: PluginManager,
        key_store: KeyStore,
    ) -> Result<Self, String> {
        let rpc_client = HttpRpcClient::new(config.get_url().to_string());
        let raw_rpc_client = RawHttpRpcClient::new(config.get_url());

        let mut config_file = ckb_cli_dir.clone();
        config_file.push("config");

        let parser = build_interactive();
        let env_regex = Regex::new(ENV_PATTERN).map_err(|e| e.to_string())?;
        let ansi_regex = Regex::new(ANSI_ESCAPE_PATTERN).map_err(|e| e.to_string())?;
        let completer = TuiCompleter::new(&parser);
        let palette_commands = extract_commands(&parser);

        Ok(Self {
            ui_state: UiState::default(),
            command_state: CommandState::new(&ckb_cli_dir)?,
            chain_state: Arc::new(RwLock::new(ChainState::default())),
            config,
            config_file,
            rpc_client,
            raw_rpc_client,
            plugin_mgr,
            key_store,
            should_quit: false,
            ckb_cli_dir,
            parser,
            genesis_info: None,
            env_regex,
            ansi_regex,
            completer,
            current_completions: Vec::new(),
            history_search_mode: false,
            history_search_query: String::new(),
            history_search_matches: Vec::new(),
            history_search_index: 0,
            logs: VecDeque::new(),
            palette_commands,
        })
    }

    pub fn add_log(&mut self, entry: LogEntry) {
        if self.logs.len() >= MAX_LOG_ENTRIES {
            self.logs.pop_front();
        }
        self.logs.push_back(entry);
        self.ui_state.logs_scroll = usize::MAX;
    }

    fn strip_ansi(&self, s: &str) -> String {
        self.ansi_regex.replace_all(s, "").to_string()
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), String> {
        self.add_log(LogEntry::info(format!(
            "CKB CLI Modern TUI started (RPC: {})",
            self.config.get_url()
        )));

        let (event_tx, event_rx) = std::sync::mpsc::channel::<AppEvent>();
        let shutdown = Arc::new(AtomicBool::new(false));

        let chain_state = Arc::clone(&self.chain_state);
        let rpc_url = self.config.get_url().to_string();
        let event_tx_clone = event_tx.clone();
        let shutdown_chain = Arc::clone(&shutdown);

        std::thread::spawn(move || {
            let mut rpc_client = HttpRpcClient::new(rpc_url);
            while !shutdown_chain.load(Ordering::Relaxed) {
                if let Ok(state) = fetch_chain_state(&mut rpc_client) {
                    if let Ok(mut chain) = chain_state.write() {
                        *chain = state.clone();
                    }
                    let _ = event_tx_clone.send(AppEvent::ChainUpdate(state));
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        });

        let shutdown_event = Arc::clone(&shutdown);
        std::thread::spawn(move || {
            while !shutdown_event.load(Ordering::Relaxed) {
                if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                    if let Ok(evt) = event::read() {
                        let app_event = match evt {
                            Event::Key(key) => AppEvent::Key(key),
                            Event::Mouse(mouse) => AppEvent::Mouse(mouse),
                            Event::Resize(w, h) => AppEvent::Resize(w, h),
                            _ => continue,
                        };
                        if event_tx.send(app_event).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        while !self.should_quit {
            let chain_state = self.chain_state.read().map_err(|e| e.to_string())?.clone();

            terminal
                .draw(|frame| ui::render(frame, self, &chain_state))
                .map_err(|e| format!("Failed to draw: {}", e))?;

            match event_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(AppEvent::Key(key)) => self.handle_key_event(key),
                Ok(AppEvent::ChainUpdate(state)) => {
                    let prev_height = chain_state.height;
                    if state.height != prev_height && prev_height > 0 {
                        self.add_log(LogEntry::debug(format!(
                            "Chain updated: height {} → {} (epoch {})",
                            prev_height, state.height, state.epoch
                        )));
                    }
                }
                Ok(AppEvent::Resize(_, _)) => {}
                Ok(AppEvent::Mouse(mouse)) => self.handle_mouse_event(mouse),
                Ok(AppEvent::Tick) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        shutdown.store(true, Ordering::Relaxed);

        if let Err(e) = self.command_state.save_history() {
            eprintln!("Warning: Failed to save command history: {}", e);
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) {
        if self.history_search_mode {
            self.handle_history_search_key(key);
            return;
        }

        if self.ui_state.show_palette {
            self.handle_palette_key(key);
            return;
        }

        if self.ui_state.output_search_mode {
            self.handle_output_search_key(key);
            return;
        }

        if self.ui_state.show_completion {
            match key.code {
                KeyCode::Tab => {
                    if !self.current_completions.is_empty() {
                        self.ui_state.completion_index =
                            (self.ui_state.completion_index + 1) % self.current_completions.len();
                    }
                    return;
                }
                KeyCode::BackTab => {
                    if !self.current_completions.is_empty() {
                        if self.ui_state.completion_index == 0 {
                            self.ui_state.completion_index = self.current_completions.len() - 1;
                        } else {
                            self.ui_state.completion_index -= 1;
                        }
                    }
                    return;
                }
                KeyCode::Enter => {
                    self.accept_completion();
                    return;
                }
                KeyCode::Esc => {
                    self.ui_state.show_completion = false;
                    self.current_completions.clear();
                    return;
                }
                KeyCode::Up => {
                    if self.ui_state.completion_index > 0 {
                        self.ui_state.completion_index -= 1;
                    }
                    return;
                }
                KeyCode::Down => {
                    if self.ui_state.completion_index
                        < self.current_completions.len().saturating_sub(1)
                    {
                        self.ui_state.completion_index += 1;
                    }
                    return;
                }
                _ => {
                    self.ui_state.show_completion = false;
                    self.current_completions.clear();
                }
            }
        }

        if self.ui_state.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') => {
                    self.ui_state.show_help = false;
                }
                _ => {}
            }
            return;
        }

        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.should_quit = true;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                self.command_state.clear_output();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
                self.start_history_search();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                self.open_palette();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
                self.export_output();
            }
            (_, KeyCode::F(1)) => {
                self.ui_state.focused_pane = Pane::Sidebar;
            }
            (_, KeyCode::F(2)) => {
                self.ui_state.focused_pane = Pane::Output;
            }
            (_, KeyCode::F(3)) => {
                self.ui_state.focused_pane = Pane::Input;
            }
            (KeyModifiers::ALT, KeyCode::Char('1')) => {
                self.ui_state.current_tab = Tab::Command;
            }
            (KeyModifiers::ALT, KeyCode::Char('2')) => {
                self.ui_state.current_tab = Tab::Logs;
            }
            (KeyModifiers::CONTROL, KeyCode::Tab) => {
                self.ui_state.current_tab = match self.ui_state.current_tab {
                    Tab::Command => Tab::Logs,
                    Tab::Logs => Tab::Command,
                };
            }
            (_, KeyCode::Char('?')) => {
                self.ui_state.show_help = true;
            }
            (_, KeyCode::Char('/')) if self.ui_state.focused_pane == Pane::Output => {
                self.start_output_search();
            }
            (_, KeyCode::Tab) => {
                self.trigger_completion();
            }
            (_, KeyCode::Enter) => {
                self.execute_command();
            }
            (_, KeyCode::Char(c)) if self.ui_state.focused_pane == Pane::Input => {
                self.command_state.input.push(c);
            }
            (_, KeyCode::Backspace) if self.ui_state.focused_pane == Pane::Input => {
                self.command_state.input.pop();
            }
            (_, KeyCode::Up) if self.ui_state.focused_pane == Pane::Input => {
                self.command_state.navigate_history_up();
            }
            (_, KeyCode::Down) if self.ui_state.focused_pane == Pane::Input => {
                self.command_state.navigate_history_down();
            }
            (_, KeyCode::Up)
            | (_, KeyCode::Down)
            | (_, KeyCode::PageUp)
            | (_, KeyCode::PageDown)
            | (_, KeyCode::Home)
            | (_, KeyCode::End)
            | (_, KeyCode::Char('j'))
            | (_, KeyCode::Char('k'))
            | (_, KeyCode::Char('g'))
            | (_, KeyCode::Char('G'))
                if self.ui_state.focused_pane == Pane::Output =>
            {
                handle_output_scroll(
                    self.ui_state.focused_pane,
                    &mut self.ui_state.output_scroll,
                    key,
                );
            }
            _ => {}
        }
    }

    fn handle_history_search_key(&mut self, key: event::KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) | (_, KeyCode::Esc) => {
                self.history_search_mode = false;
                self.history_search_query.clear();
                self.history_search_matches.clear();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('r')) | (_, KeyCode::Up) => {
                if !self.history_search_matches.is_empty() {
                    self.history_search_index = (self.history_search_index + 1)
                        .min(self.history_search_matches.len().saturating_sub(1));
                }
            }
            (_, KeyCode::Down) => {
                if self.history_search_index > 0 {
                    self.history_search_index -= 1;
                }
            }
            (_, KeyCode::Enter) => {
                if let Some(selected) = self.history_search_matches.get(self.history_search_index) {
                    self.command_state.input = selected.clone();
                }
                self.history_search_mode = false;
                self.history_search_query.clear();
                self.history_search_matches.clear();
            }
            (_, KeyCode::Char(c)) => {
                self.history_search_query.push(c);
                self.update_history_search();
            }
            (_, KeyCode::Backspace) => {
                self.history_search_query.pop();
                self.update_history_search();
            }
            _ => {}
        }
    }

    fn start_history_search(&mut self) {
        self.history_search_mode = true;
        self.history_search_query.clear();
        self.history_search_index = 0;
        self.update_history_search();
    }

    fn update_history_search(&mut self) {
        let query_lower = self.history_search_query.to_lowercase();
        self.history_search_matches = self
            .command_state
            .history
            .iter()
            .rev()
            .filter(|cmd| cmd.to_lowercase().contains(&query_lower))
            .cloned()
            .collect();
        self.history_search_index = 0;
    }

    fn open_palette(&mut self) {
        self.ui_state.show_palette = true;
        self.ui_state.palette_query.clear();
        self.ui_state.palette_index = 0;
    }

    fn handle_palette_key(&mut self, key: event::KeyEvent) {
        let filtered = filter_commands(&self.palette_commands, &self.ui_state.palette_query);

        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) | (_, KeyCode::Esc) => {
                self.ui_state.show_palette = false;
                self.ui_state.palette_query.clear();
            }
            (_, KeyCode::Up) => {
                if self.ui_state.palette_index > 0 {
                    self.ui_state.palette_index -= 1;
                }
            }
            (_, KeyCode::Down) => {
                if self.ui_state.palette_index < filtered.len().saturating_sub(1) {
                    self.ui_state.palette_index += 1;
                }
            }
            (_, KeyCode::Enter) => {
                if let Some(entry) = filtered.get(self.ui_state.palette_index) {
                    self.command_state.input = format!("{} ", entry.command);
                }
                self.ui_state.show_palette = false;
                self.ui_state.palette_query.clear();
            }
            (_, KeyCode::Char(c)) => {
                self.ui_state.palette_query.push(c);
                self.ui_state.palette_index = 0;
            }
            (_, KeyCode::Backspace) => {
                self.ui_state.palette_query.pop();
                self.ui_state.palette_index = 0;
            }
            _ => {}
        }
    }

    fn start_output_search(&mut self) {
        self.ui_state.output_search_mode = true;
        self.ui_state.output_search_query.clear();
    }

    fn handle_output_search_key(&mut self, key: event::KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) | (_, KeyCode::Esc) => {
                self.ui_state.output_search_mode = false;
                self.ui_state.output_search_query.clear();
            }
            (_, KeyCode::Enter) => {
                self.ui_state.output_search_mode = false;
            }
            (_, KeyCode::Char(c)) => {
                self.ui_state.output_search_query.push(c);
            }
            (_, KeyCode::Backspace) => {
                self.ui_state.output_search_query.pop();
            }
            _ => {}
        }
    }

    fn export_output(&mut self) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut export_dir = self.ckb_cli_dir.clone();
        export_dir.push("exports");

        if let Err(e) = std::fs::create_dir_all(&export_dir) {
            self.add_log(LogEntry::error(format!(
                "Failed to create export dir: {}",
                e
            )));
            return;
        }

        let filename = format!("output_{}.txt", timestamp);
        let filepath = export_dir.join(&filename);

        match File::create(&filepath) {
            Ok(mut file) => {
                let mut content = String::new();
                for entry in &self.command_state.output_buffer {
                    content.push_str(&format!("> {}\n", entry.command));
                    content.push_str(&entry.result);
                    content.push_str("\n\n");
                }

                match file.write_all(content.as_bytes()) {
                    Ok(_) => {
                        self.add_log(LogEntry::info(format!(
                            "Output exported to {}",
                            filepath.display()
                        )));
                    }
                    Err(e) => {
                        self.add_log(LogEntry::error(format!("Failed to write: {}", e)));
                    }
                }
            }
            Err(e) => {
                self.add_log(LogEntry::error(format!("Failed to create file: {}", e)));
            }
        }
    }

    fn trigger_completion(&mut self) {
        self.current_completions = self
            .completer
            .get_completions(&self.command_state.input, &self.parser);

        if !self.current_completions.is_empty() {
            self.ui_state.show_completion = true;
            self.ui_state.completion_index = 0;
        }
    }

    fn accept_completion(&mut self) {
        if let Some(completion) = self.current_completions.get(self.ui_state.completion_index) {
            let input = &self.command_state.input;
            let word_start = input
                .rfind(|c: char| c.is_whitespace())
                .map(|i| i + 1)
                .unwrap_or(0);

            self.command_state.input =
                format!("{}{} ", &input[..word_start], completion.replacement);
        }
        self.ui_state.show_completion = false;
        self.current_completions.clear();
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(_) => {
                if let Some(pane) = self.ui_state.layout.pane_at(mouse.column, mouse.row) {
                    self.ui_state.focused_pane = pane;
                }
            }
            MouseEventKind::ScrollUp => {
                if self
                    .ui_state
                    .layout
                    .contains(Pane::Output, mouse.column, mouse.row)
                {
                    self.ui_state.output_scroll = self.ui_state.output_scroll.saturating_sub(3);
                }
            }
            MouseEventKind::ScrollDown => {
                if self
                    .ui_state
                    .layout
                    .contains(Pane::Output, mouse.column, mouse.row)
                {
                    self.ui_state.output_scroll = self
                        .ui_state
                        .output_scroll
                        .saturating_add(3)
                        .min(self.ui_state.max_output_scroll);
                }
            }
            _ => {}
        }
    }

    fn execute_command(&mut self) {
        let input = self.command_state.input.trim().to_string();
        if input.is_empty() {
            return;
        }

        if input == "exit" || input == "quit" {
            self.should_quit = true;
            return;
        }

        self.command_state.add_to_history(input.clone());
        self.command_state.input.clear();

        self.add_log(LogEntry::info(format!("Executing: {}", input)));

        match self.handle_command(&input) {
            Ok((output, success)) => {
                let clean_output = self.strip_ansi(&output);
                if success {
                    self.add_log(LogEntry::info(format!(
                        "Command completed successfully ({} bytes output)",
                        clean_output.len()
                    )));
                } else {
                    self.add_log(LogEntry::warn("Command returned with warning"));
                }
                self.command_state.add_output(input, clean_output, success);
            }
            Err(err) => {
                let clean_err = self.strip_ansi(&err);
                self.add_log(LogEntry::error(format!("Command failed: {}", clean_err)));
                self.command_state.add_output(input, clean_err, false);
            }
        }

        self.ui_state.output_scroll = usize::MAX;
    }

    fn genesis_info(&mut self) -> Result<GenesisInfo, String> {
        if self.genesis_info.is_none() {
            self.genesis_info = Some(get_genesis_info(&None, &mut self.rpc_client)?);
        }
        Ok(self.genesis_info.clone().unwrap())
    }

    fn handle_command(&mut self, line: &str) -> Result<(String, bool), String> {
        let args = shell_words::split(self.config.replace_cmd(&self.env_regex, line).as_str())
            .map_err(|e| e.to_string())?;

        if args.is_empty() {
            return Ok((String::new(), true));
        }

        let format = self.config.output_format();
        let use_ansi_colors = false;
        let debug = self.config.debug();

        let current_cmd_name = &args[0];

        if self
            .plugin_mgr
            .sub_commands()
            .contains_key(current_cmd_name.as_str())
        {
            let rest_args = line[current_cmd_name.len()..].to_string();
            let resp = self
                .plugin_mgr
                .sub_command(current_cmd_name.as_str(), rest_args)?;
            return Ok((resp.render(format, use_ansi_colors), true));
        }

        let parser = self.parser.clone();
        match parser.try_get_matches_from(args) {
            Ok(matches) => match matches.subcommand() {
                ("config", Some(m)) => {
                    if let Some(url) = m.value_of("url") {
                        self.config.set_url(url.to_string());
                        self.rpc_client = HttpRpcClient::new(self.config.get_url().to_string());
                        self.raw_rpc_client = RawHttpRpcClient::new(self.config.get_url());
                        self.config
                            .set_network(get_network_type(&mut self.rpc_client).ok());
                        self.genesis_info = None;
                    }
                    if m.is_present("color") {
                        self.config.switch_color();
                    }
                    if let Some(fmt) = m.value_of("output-format") {
                        let output_format =
                            OutputFormat::from_str(fmt).unwrap_or(OutputFormat::Yaml);
                        self.config.set_output_format(output_format);
                    }
                    if m.is_present("debug") {
                        self.config.switch_debug();
                    }
                    let _ = self.config.save(self.config_file.as_path());
                    Ok(("Configuration updated".to_string(), true))
                }
                ("info", _) => {
                    let info = format!(
                        "url: {}\nnetwork: {:?}\noutput_format: {:?}\ncolor: {}\ndebug: {}",
                        self.config.get_url(),
                        self.config.network(),
                        self.config.output_format(),
                        self.config.color(),
                        self.config.debug()
                    );
                    Ok((info, true))
                }
                ("rpc", Some(sub_matches)) => {
                    let output = RpcSubCommand::new(&mut self.rpc_client, &mut self.raw_rpc_client)
                        .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, use_ansi_colors), true))
                }
                ("account", Some(sub_matches)) => {
                    let output = AccountSubCommand::new(&mut self.plugin_mgr, &mut self.key_store)
                        .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, use_ansi_colors), true))
                }
                ("mock-tx", Some(sub_matches)) => {
                    let genesis_info = self.genesis_info().ok();
                    let output = MockTxSubCommand::new(
                        &mut self.rpc_client,
                        &mut self.plugin_mgr,
                        genesis_info,
                    )
                    .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, use_ansi_colors), true))
                }
                ("tx", Some(sub_matches)) => {
                    let genesis_info = self.genesis_info().ok();
                    let output =
                        TxSubCommand::new(&mut self.rpc_client, &mut self.plugin_mgr, genesis_info)
                            .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, use_ansi_colors), true))
                }
                ("util", Some(sub_matches)) => {
                    let output = UtilSubCommand::new(&mut self.rpc_client, &mut self.plugin_mgr)
                        .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, use_ansi_colors), true))
                }
                ("plugin", Some(sub_matches)) => {
                    let output =
                        PluginSubCommand::new(&mut self.plugin_mgr).process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, use_ansi_colors), true))
                }
                ("molecule", Some(sub_matches)) => {
                    let output = MoleculeSubCommand::new().process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, use_ansi_colors), true))
                }
                ("wallet", Some(sub_matches)) => {
                    let genesis_info = self.genesis_info()?;
                    let output = WalletSubCommand::new(
                        &mut self.rpc_client,
                        &mut self.plugin_mgr,
                        Some(genesis_info),
                    )
                    .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, use_ansi_colors), true))
                }
                ("dao", Some(sub_matches)) => {
                    let genesis_info = self.genesis_info()?;
                    let output = DAOSubCommand::new(
                        &mut self.rpc_client,
                        &mut self.plugin_mgr,
                        genesis_info,
                    )
                    .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, use_ansi_colors), true))
                }
                ("sudt", Some(sub_matches)) => {
                    let genesis_info = self.genesis_info()?;
                    let output = SudtSubCommand::new(
                        &mut self.rpc_client,
                        &mut self.plugin_mgr,
                        genesis_info,
                    )
                    .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, use_ansi_colors), true))
                }
                ("deploy", Some(sub_matches)) => {
                    let genesis_info = self.genesis_info()?;
                    let output = DeploySubCommand::new(
                        &mut self.rpc_client,
                        &mut self.plugin_mgr,
                        genesis_info,
                    )
                    .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, use_ansi_colors), true))
                }
                _ => Ok((format!("Unknown command: {}", line), false)),
            },
            Err(err) => Err(err.to_string()),
        }
    }
}

fn output_to_string(
    output: &crate::subcommands::Output,
    format: OutputFormat,
    color: bool,
) -> String {
    output.render(format, color)
}

fn fetch_chain_state(rpc_client: &mut HttpRpcClient) -> Result<ChainState, String> {
    let tip = rpc_client.get_tip_block_number().ok();
    let blockchain_info = rpc_client.get_blockchain_info().ok();
    let peers = rpc_client.get_peers().ok();

    let (epoch, is_syncing) = if let Some(info) = blockchain_info {
        (info.epoch, info.is_initial_block_download)
    } else {
        (0, false)
    };

    Ok(ChainState {
        height: tip.unwrap_or(0),
        epoch,
        peers: peers.map(|p| p.len()).unwrap_or(0),
        sync_progress: if is_syncing { 0.5 } else { 1.0 },
        last_block_time: 0,
        is_syncing,
        last_updated: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    })
}

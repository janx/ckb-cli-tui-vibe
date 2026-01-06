use std::io::Stdout;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};

use ckb_signer::KeyStore;

use crate::plugin::PluginManager;
use crate::utils::config::GlobalConfig;
use crate::utils::rpc::{HttpRpcClient, RawHttpRpcClient};

use super::state::{ChainState, CommandState, UiState};
use super::ui;

#[allow(dead_code)]
pub struct TuiApp {
    pub ui_state: UiState,
    pub command_state: CommandState,
    pub chain_state: ChainState,
    pub config: GlobalConfig,
    pub rpc_client: HttpRpcClient,
    pub raw_rpc_client: RawHttpRpcClient,
    pub plugin_mgr: PluginManager,
    pub key_store: KeyStore,
    pub should_quit: bool,
    pub ckb_cli_dir: PathBuf,
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

        Ok(Self {
            ui_state: UiState::default(),
            command_state: CommandState::new(&ckb_cli_dir)?,
            chain_state: ChainState::default(),
            config,
            rpc_client,
            raw_rpc_client,
            plugin_mgr,
            key_store,
            should_quit: false,
            ckb_cli_dir,
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), String> {
        while !self.should_quit {
            terminal
                .draw(|frame| ui::render(frame, self))
                .map_err(|e| format!("Failed to draw: {}", e))?;

            if event::poll(Duration::from_millis(100)).map_err(|e| e.to_string())? {
                if let Event::Key(key) = event::read().map_err(|e| e.to_string())? {
                    self.handle_key_event(key);
                }
            }
        }

        if let Err(e) = self.command_state.save_history() {
            eprintln!("Warning: Failed to save command history: {}", e);
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.should_quit = true;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                self.command_state.clear_output();
            }
            (_, KeyCode::Enter) => {
                self.execute_command();
            }
            (_, KeyCode::Char(c)) => {
                self.command_state.input.push(c);
            }
            (_, KeyCode::Backspace) => {
                self.command_state.input.pop();
            }
            (_, KeyCode::Up) => {
                self.command_state.navigate_history_up();
            }
            (_, KeyCode::Down) => {
                self.command_state.navigate_history_down();
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

        let output = format!("Executed: {}", input);
        self.command_state.add_output(input, output, true);
        self.command_state.input.clear();
    }
}

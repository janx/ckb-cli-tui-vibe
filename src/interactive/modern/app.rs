use std::io::Stdout;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};
use regex::Regex;
use tokio::sync::mpsc;

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
    printer::{ColorWhen, OutputFormat, Printable},
    rpc::{HttpRpcClient, RawHttpRpcClient},
};

use super::event::AppEvent;
use super::state::{ChainState, CommandState, UiState};
use super::ui;

const ENV_PATTERN: &str = r"\$\{\s*(?P<key>\S+)\s*\}";

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
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), String> {
        let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(100);

        let chain_state = Arc::clone(&self.chain_state);
        let rpc_url = self.config.get_url().to_string();
        let event_tx_clone = event_tx.clone();

        std::thread::spawn(move || {
            let mut rpc_client = HttpRpcClient::new(rpc_url);
            loop {
                if let Ok(state) = fetch_chain_state(&mut rpc_client) {
                    if let Ok(mut chain) = chain_state.write() {
                        *chain = state.clone();
                    }
                    let _ = event_tx_clone.blocking_send(AppEvent::ChainUpdate(state));
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        });

        std::thread::spawn(move || loop {
            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(evt) = event::read() {
                    let app_event = match evt {
                        Event::Key(key) => AppEvent::Key(key),
                        Event::Mouse(mouse) => AppEvent::Mouse(mouse),
                        Event::Resize(w, h) => AppEvent::Resize(w, h),
                        _ => continue,
                    };
                    if event_tx.blocking_send(app_event).is_err() {
                        break;
                    }
                }
            }
        });

        while !self.should_quit {
            {
                let chain_state = self.chain_state.read().map_err(|e| e.to_string())?;
                terminal
                    .draw(|frame| ui::render(frame, self, &chain_state))
                    .map_err(|e| format!("Failed to draw: {}", e))?;
            }

            match event_rx.blocking_recv() {
                Some(AppEvent::Key(key)) => self.handle_key_event(key),
                Some(AppEvent::ChainUpdate(_)) => {}
                Some(AppEvent::Resize(_, _)) => {}
                Some(AppEvent::Mouse(_)) => {}
                Some(AppEvent::Tick) => {}
                None => break,
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
        self.command_state.input.clear();

        match self.handle_command(&input) {
            Ok((output, success)) => {
                self.command_state.add_output(input, output, success);
            }
            Err(err) => {
                self.command_state.add_output(input, err, false);
            }
        }
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
        let color = ColorWhen::new(self.config.color()).color();
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
            return Ok((resp.render(format, color), true));
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
                    Ok((output_to_string(&output, format, color), true))
                }
                ("account", Some(sub_matches)) => {
                    let output = AccountSubCommand::new(&mut self.plugin_mgr, &mut self.key_store)
                        .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, color), true))
                }
                ("mock-tx", Some(sub_matches)) => {
                    let genesis_info = self.genesis_info().ok();
                    let output = MockTxSubCommand::new(
                        &mut self.rpc_client,
                        &mut self.plugin_mgr,
                        genesis_info,
                    )
                    .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, color), true))
                }
                ("tx", Some(sub_matches)) => {
                    let genesis_info = self.genesis_info().ok();
                    let output =
                        TxSubCommand::new(&mut self.rpc_client, &mut self.plugin_mgr, genesis_info)
                            .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, color), true))
                }
                ("util", Some(sub_matches)) => {
                    let output = UtilSubCommand::new(&mut self.rpc_client, &mut self.plugin_mgr)
                        .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, color), true))
                }
                ("plugin", Some(sub_matches)) => {
                    let output =
                        PluginSubCommand::new(&mut self.plugin_mgr).process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, color), true))
                }
                ("molecule", Some(sub_matches)) => {
                    let output = MoleculeSubCommand::new().process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, color), true))
                }
                ("wallet", Some(sub_matches)) => {
                    let genesis_info = self.genesis_info()?;
                    let output = WalletSubCommand::new(
                        &mut self.rpc_client,
                        &mut self.plugin_mgr,
                        Some(genesis_info),
                    )
                    .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, color), true))
                }
                ("dao", Some(sub_matches)) => {
                    let genesis_info = self.genesis_info()?;
                    let output = DAOSubCommand::new(
                        &mut self.rpc_client,
                        &mut self.plugin_mgr,
                        genesis_info,
                    )
                    .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, color), true))
                }
                ("sudt", Some(sub_matches)) => {
                    let genesis_info = self.genesis_info()?;
                    let output = SudtSubCommand::new(
                        &mut self.rpc_client,
                        &mut self.plugin_mgr,
                        genesis_info,
                    )
                    .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, color), true))
                }
                ("deploy", Some(sub_matches)) => {
                    let genesis_info = self.genesis_info()?;
                    let output = DeploySubCommand::new(
                        &mut self.rpc_client,
                        &mut self.plugin_mgr,
                        genesis_info,
                    )
                    .process(sub_matches, debug)?;
                    Ok((output_to_string(&output, format, color), true))
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

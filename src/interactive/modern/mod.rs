mod app;
mod event;
mod state;
mod ui;

use std::io::{self, Stdout};
use std::panic;
use std::path::PathBuf;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use ckb_signer::KeyStore;

use crate::plugin::PluginManager;
use crate::utils::config::GlobalConfig;

use app::TuiApp;

type Tui = Terminal<CrosstermBackend<Stdout>>;

fn init_terminal() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Tui) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()
}

fn install_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));
}

pub fn run(
    ckb_cli_dir: PathBuf,
    config: GlobalConfig,
    plugin_mgr: PluginManager,
    key_store: KeyStore,
) -> Result<(), String> {
    install_panic_hook();

    let mut terminal = init_terminal().map_err(|e| format!("Failed to init terminal: {}", e))?;

    let mut app = TuiApp::new(ckb_cli_dir, config, plugin_mgr, key_store)?;
    let result = app.run(&mut terminal);

    restore_terminal(&mut terminal).map_err(|e| format!("Failed to restore terminal: {}", e))?;

    result
}

mod classic;
#[cfg(unix)]
pub mod modern;

pub use classic::InteractiveEnv;

use std::path::PathBuf;

use ckb_signer::KeyStore;

use crate::plugin::PluginManager;
use crate::utils::config::GlobalConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractiveMode {
    Classic,
    #[cfg(unix)]
    Modern,
}

// Can't use #[derive(Default)] because default varies by platform
#[allow(clippy::derivable_impls)]
impl Default for InteractiveMode {
    fn default() -> Self {
        #[cfg(unix)]
        {
            InteractiveMode::Modern
        }
        #[cfg(not(unix))]
        {
            InteractiveMode::Classic
        }
    }
}

pub fn start_interactive(
    mode: InteractiveMode,
    ckb_cli_dir: PathBuf,
    config: GlobalConfig,
    plugin_mgr: PluginManager,
    key_store: KeyStore,
) -> Result<(), String> {
    match mode {
        InteractiveMode::Classic => {
            InteractiveEnv::from_config(ckb_cli_dir, config, plugin_mgr, key_store)?.start()
        }
        #[cfg(unix)]
        InteractiveMode::Modern => modern::run(ckb_cli_dir, config, plugin_mgr, key_store),
    }
}

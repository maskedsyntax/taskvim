mod domain;
mod error;
mod storage;
mod core;
mod ui;
mod config;

use crate::storage::SqliteStorage;
use crate::core::AppState;
use crate::ui::Tui;
use crate::config::LuaConfig;
use crate::error::Result;

fn main() -> Result<()> {
    // Initialize Lua config
    let lua_config = LuaConfig::new()?;
    let _ = lua_config.load_user_config();

    // Initialize storage
    let storage = SqliteStorage::new("taskvim.db")?;
    
    // Initialize app state
    let mut state = AppState::new(storage, lua_config.get_config())?;
    
    // Initialize and run TUI
    let mut tui = Tui::new()?;
    tui.run(&mut state)?;
    
    Ok(())
}

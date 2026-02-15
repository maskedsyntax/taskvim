mod domain;
mod error;
mod storage;
mod core;
mod ui;

use crate::storage::SqliteStorage;
use crate::core::AppState;
use crate::ui::Tui;
use crate::error::Result;

fn main() -> Result<()> {
    // Initialize storage
    let storage = SqliteStorage::new("taskvim.db")?;
    
    // Initialize app state
    let mut state = AppState::new(storage)?;
    
    // Initialize and run TUI
    let mut tui = Tui::new()?;
    tui.run(&mut state)?;
    
    Ok(())
}

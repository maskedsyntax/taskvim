use mlua::{Lua, Table};
use crate::error::Result;
use std::path::PathBuf;
use directories::ProjectDirs;

pub struct LuaConfig {
    lua: Lua,
}

impl LuaConfig {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        let config = Self { lua };
        config.init_api()?;
        Ok(config)
    }

    fn init_api(&self) -> Result<()> {
        let globals = self.lua.globals();

        // set table
        let set = self.lua.create_table()?;
        set.set("theme", self.lua.create_function(|_, theme: String| {
            println!("Setting theme to: {}", theme);
            Ok(())
        })?)?;
        globals.set("set", set)?;

        // map function
        globals.set("map", self.lua.create_function(|_, (mode, key, action): (String, String, String)| {
            println!("Mapping {} in mode {} to {}", key, mode, action);
            Ok(())
        })?)?;

        Ok(())
    }

    pub fn load_user_config(&self) -> Result<()> {
        if let Some(dirs) = ProjectDirs::from("com", "maskedsyntax", "taskvim") {
            let config_dir = dirs.config_dir();
            let init_lua = config_dir.join("init.lua");
            
            if init_lua.exists() {
                let script = std::fs::read_to_string(init_lua)?;
                self.lua.load(&script).exec()?;
            }
        }
        Ok(())
    }
}

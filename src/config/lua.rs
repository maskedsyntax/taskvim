use crate::error::Result;
use crate::core::keymap::{Keymap, KeyCombination};
use crate::core::actions::Action;
use crate::core::state::Mode;
use mlua::Lua;
use std::sync::{Arc, Mutex};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Config {
    pub theme: String,
    pub default_priority: i32,
    pub show_sidebar: bool,
    pub keymap: Keymap,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            default_priority: 3,
            show_sidebar: true,
            keymap: Keymap::new(),
        }
    }
}

pub struct LuaConfig {
    lua: Lua,
    config: Arc<Mutex<Config>>,
}

impl LuaConfig {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        let config = Arc::new(Mutex::new(Config::default()));
        let lua_config = Self { lua, config };
        lua_config.init_api()?;
        Ok(lua_config)
    }

    pub fn get_config(&self) -> Config {
        self.config.lock().unwrap().clone()
    }

    fn init_api(&self) -> Result<()> {
        let globals = self.lua.globals();
        let config_arc = Arc::clone(&self.config);

        // set table
        let set = self.lua.create_table()?;
        
        let c_theme = Arc::clone(&config_arc);
        set.set("theme", self.lua.create_function(move |_, theme: String| {
            let mut c = c_theme.lock().unwrap();
            c.theme = theme;
            Ok(())
        })?)?;

        let c_priority = Arc::clone(&config_arc);
        set.set("default_priority", self.lua.create_function(move |_, priority: i32| {
            let mut c = c_priority.lock().unwrap();
            c.default_priority = priority;
            Ok(())
        })?)?;

        let c_sidebar = Arc::clone(&config_arc);
        set.set("sidebar", self.lua.create_function(move |_, show: bool| {
            let mut c = c_sidebar.lock().unwrap();
            c.show_sidebar = show;
            Ok(())
        })?)?;

        globals.set("set", set)?;

        // map function
        let c_map = Arc::clone(&config_arc);
        globals.set("map", self.lua.create_function(move |_, (mode_str, key_str, action_str): (String, String, String)| {
            let mode = match mode_str.as_str() {
                "n" | "normal" => Mode::Normal,
                "v" | "visual" => Mode::Visual,
                "s" | "stats" => Mode::Stats,
                _ => return Ok(()), // Ignore unknown modes
            };

            if let Some(combo) = KeyCombination::from_str(&key_str) {
                if let Ok(action) = Action::from_str(&action_str) {
                    let mut c = c_map.lock().unwrap();
                    c.keymap.mappings.entry(mode).or_default().insert(combo, action);
                }
            }
            Ok(())
        })?)?;

        Ok(())
    }

    pub fn load_user_config(&self) -> Result<()> {
        let dirs = directories::ProjectDirs::from("com", "maskedsyntax", "taskvim");
        if let Some(dirs) = dirs {
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

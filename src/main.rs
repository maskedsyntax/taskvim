mod db;
mod domain;
mod state;
mod ui;
mod config;

use gtk::prelude::*;
use gtk::{Application, CssProvider, StyleContext, Settings};
use std::rc::Rc;
use std::cell::RefCell;
use db::Db;
use state::AppState;
use config::Config;
use ui::themes;

fn main() {
    let app = Application::builder()
        .application_id("com.maskedsyntax.taskit")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let provider = CssProvider::new();
    
    // Load config
    let config = Config::load();
    
    // Apply initial style
    update_style(&provider, &config);

    if let Some(screen) = gtk::gdk::Screen::default() {
        StyleContext::add_provider_for_screen(
            &screen,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let db = Db::init().expect("Failed to initialize database");
    let state = Rc::new(RefCell::new(AppState::new(db)));
    
    if let Some(settings) = Settings::default() {
        let is_dark = config.theme == "Dark";
        settings.set_gtk_application_prefer_dark_theme(is_dark);
    }
    
    state.borrow_mut().refresh();

    let config_rc = Rc::new(RefCell::new(config));
    
    let provider_clone = provider.clone();
    let update_fn = Rc::new(move |new_config: Config| {
        update_style(&provider_clone, &new_config);
        
        if let Some(settings) = Settings::default() {
             let is_dark = new_config.theme == "Dark";
             settings.set_gtk_application_prefer_dark_theme(is_dark);
        }
    });

    let window = ui::window::build(app, state, config_rc, update_fn);
    window.show_all();
}

fn update_style(provider: &CssProvider, config: &Config) {
    let theme_css = themes::get_theme_css(&config.theme);
    
    // Font handling: Try to parse font name if it comes from FontButton (e.g., "Sans 12")
    // We split by space, take the last part as size if numeric, join the rest as family.
    let font_css = if let Some(font) = &config.font {
        let parts: Vec<&str> = font.split_whitespace().collect();
        if let Some((last, rest)) = parts.split_last() {
            if let Ok(size) = last.parse::<f64>() {
                let family = rest.join(" ");
                format!("* {{ font-family: \"{}\"; font-size: {}pt; }}", family, size)
            } else {
                // Fallback if no size found or not numeric
                format!("* {{ font-family: \"{}\"; }}", font)
            }
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };

    let full_css = format!("
        {}
        {}
        .task-done {{ text-decoration: line-through; opacity: 0.6; }}
    ", theme_css, font_css);

    let _ = provider.load_from_data(full_css.as_bytes());
}

mod db;
mod domain;
mod state;
mod ui;

use gtk::prelude::*;
use gtk::Application;
use std::rc::Rc;
use std::cell::RefCell;
use db::Db;
use state::AppState;

fn main() {
    let app = Application::builder()
        .application_id("com.maskedsyntax.taskit")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let provider = gtk::CssProvider::new();
    // GTK3 CSS is slightly different, uses standard CSS properties.
    // 'alpha(currentColor, 0.1)' might not work in older GTK3 CSS parsers, but standard GTK3 supports it or rgba().
    // We'll use standard GTK style classes where possible.
    provider.load_from_data(b"
        .task-done { text-decoration: line-through; opacity: 0.6; }
        .sidebar-item { padding: 8px; border-radius: 6px; }
        .sidebar-item:hover { background-color: rgba(0,0,0,0.05); } 
        .boxed-list { border: 1px solid alpha(currentColor, 0.15); border-radius: 8px; }
        .card { border: 1px solid alpha(currentColor, 0.15); border-radius: 8px; padding: 12px; }
        .flat { background: none; border: none; box-shadow: none; }
        .suggested-action { background-color: @theme_selected_bg_color; color: @theme_selected_fg_color; }
        .title-label { font-weight: bold; font-size: 16pt; }
        .dim-label { opacity: 0.6; }
        /* Dark mode tweaks if needed */
    ").expect("Failed to load CSS");

    if let Some(screen) = gtk::gdk::Screen::default() {
        gtk::StyleContext::add_provider_for_screen(
            &screen,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let db = Db::init().expect("Failed to initialize database");
    let state = Rc::new(RefCell::new(AppState::new(db)));
    
    // Initialize theme
    let is_dark = state.borrow().is_dark_mode;
    if let Some(settings) = gtk::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(is_dark);
        settings.set_gtk_theme_name(Some(if is_dark { "Adwaita-dark" } else { "Adwaita" }));
    }

    state.borrow_mut().refresh();

    let window = ui::window::build(app, state);
    window.show_all();
}

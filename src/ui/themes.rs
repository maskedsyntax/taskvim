pub fn get_theme_css(theme_name: &str) -> String {
    // We only support "System" (which can be light or dark based on OS) or forced "Dark".
    // But honestly, "Adwaita" handles most. We just overlay our structural CSS.
    // The user wants "lean, thin, minimal".
    
    // We'll return the structural CSS + variables.
    // We rely on GTK theme colors for palette usually, but we can override for "Dark" if requested.
    
    match theme_name {
        "Dark" => dark_overrides() + &common_css(),
        _ => common_css(), // System / Light
    }
}

pub fn get_all_themes() -> Vec<&'static str> {
    vec![
        "System",
        "Dark",
    ]
}

fn common_css() -> String {
    "
    /* Global Variables */
    @define-color border_radius 6px;

    /* Base Elements */
    window, box, grid, paned {
        /* Inherit from theme */
    }

    /* Consistent Border Radius */
    button, entry, .card, .sidebar-item, .boxed-list, menu, popover {
        border-radius: 6px;
    }

    /* Sidebar Styling - Boxed & Lean */
    .sidebar {
        background-color: @theme_bg_color; /* Or a slight shade darker if possible */
    }

    .sidebar-row {
        background: transparent;
        border: none;
        padding: 0;
    }

    .sidebar-item {
        background-color: transparent;
        color: @theme_fg_color;
        border: 1px solid transparent;
        padding: 6px 10px;
        transition: all 200ms ease;
    }
    
    .sidebar-item:hover {
        background-color: alpha(@theme_fg_color, 0.05);
    }
    
    .sidebar-item:selected, .sidebar-item.selected {
        background-color: @theme_selected_bg_color;
        color: @theme_selected_fg_color;
        border: 1px solid @theme_selected_bg_color; /* Ensure it looks boxed */
    }

    /* Buttons */
    button {
        min-height: 24px; /* Thinner */
        padding: 2px 8px;
        border: 1px solid alpha(currentColor, 0.15);
        background-image: none;
        box-shadow: none;
    }
    
    button:hover {
        background-color: alpha(currentColor, 0.05);
    }
    
    button.flat {
        border-color: transparent;
        background-color: transparent;
    }

    /* Entries */
    entry {
        min-height: 28px;
        padding: 2px 6px;
        border: 1px solid alpha(currentColor, 0.15);
        background-image: none;
        box-shadow: none;
    }
    
    /* Headers */
    .title-label {
        font-weight: 800;
        font-size: 14pt;
        letter-spacing: -0.5px;
    }

    /* Dim Label */
    .dim-label {
        opacity: 0.5;
        font-weight: 600;
        font-size: 9pt;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }
    
    /* Scrollbars */
    scrollbar slider {
        min-width: 4px;
        border-radius: 10px;
    }
    ".to_string()
}

fn dark_overrides() -> String {
    // Just force standard dark colors if system doesn't do it
    "
    @define-color theme_bg_color #1e1e2e;
    @define-color theme_fg_color #cdd6f4;
    @define-color theme_selected_bg_color #89b4fa;
    @define-color theme_selected_fg_color #1e1e2e;
    
    window {
        color: #cdd6f4;
        background-color: #1e1e2e;
    }
    ".to_string()
}
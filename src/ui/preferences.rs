use gtk::prelude::*;
use gtk::{Box, Button, ComboBoxText, Dialog, FontButton, Label, Orientation, Window};
use crate::config::Config;
use crate::ui::themes;

pub fn show(parent: &Window, current_config: Config, on_update: impl Fn(Config) + 'static) {
    let dialog = Dialog::with_buttons(
        Some("Preferences"),
        Some(parent),
        gtk::DialogFlags::MODAL,
        &[("Close", gtk::ResponseType::Close)],
    );
    dialog.set_default_size(400, 300);

    let content_area = dialog.content_area();
    content_area.set_margin_top(20);
    content_area.set_margin_bottom(20);
    content_area.set_margin_start(20);
    content_area.set_margin_end(20);
    content_area.set_spacing(15);

    // Theme Selection
    let theme_box = Box::new(Orientation::Horizontal, 10);
    let theme_label = Label::new(Some("Theme:"));
    let theme_combo = ComboBoxText::new();
    
    for theme in themes::get_all_themes() {
        theme_combo.append_text(theme);
    }
    theme_combo.set_active_id(Some(&current_config.theme));
    
    // Select active by iterating since we append_text not append(id, text)
    let mut active_idx = 0;
    for (i, t) in themes::get_all_themes().iter().enumerate() {
        if *t == current_config.theme {
            active_idx = i;
            break;
        }
    }
    theme_combo.set_active(Some(active_idx as u32));

    theme_box.pack_start(&theme_label, false, false, 0);
    theme_box.pack_start(&theme_combo, true, true, 0);
    content_area.add(&theme_box);

    // Font Selection
    let font_box = Box::new(Orientation::Horizontal, 10);
    let font_label = Label::new(Some("Font:"));
    let font_btn = FontButton::new();
    if let Some(font) = &current_config.font {
        font_btn.set_font(font);
    }
    font_btn.set_use_font(true);
    font_btn.set_use_size(true);

    font_box.pack_start(&font_label, false, false, 0);
    font_box.pack_start(&font_btn, true, true, 0);
    content_area.add(&font_box);

    // Signal Handlers
    let config = std::rc::Rc::new(std::cell::RefCell::new(current_config));
    let on_update_rc = std::rc::Rc::new(on_update);

    let config_clone = config.clone();
    let on_update_clone = on_update_rc.clone();
    theme_combo.connect_changed(move |combo| {
        if let Some(txt) = combo.active_text() {
            config_clone.borrow_mut().theme = txt.to_string();
            // Auto-save and apply
            let _ = config_clone.borrow().save();
            on_update_clone(config_clone.borrow().clone());
        }
    });

    let config_clone2 = config.clone();
    let on_update_clone2 = on_update_rc.clone();
    font_btn.connect_font_set(move |btn| {
        if let Some(font) = btn.font() {
            config_clone2.borrow_mut().font = Some(font.to_string());
            let _ = config_clone2.borrow().save();
            on_update_clone2(config_clone2.borrow().clone());
        }
    });

    dialog.show_all();
    dialog.run();
    dialog.close();
}

use gtk::prelude::*;
use gtk::{Box, Button, Entry, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, Image, Popover, GestureMultiPress, Window, Separator};
use std::rc::Rc;
use std::cell::RefCell;
use crate::state::{AppState, Filter};
use crate::config::Config;
use crate::ui::{preferences, icons};

pub fn build(
    state: Rc<RefCell<AppState>>, 
    refresh_all: Rc<dyn Fn()>,
    config: Rc<RefCell<Config>>,
    on_update: Rc<dyn Fn(Config)>,
    parent_window: Window
) -> (gtk::Widget, impl Fn()) {
    let container = Box::new(Orientation::Vertical, 0);
    container.style_context().add_class("sidebar");
    
    // Header with App Title and Settings
    let header_box = Box::new(Orientation::Horizontal, 10);
    header_box.set_margin_top(16);
    header_box.set_margin_bottom(16);
    header_box.set_margin_start(16);
    header_box.set_margin_end(16);

    let app_label = Label::new(Some("TaskIt"));
    app_label.style_context().add_class("title-label");
    app_label.set_hexpand(true);
    app_label.set_xalign(0.0);

    let settings_btn = Button::new();
    if let Some(pixbuf) = icons::get_icon_pixbuf("settings", 16) {
        settings_btn.set_image(Some(&Image::from_pixbuf(Some(&pixbuf))));
    } else {
        settings_btn.set_label("S");
    }
    settings_btn.style_context().add_class("flat");
    settings_btn.style_context().add_class("circular");
    settings_btn.set_tooltip_text(Some("Settings"));

    header_box.pack_start(&app_label, true, true, 0);
    header_box.pack_start(&settings_btn, false, false, 0);
    container.pack_start(&header_box, false, false, 0);

    // Navigation List
    let nav_list = ListBox::new();
    nav_list.style_context().add_class("navigation-sidebar");
    nav_list.set_selection_mode(gtk::SelectionMode::None);

    let scrolled = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_vexpand(true);
    scrolled.add(&nav_list);

    container.pack_start(&scrolled, true, true, 0);

    // New Project Entry (Bottom)
    let new_project_box = Box::new(Orientation::Horizontal, 4);
    new_project_box.style_context().add_class("sidebar-item");
    new_project_box.set_margin_top(12);
    new_project_box.set_margin_bottom(12);
    new_project_box.set_margin_start(12);
    new_project_box.set_margin_end(12);
    
    let new_project_entry = Entry::new();
    new_project_entry.set_placeholder_text(Some("New Project..."));
    new_project_entry.set_hexpand(true);
    
    let add_btn = Button::new();
    if let Some(pixbuf) = icons::get_icon_pixbuf("plus", 16) {
        add_btn.set_image(Some(&Image::from_pixbuf(Some(&pixbuf))));
    } else {
        add_btn.set_label("+");
    }
    // Remove "flat" to give it a button shape if desired, or keep flat but ensure size
    // User said "Add button is too short". Usually implies width or visual presence.
    // Let's keep it standard button style (not flat) to match the "box" look request?
    // Or just ensure it's square.
    // add_btn.style_context().add_class("flat"); 

    new_project_box.pack_start(&new_project_entry, true, true, 0);
    new_project_box.pack_start(&add_btn, false, false, 0); // Pack false/false keeps it min size.
    container.pack_start(&new_project_box, false, false, 0);

    // Signal handlers for Settings
    let parent_clone = parent_window.clone();
    let config_clone = config.clone();
    let on_update_clone = on_update.clone();
    
    settings_btn.connect_clicked(move |_| {
        let current_conf = config_clone.borrow().clone();
        let update_cb = on_update_clone.clone();
        
        preferences::show(&parent_clone, current_conf, move |new_conf| {
            update_cb(new_conf);
        });
    });

    // Signal handlers for Add Project
    let state_clone = state.clone();
    let refresh_clone = refresh_all.clone();
    let entry_clone = new_project_entry.clone();
    let add_action = Rc::new(move || {
        let text = entry_clone.text();
        if !text.is_empty() {
            let _ = state_clone.borrow_mut().add_project(text.to_string());
            entry_clone.set_text("");
            refresh_clone();
        }
    });

    let add_action_clone = add_action.clone();
    new_project_entry.connect_activate(move |_| add_action_clone());
    add_btn.connect_clicked(move |_| add_action());

    // Refresh Logic
    let state_clone = state.clone();
    let refresh_all_clone = refresh_all.clone();
    let nav_list_clone = nav_list.clone();

    let refresh = move || {
        // Clear list
        for child in nav_list_clone.children() {
            nav_list_clone.remove(&child);
        }

        let s = state_clone.borrow();
        let active = s.active_filter.clone();

        let add_item = |label: &str, icon_name: &str, filter: Filter| {
            let row = ListBoxRow::new();
            // We want "boxed" look. ListBoxRow can be styled.
            // But we can also add a margin to the child box to make it look like a separated card.
            row.style_context().add_class("sidebar-row");
            row.set_selectable(false); // We handle selection visually
            row.set_activatable(false);

            let b = Box::new(Orientation::Horizontal, 12);
            b.style_context().add_class("sidebar-item"); // Inner box gets the styling (bg, border)
            b.set_margin_top(4);
            b.set_margin_bottom(4);
            b.set_margin_start(12);
            b.set_margin_end(12);
            
            let icon_pixbuf = icons::get_icon_pixbuf(icon_name, 18);
            let icon = if let Some(pb) = icon_pixbuf {
                Image::from_pixbuf(Some(&pb))
            } else {
                Image::from_icon_name(Some("image-missing"), gtk::IconSize::Button)
            };

            let lbl = Label::new(Some(label));
            lbl.set_hexpand(true);
            lbl.set_xalign(0.0);

            b.pack_start(&icon, false, false, 0);
            b.pack_start(&lbl, true, true, 0);
            
            // Selection Logic
            // Debug print or visual check:
            // The issue "blue bg doesn't move" -> `active == filter` must be true.
            if active == filter {
                b.style_context().add_class("selected");
            }

            // Click handler
            let filter_clone = filter.clone();
            let state_gesture = state_clone.clone();
            let refresh_gesture = refresh_all_clone.clone();
            
            // Use EventBox or Gesture on the box? 
            // ListBoxRow captures events. 
            let gesture = GestureMultiPress::new(&row); // Attach to row
            gesture.connect_released(move |_, _, _, _| {
                state_gesture.borrow_mut().active_filter = filter_clone.clone();
                refresh_gesture();
            });
            
            row.add(&b);
            nav_list_clone.add(&row);
            
            // If project, add extra buttons
            if let Filter::Project(pid) = filter {
                // Edit/Delete buttons logic (simplified for brevity, same as before but using lucide)
                 let edit_btn = Button::new();
                 if let Some(pb) = icons::get_icon_pixbuf("pencil", 14) { edit_btn.set_image(Some(&Image::from_pixbuf(Some(&pb)))); }
                 edit_btn.style_context().add_class("flat");
                 edit_btn.style_context().add_class("circular");
                 
                 let del_btn = Button::new();
                 if let Some(pb) = icons::get_icon_pixbuf("trash-2", 14) { del_btn.set_image(Some(&Image::from_pixbuf(Some(&pb)))); }
                 del_btn.style_context().add_class("flat");
                 del_btn.style_context().add_class("circular");

                 // ... (Popovers and connection logic similar to before) ...
                 // Re-implementing connection logic briefly:
                 let s_clone = state_clone.clone();
                 let r_clone = refresh_all_clone.clone();
                 let proj_name = label.to_string();
                 
                 let popover = Popover::new(Some(&edit_btn));
                 let pop_box = Box::new(Orientation::Horizontal, 4);
                 pop_box.set_margin_start(8); pop_box.set_margin_end(8); pop_box.set_margin_top(8); pop_box.set_margin_bottom(8);
                 let pop_entry = Entry::new();
                 pop_entry.set_text(&proj_name);
                 let pop_save = Button::with_label("Save");
                 pop_save.style_context().add_class("suggested-action");
                 
                 pop_box.pack_start(&pop_entry, true, true, 0);
                 pop_box.pack_start(&pop_save, false, false, 0);
                 pop_box.show_all();
                 popover.add(&pop_box);
                 
                 let edit_menu_btn = gtk::MenuButton::new();
                 if let Some(pb) = icons::get_icon_pixbuf("pencil", 14) { 
                     edit_menu_btn.set_image(Some(&Image::from_pixbuf(Some(&pb)))); 
                 }
                 edit_menu_btn.set_popover(Some(&popover));
                 edit_menu_btn.style_context().add_class("flat");
                 edit_menu_btn.style_context().add_class("circular");

                 let s_clone_ren = s_clone.clone();
                 let r_clone_ren = r_clone.clone();
                 let popover_clone = popover.clone();
                 pop_save.connect_clicked(move |_| {
                     let new_name = pop_entry.text();
                     if !new_name.is_empty() {
                         let _ = s_clone_ren.borrow_mut().update_project_name(pid, new_name.to_string());
                         r_clone_ren();
                         popover_clone.popdown();
                     }
                 });
                 
                 let s_clone2 = state_clone.clone();
                 let r_clone2 = refresh_all_clone.clone();
                 del_btn.connect_clicked(move |_| {
                     let _ = s_clone2.borrow_mut().delete_project(pid);
                     r_clone2();
                 });

                 b.pack_start(&edit_menu_btn, false, false, 0);
                 b.pack_start(&del_btn, false, false, 0);
            }
        };

        add_item("Inbox", "inbox", Filter::Inbox);
        add_item("Today", "calendar", Filter::Today);
        add_item("Upcoming", "calendar-days", Filter::Upcoming); // Using "calendar-days" as icon

        let sep_box = Box::new(Orientation::Horizontal, 0);
        sep_box.set_margin_top(12);
        sep_box.set_margin_bottom(4);
        sep_box.set_margin_start(16);
        let sep = Label::new(Some("Projects"));
        sep.style_context().add_class("dim-label");
        sep.set_xalign(0.0);
        sep_box.pack_start(&sep, true, true, 0);
        
        let row_sep = ListBoxRow::new();
        row_sep.set_activatable(false);
        row_sep.set_selectable(false);
        row_sep.add(&sep_box);
        nav_list_clone.add(&row_sep);

        for proj in &s.projects {
            add_item(&proj.name, "folder", Filter::Project(proj.id.unwrap()));
        }
        
        nav_list_clone.show_all();
    };

    (container.upcast(), refresh)
}

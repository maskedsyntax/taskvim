use gtk::prelude::*;
use gtk::{Box, Button, Entry, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, Image, Popover, GestureMultiPress};
use std::rc::Rc;
use std::cell::RefCell;
use crate::state::{AppState, Filter};

pub fn build(state: Rc<RefCell<AppState>>, refresh_all: Rc<dyn Fn()>) -> (gtk::Widget, impl Fn()) {
    let container = Box::new(Orientation::Vertical, 0);
    container.style_context().add_class("sidebar");

    // Navigation List
    let nav_list = ListBox::new();
    nav_list.style_context().add_class("navigation-sidebar");
    nav_list.set_selection_mode(gtk::SelectionMode::None);

    let scrolled = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_vexpand(true);
    scrolled.add(&nav_list);

    container.pack_start(&scrolled, true, true, 0);

    // New Project Entry
    let new_project_box = Box::new(Orientation::Horizontal, 4);
    new_project_box.style_context().add_class("sidebar-item");
    new_project_box.set_margin_top(8);
    new_project_box.set_margin_bottom(8);
    new_project_box.set_margin_start(8);
    new_project_box.set_margin_end(8);
    
    let new_project_entry = Entry::new();
    new_project_entry.set_placeholder_text(Some("New Project..."));
    new_project_entry.set_hexpand(true);
    
    let add_btn = Button::from_icon_name(Some("list-add-symbolic"), gtk::IconSize::Button);
    add_btn.style_context().add_class("flat");

    new_project_box.pack_start(&new_project_entry, true, true, 0);
    new_project_box.pack_start(&add_btn, false, false, 0);
    container.pack_start(&new_project_box, false, false, 0);

    // Signal handlers
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
            row.style_context().add_class("sidebar-item");
            
            let b = Box::new(Orientation::Horizontal, 12);
            b.set_margin_top(8);
            b.set_margin_bottom(8);
            b.set_margin_start(8);
            b.set_margin_end(8);
            
            let icon = Image::from_icon_name(Some(icon_name), gtk::IconSize::Button);
            let lbl = Label::new(Some(label));
            lbl.set_hexpand(true);
            lbl.set_xalign(0.0);

            b.pack_start(&icon, false, false, 0);
            b.pack_start(&lbl, true, true, 0);
            
            if active == filter {
                row.style_context().add_class("selected");
            }

            // Click handler
            let filter_clone = filter.clone();
            let state_gesture = state_clone.clone();
            let refresh_gesture = refresh_all_clone.clone();
            
            // In GTK3 ListBox, we usually use `row-activated` signal on the ListBox or EventBox.
            // But GestureMultiPress works too.
            let gesture = GestureMultiPress::new(&row);
            gesture.connect_released(move |_, _, _, _| {
                state_gesture.borrow_mut().active_filter = filter_clone.clone();
                refresh_gesture();
            });
            
            row.add(&b);
            nav_list_clone.add(&row);
            
            // If project, add extra buttons
            if let Filter::Project(pid) = filter {
                let edit_btn = Button::from_icon_name(Some("document-edit-symbolic"), gtk::IconSize::Button);
                edit_btn.style_context().add_class("flat");
                edit_btn.style_context().add_class("circular");
                
                let s_clone = state_clone.clone();
                let r_clone = refresh_all_clone.clone();
                let proj_name = label.to_string();
                
                let popover = Popover::new(Some(&edit_btn));
                let pop_box = Box::new(Orientation::Horizontal, 4);
                pop_box.set_margin_top(8);
                pop_box.set_margin_bottom(8);
                pop_box.set_margin_start(8);
                pop_box.set_margin_end(8);
                let pop_entry = Entry::new();
                pop_entry.set_text(&proj_name);
                let pop_save = Button::with_label("Save");
                pop_save.style_context().add_class("suggested-action");
                
                pop_box.pack_start(&pop_entry, true, true, 0);
                pop_box.pack_start(&pop_save, false, false, 0);
                pop_box.show_all();
                popover.add(&pop_box);
                
                // GTK3 MenuButton is one way, but we can just toggle popover on click manually for Button.
                // Or use MenuButton.
                let edit_menu_btn = gtk::MenuButton::new();
                edit_menu_btn.set_image(Some(&Image::from_icon_name(Some("document-edit-symbolic"), gtk::IconSize::Button)));
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

                let del_btn = Button::from_icon_name(Some("user-trash-symbolic"), gtk::IconSize::Button);
                del_btn.style_context().add_class("flat");
                del_btn.style_context().add_class("circular");
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

        add_item("Inbox", "mail-inbox-symbolic", Filter::Inbox);
        add_item("Today", "weather-clear-symbolic", Filter::Today);
        add_item("Upcoming", "x-office-calendar-symbolic", Filter::Upcoming);

        let sep = Label::new(Some("Projects"));
        sep.style_context().add_class("dim-label");
        sep.set_margin_top(12);
        sep.set_margin_bottom(4);
        sep.set_xalign(0.0);
        sep.set_margin_start(12);
        
        let row_sep = ListBoxRow::new();
        row_sep.set_activatable(false);
        row_sep.set_selectable(false);
        row_sep.add(&sep);
        nav_list_clone.add(&row_sep);

        for proj in &s.projects {
            add_item(&proj.name, "folder-symbolic", Filter::Project(proj.id.unwrap()));
        }
        
        nav_list_clone.show_all();
    };

    (container.upcast(), refresh)
}

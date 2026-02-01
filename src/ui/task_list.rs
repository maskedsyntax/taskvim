use gtk::prelude::*;
use gtk::{Box, Button, Entry, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, Image, CheckButton, MenuButton, Calendar, Popover, HeaderBar, IconSize, SpinButton, Adjustment};
use gtk::glib;
use std::rc::Rc;
use std::cell::RefCell;
use crate::state::{AppState};
use crate::domain::TaskStatus;
use chrono::{Local, Utc, TimeZone, Timelike};

pub fn build(state: Rc<RefCell<AppState>>, refresh_all: Rc<dyn Fn()>) -> (gtk::Widget, impl Fn()) {
    let container = Box::new(Orientation::Vertical, 0);

    // Header Bar (simulated as top bar in content area)
    let header = HeaderBar::new();
    header.set_show_close_button(false); // Inside pane, usually don't show window controls if not root, but GTK3 CSD might render them.
    // If we want it to look like a titlebar, we might need to style it.
    
    // Search Entry
    let search_entry = Entry::new();
    search_entry.set_placeholder_text(Some("Search tasks..."));
    search_entry.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, Some("system-search-symbolic"));
    search_entry.set_width_chars(30);
    
    let state_clone = state.clone();
    let refresh_clone = refresh_all.clone();
    search_entry.connect_changed(move |e| {
        state_clone.borrow_mut().search_query = e.text().to_string();
        refresh_clone();
    });
    header.set_custom_title(Some(&search_entry));

    // Theme Toggle
    let theme_btn = Button::from_icon_name(Some("weather-clear-symbolic"), IconSize::Button);
    let state_clone = state.clone();
    theme_btn.connect_clicked(move |btn| {
        let mut s = state_clone.borrow_mut();
        s.is_dark_mode = !s.is_dark_mode;
        
        let settings = gtk::Settings::default().unwrap();
        settings.set_gtk_application_prefer_dark_theme(s.is_dark_mode);
        let theme_name = if s.is_dark_mode { "Adwaita-dark" } else { "Adwaita" };
        settings.set_gtk_theme_name(Some(theme_name));
        let theme_name = if s.is_dark_mode { "Adwaita-dark" } else { "Adwaita" };
        settings.set_gtk_theme_name(Some(theme_name));
        
        let icon_name = if s.is_dark_mode { "weather-clear-night-symbolic" } else { "weather-clear-symbolic" };
        btn.set_image(Some(&Image::from_icon_name(Some(icon_name), IconSize::Button)));
    });
    header.pack_end(&theme_btn);

    container.pack_start(&header, false, false, 0);

    // Main Content Box
    let content_box = Box::new(Orientation::Vertical, 0);
    content_box.set_margin_top(12);
    content_box.set_margin_bottom(12);
    content_box.set_margin_start(12);
    content_box.set_margin_end(12);
    
    // List Header (Active Filter Name)
    let list_header = Label::new(Some("Inbox"));
    list_header.set_xalign(0.0);
    list_header.style_context().add_class("title-1"); // Large title
    list_header.set_margin_bottom(16);
    content_box.pack_start(&list_header, false, false, 0);

    // Add Task Input Area
    let input_box = Box::new(Orientation::Horizontal, 8);
    input_box.style_context().add_class("card");
    input_box.set_margin_bottom(12);

    let task_entry = Entry::new();
    task_entry.set_placeholder_text(Some("Add a new task..."));
    task_entry.set_hexpand(true);
    
    // Date Picker
    let date_btn = MenuButton::new();
    date_btn.set_image(Some(&Image::from_icon_name(Some("x-office-calendar-symbolic"), IconSize::Button)));
    
    let date_popover = Popover::new(Some(&date_btn));
    let date_box = Box::new(Orientation::Vertical, 8);
    date_box.set_margin_top(8); date_box.set_margin_bottom(8); date_box.set_margin_start(8); date_box.set_margin_end(8);
    
    let calendar = Calendar::new();
    date_box.pack_start(&calendar, false, false, 0);
    
    let time_box = Box::new(Orientation::Horizontal, 4);
    time_box.set_halign(gtk::Align::Center);
    let time_label = Label::new(Some("Time:"));
    
    let hour_adj = Adjustment::new(12.0, 0.0, 23.0, 1.0, 10.0, 0.0);
    let hour_spin = SpinButton::new(Some(&hour_adj), 1.0, 0);
    hour_spin.set_wrap(true);
    hour_spin.set_width_chars(2);
    
    let min_adj = Adjustment::new(0.0, 0.0, 59.0, 1.0, 10.0, 0.0);
    let min_spin = SpinButton::new(Some(&min_adj), 1.0, 0);
    min_spin.set_wrap(true);
    min_spin.set_width_chars(2);
    
    time_box.pack_start(&time_label, false, false, 4);
    time_box.pack_start(&hour_spin, false, false, 0);
    time_box.pack_start(&Label::new(Some(":")), false, false, 2);
    time_box.pack_start(&min_spin, false, false, 0);
    
    date_box.pack_start(&time_box, false, false, 0);
    date_box.show_all();
    date_popover.add(&date_box);
    date_btn.set_popover(Some(&date_popover));
    
    let selected_date = Rc::new(RefCell::new(None::<chrono::DateTime<Utc>>));
    
    // Set default spins to current time
    let now = Local::now();
    hour_spin.set_value(now.hour() as f64);
    min_spin.set_value(now.minute() as f64);
    
    let selected_date_clone = selected_date.clone();
    let date_btn_clone = date_btn.clone();

    // Helper to update selected_date
    let update_date = Rc::new(move |cal: &Calendar, h_spin: &SpinButton, m_spin: &SpinButton| {
        let (year, month, day) = cal.date(); 
        let h = h_spin.value() as u32;
        let m = m_spin.value() as u32;
        
        if let Some(naive_date) = chrono::NaiveDate::from_ymd_opt(year as i32, month + 1, day) {
            if let Some(naive_dt) = naive_date.and_hms_opt(h, m, 0) {
                 // Convert to UTC
                 match Local.from_local_datetime(&naive_dt) {
                    chrono::LocalResult::Single(local_dt) => {
                        *selected_date_clone.borrow_mut() = Some(local_dt.with_timezone(&Utc));
                        date_btn_clone.style_context().add_class("suggested-action");
                    },
                    _ => {} // Ambiguous or invalid time (DST transition), ignore or default
                 }
            }
        }
    });
    
    let update_clone1 = update_date.clone();
    let h_clone = hour_spin.clone();
    let m_clone = min_spin.clone();
    calendar.connect_day_selected(move |cal| {
        update_clone1(cal, &h_clone, &m_clone);
    });
    
    let update_clone2 = update_date.clone();
    let cal_clone2 = calendar.clone();
    let m_clone2 = min_spin.clone();
    hour_spin.connect_value_changed(move |h_spin| {
        update_clone2(&cal_clone2, h_spin, &m_clone2);
    });

    let update_clone3 = update_date.clone();
    let cal_clone3 = calendar.clone();
    let h_clone3 = hour_spin.clone();
    min_spin.connect_value_changed(move |m_spin| {
        update_clone3(&cal_clone3, &h_clone3, m_spin);
    });

    let add_btn = Button::with_label("Add");
    add_btn.style_context().add_class("suggested-action");

    input_box.pack_start(&task_entry, true, true, 0);
    input_box.pack_start(&date_btn, false, false, 0);
    input_box.pack_start(&add_btn, false, false, 0);
    
    content_box.pack_start(&input_box, false, false, 0);

    // Task List
    let task_list_box = ListBox::new();
    task_list_box.set_selection_mode(gtk::SelectionMode::None);
    task_list_box.style_context().add_class("boxed-list");

    let scrolled = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.add(&task_list_box);
    scrolled.set_vexpand(true);
    
    content_box.pack_start(&scrolled, true, true, 0);
    container.pack_start(&content_box, true, true, 0);

    // Add Logic
    let state_clone = state.clone();
    let refresh_clone = refresh_all.clone();
    let task_entry_clone = task_entry.clone();
    let selected_date_clone = selected_date.clone();
    let date_btn_clone = date_btn.clone();

    let add_task_action = Rc::new(move || {
        let title = task_entry_clone.text();
        if !title.is_empty() {
            let date = *selected_date_clone.borrow();
            let _ = state_clone.borrow_mut().add_task(title.to_string(), date);
            
            // Reset
            task_entry_clone.set_text("");
            *selected_date_clone.borrow_mut() = None;
            date_btn_clone.style_context().remove_class("suggested-action");
            
            refresh_clone();
        }
    });

    let add_action_clone = add_task_action.clone();
    task_entry.connect_activate(move |_| add_action_clone());
    add_btn.connect_clicked(move |_| add_task_action());

    // Refresh Logic
    let state_clone = state.clone();
    let refresh_all_clone = refresh_all.clone();
    let task_list_box_clone = task_list_box.clone();
    let list_header_clone = list_header.clone();

    let refresh = move || {
        for child in task_list_box_clone.children() {
            task_list_box_clone.remove(&child);
        }

        let s = state_clone.borrow();
        
        // Update Header
        let (title_text, color) = match &s.active_filter {
            crate::state::Filter::Inbox => ("Inbox".to_string(), None),
            crate::state::Filter::Today => ("Today".to_string(), None),
            crate::state::Filter::Upcoming => ("Upcoming".to_string(), None),
            crate::state::Filter::Project(id) => {
                if let Some(p) = s.projects.iter().find(|p| p.id == Some(*id)) {
                    (p.name.clone(), Some(p.color.clone()))
                } else {
                    ("Unknown Project".to_string(), None)
                }
            }
        };
        
        if let Some(c) = color {
             list_header_clone.set_markup(&format!("<span foreground=\"{}\">{}</span>", c, glib::markup_escape_text(&title_text)));
        } else {
             list_header_clone.set_text(&title_text);
        }

        let tasks = s.filtered_tasks();

        for task in tasks {
            let row = ListBoxRow::new();
            let b = Box::new(Orientation::Horizontal, 12);
            b.set_margin_top(12);
            b.set_margin_bottom(12);
            b.set_margin_start(12);
            b.set_margin_end(12);

            // Checkbox
            let check = CheckButton::new();
            check.set_active(task.status == TaskStatus::Done);
            check.set_valign(gtk::Align::Center); // Align to center
            let tid = task.id.unwrap();
            let s_clone = state_clone.clone();
            let r_clone = refresh_all_clone.clone();
            let status = task.status.clone();
            
            check.connect_toggled(move |_| {
                s_clone.borrow_mut().toggle_task(tid, status.clone());
                r_clone();
            });
            b.pack_start(&check, false, false, 0);

            // Title and Date VBox
            let v_box = Box::new(Orientation::Vertical, 2);
            v_box.set_hexpand(true);
            v_box.set_valign(gtk::Align::Center);

            let title = Label::new(Some(&task.title));
            title.set_xalign(0.0);
            if task.status == TaskStatus::Done {
                title.style_context().add_class("dim-label");
                title.set_markup(&format!("<s>{}</s>", glib::markup_escape_text(&task.title)));
            }
            v_box.pack_start(&title, false, false, 0);

            // Date
            if let Some(date) = task.due_date {
                let local = date.with_timezone(&Local);
                let date_str = local.format("%b %d %Y at %l:%M %p").to_string();
                let date_lbl = Label::new(Some(&date_str));
                date_lbl.style_context().add_class("dim-label");
                // Use a smaller font or box if needed. GTK "dim-label" is usually enough.
                // To make it look like a badge, we could wrap it in a box with bg, but text is fine as per inspo.
                date_lbl.set_xalign(0.0);
                v_box.pack_start(&date_lbl, false, false, 0);
            }
            
            b.pack_start(&v_box, true, true, 0);

            // Edit
            let edit_menu_btn = MenuButton::new();
            edit_menu_btn.set_image(Some(&Image::from_icon_name(Some("document-edit-symbolic"), IconSize::Button)));
            edit_menu_btn.style_context().add_class("flat");
            edit_menu_btn.style_context().add_class("circular");
            edit_menu_btn.set_valign(gtk::Align::Center);
            
            let popover = Popover::new(Some(&edit_menu_btn));
            let pop_box = Box::new(Orientation::Horizontal, 4);
            pop_box.set_margin_top(8);
            pop_box.set_margin_bottom(8);
            pop_box.set_margin_start(8);
            pop_box.set_margin_end(8);
            let pop_entry = Entry::new();
            pop_entry.set_text(&task.title);
            let pop_save = Button::with_label("Save");
            pop_save.style_context().add_class("suggested-action");
            
            pop_box.pack_start(&pop_entry, true, true, 0);
            pop_box.pack_start(&pop_save, false, false, 0);
            pop_box.show_all();
            popover.add(&pop_box);
            edit_menu_btn.set_popover(Some(&popover));

            let s_clone_edit = state_clone.clone();
            let r_clone_edit = refresh_all_clone.clone();
            let popover_clone = popover.clone();
            
            pop_save.connect_clicked(move |_| {
                let new_title = pop_entry.text();
                if !new_title.is_empty() {
                    s_clone_edit.borrow_mut().update_task_title(tid, new_title.to_string());
                    r_clone_edit();
                    popover_clone.popdown();
                }
            });
            b.pack_start(&edit_menu_btn, false, false, 0);

            // Delete
            let del_btn = Button::from_icon_name(Some("user-trash-symbolic"), IconSize::Button);
            del_btn.style_context().add_class("flat");
            del_btn.style_context().add_class("circular");
            del_btn.set_valign(gtk::Align::Center);
            let s_clone_del = state_clone.clone();
            let r_clone_del = refresh_all_clone.clone();
            
            del_btn.connect_clicked(move |_| {
                s_clone_del.borrow_mut().delete_task(tid);
                r_clone_del();
            });
            b.pack_start(&del_btn, false, false, 0);

            row.add(&b);
            task_list_box_clone.add(&row);
        }
        task_list_box_clone.show_all();
    };

    (container.upcast(), refresh)
}
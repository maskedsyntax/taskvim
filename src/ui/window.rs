use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Paned, Orientation};
use std::rc::Rc;
use std::cell::RefCell;
use crate::state::AppState;
use crate::ui::{sidebar, task_list};
use crate::config::Config;

pub fn build(
    app: &Application, 
    state: Rc<RefCell<AppState>>,
    config: Rc<RefCell<Config>>,
    on_update: Rc<dyn Fn(Config)>
) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("TaskIt")
        .default_width(1024)
        .default_height(768)
        .build();

    let paned = Paned::new(Orientation::Horizontal);
    paned.set_position(250);

    // Refresh coordination
    let sidebar_refresher = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
    let task_list_refresher = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));

    let sidebar_refresher_clone = sidebar_refresher.clone();
    let task_list_refresher_clone = task_list_refresher.clone();

    let refresh_all = Rc::new(move || {
        if let Some(f) = sidebar_refresher_clone.borrow().as_ref() { f(); }
        if let Some(f) = task_list_refresher_clone.borrow().as_ref() { f(); }
    });

    // Pass config/updater to sidebar
    let (sidebar_widget, s_refresh) = sidebar::build(state.clone(), refresh_all.clone(), config, on_update, window.clone().into());
    let (content_widget, t_refresh) = task_list::build(state.clone(), refresh_all.clone());

    *sidebar_refresher.borrow_mut() = Some(Box::new(s_refresh));
    *task_list_refresher.borrow_mut() = Some(Box::new(t_refresh));

    paned.pack1(&sidebar_widget, false, false);
    paned.pack2(&content_widget, true, false);
    
    window.add(&paned);
    
    refresh_all();

    window
}
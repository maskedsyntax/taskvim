use std::sync::{Arc, Mutex};
use crate::db::Db;
use crate::domain::{Task, Project, TaskStatus};
use chrono::{Local, Utc};

#[derive(Clone, Debug, PartialEq)]
pub enum Filter {
    Inbox,
    Today,
    Upcoming,
    Project(i64),
}

pub struct AppState {
    db: Arc<Mutex<Db>>,
    pub tasks: Vec<Task>,
    pub projects: Vec<Project>,
    pub active_filter: Filter,
    pub search_query: String,
    // We can keep track of dark mode here or let GTK handle it.
    // Let's allow toggling override.
    pub is_dark_mode: bool, 
}

impl AppState {
    pub fn new(db: Db) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            tasks: Vec::new(),
            projects: Vec::new(),
            active_filter: Filter::Inbox,
            search_query: String::new(),
            is_dark_mode: false,
        }
    }

    pub fn refresh(&mut self) {
        let db = self.db.lock().unwrap();
        if let Ok(tasks) = db.get_all_tasks() {
            self.tasks = tasks;
        }
        if let Ok(projects) = db.get_projects() {
            self.projects = projects;
        }
    }

    pub fn add_task(&mut self, title: String, due_date: Option<chrono::DateTime<Utc>>) -> anyhow::Result<()> {
        let project_id = match self.active_filter {
            Filter::Project(id) => Some(id),
            _ => None,
        };
        
        let mut task = Task::new(title, project_id);
        
        if let Some(d) = due_date {
            task.due_date = Some(d);
        } else if self.active_filter == Filter::Today {
             task.due_date = Some(Utc::now());
        }

        let db = self.db.lock().unwrap();
        db.create_task(&task)?;
        drop(db);
        self.refresh();
        Ok(())
    }

    pub fn toggle_task(&mut self, task_id: i64, current_status: TaskStatus) {
        let new_status = match current_status {
            TaskStatus::Todo => TaskStatus::Done,
            TaskStatus::Done => TaskStatus::Todo,
        };
        
        let db = self.db.lock().unwrap();
        let _ = db.update_task_status(task_id, new_status);
        drop(db);
        self.refresh();
    }
    
    pub fn delete_task(&mut self, task_id: i64) {
        let db = self.db.lock().unwrap();
        let _ = db.delete_task(task_id);
        drop(db);
        self.refresh();
    }

    pub fn update_task_title(&mut self, task_id: i64, new_title: String) {
        let db = self.db.lock().unwrap();
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == Some(task_id)) {
            task.title = new_title.clone();
            let _ = db.update_task(task);
        }
        drop(db);
        self.refresh();
    }
    
    pub fn update_task_due_date(&mut self, task_id: i64, due_date: Option<chrono::DateTime<Utc>>) {
        let db = self.db.lock().unwrap();
        let _ = db.update_task_due_date(task_id, due_date);
        drop(db);
        self.refresh();
    }

    pub fn add_project(&mut self, name: String) -> anyhow::Result<()> {
        let project = Project::new(name);
        let db = self.db.lock().unwrap();
        db.create_project(&project)?;
        drop(db);
        self.refresh();
        Ok(())
    }

    pub fn update_project_name(&mut self, project_id: i64, name: String) -> anyhow::Result<()> {
        let db = self.db.lock().unwrap();
        if let Some(project) = self.projects.iter_mut().find(|p| p.id == Some(project_id)) {
            project.name = name.clone();
            db.update_project(project)?;
        }
        drop(db);
        self.refresh();
        Ok(())
    }

    pub fn delete_project(&mut self, project_id: i64) -> anyhow::Result<()> {
        let db = self.db.lock().unwrap();
        db.delete_project(project_id)?;
        drop(db);
        
        if self.active_filter == Filter::Project(project_id) {
            self.active_filter = Filter::Inbox;
        }
        
        self.refresh();
        Ok(())
    }

    pub fn filtered_tasks(&self) -> Vec<Task> {
        let query = self.search_query.to_lowercase();
        
        self.tasks.iter().filter(|t| {
            if !query.is_empty() && !t.title.to_lowercase().contains(&query) {
                return false;
            }

            match self.active_filter {
                Filter::Inbox => t.project_id.is_none(),
                Filter::Today => {
                     if let Some(due) = t.due_date {
                         let local_due = due.with_timezone(&Local);
                         let today = Local::now();
                         local_due.date_naive() == today.date_naive()
                     } else {
                         false
                     }
                },
                Filter::Upcoming => {
                     if let Some(due) = t.due_date {
                         let local_due = due.with_timezone(&Local);
                         let today = Local::now();
                         local_due.date_naive() > today.date_naive()
                     } else {
                         false
                     }
                },
                Filter::Project(pid) => t.project_id == Some(pid),
            }
        }).cloned().collect()
    }
}
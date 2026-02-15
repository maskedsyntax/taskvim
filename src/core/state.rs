use crate::domain::{Task, TaskStatus};
use crate::storage::SqliteStorage;
use crate::error::Result;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
    Filter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Position,
    Priority,
    CreatedAt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertAction {
    AddEnd,
    AddBelow,
    AddAbove,
}

pub struct AppState {
    pub tasks: Vec<Task>,
    pub selected_index: usize,
    pub mode: Mode,
    pub insert_action: InsertAction,
    pub command_buffer: String,
    pub storage: SqliteStorage,
    pub running: bool,
    pub sort_by: SortBy,
    pub filter_string: Option<String>,
    pub pending_g: bool,
    pub selection_anchor: Option<usize>,
}

impl AppState {
    pub fn new(storage: SqliteStorage) -> Result<Self> {
        let tasks = storage.get_tasks(None)?;
        Ok(Self {
            tasks,
            selected_index: 0,
            mode: Mode::Normal,
            insert_action: InsertAction::AddEnd,
            command_buffer: String::new(),
            storage,
            running: true,
            sort_by: SortBy::Position,
            filter_string: None,
            pending_g: false,
            selection_anchor: None,
        })
    }

    pub fn reload_tasks(&mut self) -> Result<()> {
        self.tasks = self.storage.get_tasks(self.filter_string.as_deref())?;
        match self.sort_by {
            SortBy::Position => self.tasks.sort_by_key(|t| t.position),
            SortBy::Priority => self.tasks.sort_by_key(|t| -t.priority),
            SortBy::CreatedAt => self.tasks.sort_by_key(|t| t.created_at),
        }
        if self.selected_index >= self.tasks.len() && !self.tasks.is_empty() {
            self.selected_index = self.tasks.len() - 1;
        }
        Ok(())
    }

    pub fn add_task(&mut self, title: String) -> Result<()> {
        let mut task = Task::new(title);
        task.position = self.tasks.iter().map(|t| t.position).max().unwrap_or(0) + 1;
        self.storage.save_task(&task)?;
        self.reload_tasks()?;
        Ok(())
    }

    pub fn add_task_below(&mut self, title: String) -> Result<()> {
        let current_pos = self.tasks.get(self.selected_index).map(|t| t.position).unwrap_or(0);
        
        // Shift all tasks after current_pos
        for task in self.tasks.iter_mut() {
            if task.position > current_pos {
                task.position += 1;
                self.storage.save_task(task)?;
            }
        }

        let mut new_task = Task::new(title);
        new_task.position = current_pos + 1;
        self.storage.save_task(&new_task)?;
        self.reload_tasks()?;
        self.selected_index += 1;
        Ok(())
    }

    pub fn add_task_above(&mut self, title: String) -> Result<()> {
        let current_pos = self.tasks.get(self.selected_index).map(|t| t.position).unwrap_or(0);
        
        // Shift all tasks starting from current_pos
        for task in self.tasks.iter_mut() {
            if task.position >= current_pos {
                task.position += 1;
                self.storage.save_task(task)?;
            }
        }

        let mut new_task = Task::new(title);
        new_task.position = current_pos;
        self.storage.save_task(&new_task)?;
        self.reload_tasks()?;
        Ok(())
    }

    pub fn delete_selected_task(&mut self) -> Result<()> {
        if self.mode == Mode::Visual {
            self.delete_visual_selection()?;
            self.mode = Mode::Normal;
            self.selection_anchor = None;
        } else {
            if let Some(task) = self.tasks.get(self.selected_index) {
                self.storage.delete_task(task.id)?;
                self.reload_tasks()?;
            }
        }
        Ok(())
    }

    pub fn delete_visual_selection(&mut self) -> Result<()> {
        if let Some(anchor) = self.selection_anchor {
            let start = anchor.min(self.selected_index);
            let end = anchor.max(self.selected_index);
            
            // Collect IDs first to avoid index shifting issues during deletion if we were modifying in-place
            // But we are deleting from DB, so it's fine.
            let ids_to_delete: Vec<Uuid> = self.tasks[start..=end].iter().map(|t| t.id).collect();
            
            for id in ids_to_delete {
                self.storage.delete_task(id)?;
            }
            self.reload_tasks()?;
            
            // Adjust selection
            if self.selected_index >= self.tasks.len() && !self.tasks.is_empty() {
                self.selected_index = self.tasks.len() - 1;
            } else if self.tasks.is_empty() {
                self.selected_index = 0;
            }
        }
        Ok(())
    }

    pub fn increase_priority(&mut self) -> Result<()> {
        if let Some(mut task) = self.tasks.get(self.selected_index).cloned() {
            if task.priority < 5 {
                task.priority += 1;
                self.storage.save_task(&task)?;
                self.reload_tasks()?;
            }
        }
        Ok(())
    }

    pub fn decrease_priority(&mut self) -> Result<()> {
        if let Some(mut task) = self.tasks.get(self.selected_index).cloned() {
            if task.priority > 1 {
                task.priority -= 1;
                self.storage.save_task(&task)?;
                self.reload_tasks()?;
            }
        }
        Ok(())
    }

    pub fn cycle_status(&mut self) -> Result<()> {
        if let Some(mut task) = self.tasks.get(self.selected_index).cloned() {
            task.status = match task.status {
                TaskStatus::Todo => TaskStatus::Doing,
                TaskStatus::Doing => TaskStatus::Done,
                TaskStatus::Done => TaskStatus::Archived,
                TaskStatus::Archived => TaskStatus::Todo,
            };
            task.updated_at = Utc::now();
            self.storage.save_task(&task)?;
            self.reload_tasks()?;
        }
        Ok(())
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if !self.tasks.is_empty() && self.selected_index < self.tasks.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn move_to_top(&mut self) {
        self.selected_index = 0;
    }

    pub fn move_to_bottom(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_index = self.tasks.len() - 1;
        }
    }

    pub fn page_down(&mut self) {
        self.selected_index = (self.selected_index + 10).min(self.tasks.len().saturating_sub(1));
    }

    pub fn page_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(10);
    }

    pub fn execute_command(&mut self, cmd: &str) -> Result<()> {
        match cmd {
            "q" => self.running = false,
            "wq" => {
                self.running = false;
            }
            "w" => {
                // Already persisted on every change for now, but could be batched later
            }
            "sort priority" => {
                self.sort_by = SortBy::Priority;
                self.reload_tasks()?;
            }
            "sort created" => {
                self.sort_by = SortBy::CreatedAt;
                self.reload_tasks()?;
            }
            "sort position" => {
                self.sort_by = SortBy::Position;
                self.reload_tasks()?;
            }
            _ => {
                if cmd.starts_with("filter ") {
                    let filter_part = &cmd[7..];
                    if filter_part.trim().is_empty() {
                         self.filter_string = None;
                    } else {
                         self.filter_string = Some(filter_part.to_string());
                    }
                    self.reload_tasks()?;
                } else if cmd == "filter" {
                     self.filter_string = None;
                     self.reload_tasks()?;
                }
            }
        }
        Ok(())
    }
}

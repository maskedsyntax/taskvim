use crate::domain::Task;
use crate::storage::SqliteStorage;
use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
    Filter,
}

pub struct AppState {
    pub tasks: Vec<Task>,
    pub selected_index: usize,
    pub mode: Mode,
    pub command_buffer: String,
    pub storage: SqliteStorage,
    pub running: bool,
}

impl AppState {
    pub fn new(storage: SqliteStorage) -> Result<Self> {
        let tasks = storage.get_tasks()?;
        Ok(Self {
            tasks,
            selected_index: 0,
            mode: Mode::Normal,
            command_buffer: String::new(),
            storage,
            running: true,
        })
    }

    pub fn reload_tasks(&mut self) -> Result<()> {
        self.tasks = self.storage.get_tasks()?;
        if self.selected_index >= self.tasks.len() && !self.tasks.is_empty() {
            self.selected_index = self.tasks.len() - 1;
        }
        Ok(())
    }

    pub fn add_task(&mut self, title: String) -> Result<()> {
        let task = Task::new(title);
        self.storage.save_task(&task)?;
        self.reload_tasks()?;
        Ok(())
    }

    pub fn delete_selected_task(&mut self) -> Result<()> {
        if let Some(task) = self.tasks.get(self.selected_index) {
            self.storage.delete_task(task.id)?;
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
}

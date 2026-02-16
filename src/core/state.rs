use crate::domain::{Task, TaskStatus};
use crate::storage::SqliteStorage;
use crate::error::Result;
use crate::config::lua::{Config, LuaConfig};
use crate::core::actions::Action;
use chrono::Utc;
use uuid::Uuid;
use std::collections::{HashSet, HashMap};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
    Filter,
    Stats,
    Search,
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
    Edit,
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
    pub pending_z: bool,
    pub pending_y: bool,
    pub pending_at: bool,
    pub pending_q: bool,
    pub selection_anchor: Option<usize>,
    pub editing_task_id: Option<Uuid>,
    pub config: Config,
    pub lua_config: Arc<LuaConfig>,
    pub collapsed_projects: HashSet<String>,
    pub yanked_task: Option<Task>,
    pub macro_recording: Option<char>,
    pub macros: HashMap<char, Vec<crossterm::event::KeyEvent>>,
    pub search_query: Option<String>,
}

impl AppState {
    pub fn new(storage: SqliteStorage, lua_config: Arc<LuaConfig>) -> Result<Self> {
        let tasks = storage.get_tasks(None)?;
        let config = lua_config.get_config();
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
            pending_z: false,
            pending_y: false,
            pending_at: false,
            pending_q: false,
            selection_anchor: None,
            editing_task_id: None,
            config,
            lua_config,
            collapsed_projects: HashSet::new(),
            yanked_task: None,
            macro_recording: None,
            macros: HashMap::new(),
            search_query: None,
        })
    }

    pub fn reload_tasks(&mut self) -> Result<()> {
        let mut all_tasks = self.storage.get_tasks(self.filter_string.as_deref())?;
        
        // Apply sorting first
        match self.sort_by {
            SortBy::Position => all_tasks.sort_by_key(|t| t.position),
            SortBy::Priority => all_tasks.sort_by_key(|t| -t.priority),
            SortBy::CreatedAt => all_tasks.sort_by_key(|t| t.created_at),
        }

        // Filter out tasks in collapsed projects and apply search
        self.tasks = all_tasks.into_iter().filter(|t| {
            let project_visible = if let Some(p) = &t.project {
                !self.collapsed_projects.contains(p)
            } else {
                true
            };

            let search_match = if let Some(q) = &self.search_query {
                t.title.to_lowercase().contains(&q.to_lowercase()) || 
                t.description.as_ref().map(|d| d.to_lowercase().contains(&q.to_lowercase())).unwrap_or(false)
            } else {
                true
            };

            project_visible && search_match
        }).collect();

        if self.selected_index >= self.tasks.len() && !self.tasks.is_empty() {
            self.selected_index = self.tasks.len() - 1;
        }
        Ok(())
    }

    pub fn add_task(&mut self, title: String) -> Result<()> {
        let mut task = Task::new(title);
        task.priority = self.config.default_priority;
        task.position = self.tasks.iter().map(|t| t.position).max().unwrap_or(0) + 1;
        self.storage.save_task(&task)?;
        let _ = self.lua_config.trigger_hook("on_task_create", Some(&task));
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
        new_task.priority = self.config.default_priority;
        new_task.position = current_pos + 1;
        self.storage.save_task(&new_task)?;
        let _ = self.lua_config.trigger_hook("on_task_create", Some(&new_task));
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
        new_task.priority = self.config.default_priority;
        new_task.position = current_pos;
        self.storage.save_task(&new_task)?;
        let _ = self.lua_config.trigger_hook("on_task_create", Some(&new_task));
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

    pub fn start_editing(&mut self) {
        if let Some(task) = self.tasks.get(self.selected_index) {
            self.editing_task_id = Some(task.id);
            self.command_buffer = task.title.clone();
            self.mode = Mode::Insert;
            self.insert_action = InsertAction::Edit;
        }
    }

    pub fn commit_edit(&mut self) -> Result<()> {
        if let Some(id) = self.editing_task_id {
            if let Some(mut task) = self.tasks.iter().find(|t| t.id == id).cloned() {
                self.storage.push_history(&task)?;
                self.storage.clear_redo()?;
                task.title = self.command_buffer.clone();
                task.updated_at = Utc::now();
                self.storage.save_task(&task)?;
                let _ = self.lua_config.trigger_hook("on_task_update", Some(&task));
                self.reload_tasks()?;
            }
            self.editing_task_id = None;
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

    pub fn undo(&mut self) -> Result<()> {
        if let Some((hist_id, task)) = self.storage.get_latest_history()? {
            // Push current state to redo before reverting
            if let Some(current) = self.tasks.iter().find(|t| t.id == task.id) {
                self.storage.push_redo(current)?;
            }
            self.storage.save_task(&task)?;
            self.storage.delete_history_entry(hist_id)?;
            self.reload_tasks()?;
        }
        Ok(())
    }

    pub fn play_macro(&mut self, reg: char) -> Result<()> {
        if let Some(events) = self.macros.get(&reg).cloned() {
            for event in events {
                if let Some(action) = self.config.keymap.get_action(self.mode, event) {
                    self.handle_action(action)?;
                }
            }
        }
        Ok(())
    }

    pub fn redo(&mut self) -> Result<()> {
        if let Some((redo_id, task)) = self.storage.get_latest_redo()? {
            // Push current state to undo before applying redo
            if let Some(current) = self.tasks.iter().find(|t| t.id == task.id) {
                self.storage.push_history(current)?;
            }
            self.storage.save_task(&task)?;
            self.storage.delete_redo_entry(redo_id)?;
            self.reload_tasks()?;
        }
        Ok(())
    }

    pub fn yank_selected(&mut self) {
        if let Some(task) = self.tasks.get(self.selected_index) {
            self.yanked_task = Some(task.clone());
        }
    }

    pub fn paste_below(&mut self) -> Result<()> {
        if let Some(task) = self.yanked_task.clone() {
            let mut new_task = task;
            new_task.id = Uuid::new_v4();
            new_task.created_at = Utc::now();
            new_task.updated_at = Utc::now();
            
            let current_pos = self.tasks.get(self.selected_index).map(|t| t.position).unwrap_or(0);
            
            // Shift
            for t in self.tasks.iter_mut() {
                if t.position > current_pos {
                    t.position += 1;
                    self.storage.save_task(t)?;
                }
            }

            new_task.position = current_pos + 1;
            self.storage.save_task(&new_task)?;
            self.storage.clear_redo()?;
            self.reload_tasks()?;
            self.selected_index += 1;
        }
        Ok(())
    }

    pub fn increase_priority(&mut self) -> Result<()> {
        if let Some(mut task) = self.tasks.get(self.selected_index).cloned() {
            if task.priority < 5 {
                self.storage.push_history(&task)?;
                self.storage.clear_redo()?;
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
                self.storage.push_history(&task)?;
                self.storage.clear_redo()?;
                task.priority -= 1;
                self.storage.save_task(&task)?;
                self.reload_tasks()?;
            }
        }
        Ok(())
    }

    pub fn cycle_status(&mut self) -> Result<()> {
        if let Some(mut task) = self.tasks.get(self.selected_index).cloned() {
            self.storage.push_history(&task)?;
            self.storage.clear_redo()?;
            task.status = match task.status {
                TaskStatus::Todo => TaskStatus::Doing,
                TaskStatus::Doing => TaskStatus::Done,
                TaskStatus::Done => TaskStatus::Archived,
                TaskStatus::Archived => TaskStatus::Todo,
            };
            task.updated_at = Utc::now();
            self.storage.save_task(&task)?;
            let _ = self.lua_config.trigger_hook("on_status_change", Some(&task));
            self.reload_tasks()?;
        }
        Ok(())
    }

    pub fn toggle_collapse(&mut self) -> Result<()> {
        if let Some(task) = self.tasks.get(self.selected_index) {
            if let Some(project) = &task.project {
                if self.collapsed_projects.contains(project) {
                    self.collapsed_projects.remove(project);
                } else {
                    self.collapsed_projects.insert(project.clone());
                }
                self.reload_tasks()?;
            }
        }
        Ok(())
    }

    pub fn get_all_projects(&self) -> Vec<String> {
        let mut projects: HashSet<String> = HashSet::new();
        // We need all projects from DB to navigate correctly
        if let Ok(tasks) = self.storage.get_tasks(None) {
            for t in tasks {
                if let Some(p) = t.project {
                    projects.insert(p);
                }
            }
        }
        let mut sorted: Vec<String> = projects.into_iter().collect();
        sorted.sort();
        sorted
    }

    pub fn next_project(&mut self) -> Result<()> {
        let projects = self.get_all_projects();
        if projects.is_empty() { return Ok(()); }

        let current_project = self.tasks.get(self.selected_index).and_then(|t| t.project.clone());
        let next_idx = if let Some(p) = current_project {
            if let Ok(idx) = projects.binary_search(&p) {
                (idx + 1) % projects.len()
            } else {
                0
            }
        } else {
            0
        };

        self.filter_string = Some(format!("project={}", projects[next_idx]));
        self.reload_tasks()?;
        self.selected_index = 0;
        Ok(())
    }

    pub fn prev_project(&mut self) -> Result<()> {
        let projects = self.get_all_projects();
        if projects.is_empty() { return Ok(()); }

        let current_project = self.tasks.get(self.selected_index).and_then(|t| t.project.clone());
        let prev_idx = if let Some(p) = current_project {
            if let Ok(idx) = projects.binary_search(&p) {
                (idx + projects.len() - 1) % projects.len()
            } else {
                projects.len() - 1
            }
        } else {
            projects.len() - 1
        };

        self.filter_string = Some(format!("project={}", projects[prev_idx]));
        self.reload_tasks()?;
        self.selected_index = 0;
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

    pub fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => self.running = false,
            Action::MoveDown => self.move_selection_down(),
            Action::MoveUp => self.move_selection_up(),
            Action::MoveToTop => self.move_to_top(),
            Action::MoveToBottom => self.move_to_bottom(),
            Action::PageDown => self.page_down(),
            Action::PageUp => self.page_up(),
            Action::Delete => self.delete_selected_task()?,
            Action::CycleStatus => self.cycle_status()?,
            Action::IncreasePriority => self.increase_priority()?,
            Action::DecreasePriority => self.decrease_priority()?,
            Action::EnterInsert => {
                if self.tasks.is_empty() {
                    self.mode = Mode::Insert;
                    self.insert_action = InsertAction::AddEnd;
                } else {
                    self.start_editing();
                }
            }
            Action::EnterInsertBelow => {
                self.mode = Mode::Insert;
                self.insert_action = InsertAction::AddBelow;
            }
            Action::EnterInsertAbove => {
                self.mode = Mode::Insert;
                self.insert_action = InsertAction::AddAbove;
            }
            Action::EnterVisual => {
                self.mode = Mode::Visual;
                self.selection_anchor = Some(self.selected_index);
            }
            Action::EnterCommand => {
                self.mode = Mode::Command;
                self.command_buffer.clear();
            }
            Action::Cancel => {
                self.mode = Mode::Normal;
                self.selection_anchor = None;
                self.editing_task_id = None;
            }
            Action::Undo => self.undo()?,
            Action::Redo => self.redo()?,
            Action::ToggleCollapse => self.toggle_collapse()?,
            Action::NextProject => self.next_project()?,
            Action::PrevProject => self.prev_project()?,
            Action::Yank => self.yank_selected(),
            Action::Paste => self.paste_below()?,
            Action::EnterSearch => {
                self.mode = Mode::Search;
                self.command_buffer.clear();
            }
        }
        Ok(())
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
            "stats" => {
                self.mode = Mode::Stats;
            }
            _ => {
                if cmd.starts_with("lua ") {
                    let code = &cmd[4..];
                    let _ = self.lua_config.run_code(code);
                } else if cmd.starts_with("filter ") {
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

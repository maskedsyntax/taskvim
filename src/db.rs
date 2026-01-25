use anyhow::{Context, Result};
use directories::ProjectDirs;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use crate::domain::{Task, Project, TaskStatus};
use chrono::{DateTime, Utc};

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn init() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        
        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;
        
        let db = Self { conn };
        db.create_tables()?;
        
        Ok(db)
    }

    fn get_db_path() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "maskedsyntax", "taskit") {
            let data_dir = proj_dirs.data_dir();
            Ok(data_dir.join("taskit.db"))
        } else {
            // Fallback
            let home = std::env::var("HOME").context("Could not find HOME directory")?;
            Ok(PathBuf::from(home).join(".local/share/taskit/taskit.db"))
        }
    }

    fn create_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL,
                due_date TEXT,
                project_id INTEGER REFERENCES projects(id),
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        
        Ok(())
    }

    pub fn create_task(&self, task: &Task) -> Result<i64> {
        let created_at = task.created_at.to_rfc3339();
        let due_date = task.due_date.map(|d| d.to_rfc3339());
        let status = task.status.to_string();

        self.conn.execute(
            "INSERT INTO tasks (title, description, status, due_date, project_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![task.title, task.description, status, due_date, task.project_id, created_at],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_task_status(&self, id: i64, status: TaskStatus) -> Result<()> {
        let s = status.to_string();
        self.conn.execute(
            "UPDATE tasks SET status = ?1 WHERE id = ?2",
            params![s, id],
        )?;
        Ok(())
    }
    
    pub fn update_task(&self, task: &Task) -> Result<()> {
        let id = task.id.context("Task ID is missing")?;
        let due_date = task.due_date.map(|d| d.to_rfc3339());
        let status = task.status.to_string();
        
        self.conn.execute(
            "UPDATE tasks SET title = ?1, description = ?2, status = ?3, due_date = ?4, project_id = ?5 WHERE id = ?6",
            params![task.title, task.description, status, due_date, task.project_id, id],
        )?;
        Ok(())
    }

    pub fn delete_task(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM tasks WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn update_task_due_date(&self, id: i64, due_date: Option<DateTime<Utc>>) -> Result<()> {
         let d = due_date.map(|dt| dt.to_rfc3339());
         self.conn.execute(
            "UPDATE tasks SET due_date = ?1 WHERE id = ?2",
            params![d, id],
        )?;
        Ok(())
    }

    pub fn get_all_tasks(&self) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare("SELECT id, title, description, status, due_date, project_id, created_at FROM tasks ORDER BY created_at DESC")?;
        
        let task_iter = stmt.query_map([], |row| {
            let due_date_str: Option<String> = row.get(4)?;
            let due_date = due_date_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));
            
            let created_at_str: String = row.get(6)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str).unwrap().with_timezone(&Utc); // Handle error better in prod

            Ok(Task {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                description: row.get(2)?,
                status: TaskStatus::from(row.get::<_, String>(3)?),
                due_date,
                project_id: row.get(5)?,
                created_at,
            })
        })?;

        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task?);
        }
        
        Ok(tasks)
    }

    pub fn create_project(&self, project: &Project) -> Result<i64> {
        let created_at = project.created_at.to_rfc3339();
        self.conn.execute(
            "INSERT INTO projects (name, created_at) VALUES (?1, ?2)",
            params![project.name, created_at],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_project(&self, project: &Project) -> Result<()> {
        let id = project.id.context("Project ID is missing")?;
        self.conn.execute(
            "UPDATE projects SET name = ?1 WHERE id = ?2",
            params![project.name, id],
        )?;
        Ok(())
    }

    pub fn delete_project(&self, id: i64) -> Result<()> {
        // Move tasks to Inbox (NULL project_id)
        self.conn.execute(
            "UPDATE tasks SET project_id = NULL WHERE project_id = ?1",
            params![id],
        )?;
        // Delete project
        self.conn.execute(
            "DELETE FROM projects WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn get_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare("SELECT id, name, created_at FROM projects")?;
        let proj_iter = stmt.query_map([], |row| {
             let created_at_str: String = row.get(2)?;
             let created_at = DateTime::parse_from_rfc3339(&created_at_str).unwrap().with_timezone(&Utc);

             Ok(Project {
                 id: Some(row.get(0)?),
                 name: row.get(1)?,
                 created_at,
             })
        })?;
        
        let mut projects = Vec::new();
        for p in proj_iter {
            projects.push(p?);
        }
        Ok(projects)
    }
}

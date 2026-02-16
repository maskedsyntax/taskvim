use crate::domain::{Task, TaskStatus};
use crate::domain::query::Filter;
use crate::error::Result;
use rusqlite::{params, Connection};
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct SqliteStorage {
    conn: Connection,
}

impl SqliteStorage {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL,
                priority INTEGER NOT NULL,
                due_date TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                project TEXT,
                recurrence_rule TEXT,
                position INTEGER NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS task_tags (
                task_id TEXT NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (task_id, tag_id),
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS dependencies (
                task_id TEXT NOT NULL,
                depends_on TEXT NOT NULL,
                PRIMARY KEY (task_id, depends_on),
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
                FOREIGN KEY (depends_on) REFERENCES tasks(id) ON DELETE CASCADE
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id TEXT NOT NULL,
                snapshot TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;

        // Indexes
        self.conn.execute("CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)", [])?;
        self.conn.execute("CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority)", [])?;
        self.conn.execute("CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date)", [])?;
        self.conn.execute("CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project)", [])?;

        Ok(())
    }

    pub fn save_task(&self, task: &Task) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tasks (
                id, title, description, status, priority, due_date, created_at, updated_at, project, recurrence_rule, position
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                task.id.to_string(),
                task.title,
                task.description,
                task.status.to_string(),
                task.priority,
                task.due_date.map(|d: DateTime<Utc>| d.to_rfc3339()),
                task.created_at.to_rfc3339(),
                task.updated_at.to_rfc3339(),
                task.project,
                task.recurrence_rule,
                task.position,
            ],
        )?;

        // Update tags
        self.conn.execute("DELETE FROM task_tags WHERE task_id = ?", [task.id.to_string()])?;
        for tag in &task.tags {
            self.conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?)", [tag])?;
            let tag_id: i64 = self.conn.query_row(
                "SELECT id FROM tags WHERE name = ?",
                [tag],
                |row| row.get(0),
            )?;
            self.conn.execute(
                "INSERT INTO task_tags (task_id, tag_id) VALUES (?, ?)",
                params![task.id.to_string(), tag_id],
            )?;
        }

        // Update dependencies
        self.conn.execute("DELETE FROM dependencies WHERE task_id = ?", [task.id.to_string()])?;
        for dep_id in &task.dependencies {
            self.conn.execute(
                "INSERT INTO dependencies (task_id, depends_on) VALUES (?, ?)",
                params![task.id.to_string(), dep_id.to_string() as String],
            )?;
        }

        Ok(())
    }

    pub fn get_tasks(&self, filter_string: Option<&str>) -> Result<Vec<Task>> {
        let mut sql = "SELECT * FROM tasks".to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        
        if let Some(s) = filter_string {
            if !s.trim().is_empty() {
                 let filters = Filter::parse(s)?;
                 if !filters.is_empty() {
                     sql.push_str(" WHERE ");
                     let mut conditions = Vec::new();
                     for filter in filters {
                         let (cond, val): (String, String) = filter.to_sql_condition()?;
                         conditions.push(cond);
                         params.push(Box::new(val));
                     }
                     sql.push_str(&conditions.join(" AND "));
                 }
            }
        }
        
        sql.push_str(" ORDER BY position ASC, created_at DESC");

        let mut stmt = self.conn.prepare(&sql)?;
        
        let task_iter = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str).map_err(|_| rusqlite::Error::InvalidQuery)?;
            
            let status_str: String = row.get(3)?;
            let created_at_str: String = row.get(6)?;
            let updated_at_str: String = row.get(7)?;
            let due_date_str: Option<String> = row.get(5)?;

            Ok(Task {
                id,
                title: row.get(1)?,
                description: row.get(2)?,
                status: TaskStatus::from(status_str),
                priority: row.get(4)?,
                due_date: due_date_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap().with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str).unwrap().with_timezone(&Utc),
                project: row.get(8)?,
                recurrence_rule: row.get(9)?,
                position: row.get(10)?,
                tags: Vec::new(),
                dependencies: Vec::new(),
            })
        })?;

        let mut tasks = Vec::new();
        for task_res in task_iter {
            let mut task = task_res?;
            
            // Get tags
            let mut tag_stmt = self.conn.prepare(
                "SELECT t.name FROM tags t JOIN task_tags tt ON t.id = tt.tag_id WHERE tt.task_id = ?"
            )?;
            let tag_iter = tag_stmt.query_map([task.id.to_string()], |row| row.get(0))?;
            for tag in tag_iter {
                task.tags.push(tag?);
            }

            // Get dependencies
            let mut dep_stmt = self.conn.prepare(
                "SELECT depends_on FROM dependencies WHERE task_id = ?"
            )?;
            let dep_iter = dep_stmt.query_map([task.id.to_string()], |row| {
                let s: String = row.get(0)?;
                Ok(Uuid::parse_str(&s).unwrap())
            })?;
            for dep in dep_iter {
                task.dependencies.push(dep?);
            }

            tasks.push(task);
        }

        Ok(tasks)
    }

    pub fn delete_task(&self, id: Uuid) -> Result<()> {
        self.conn.execute("DELETE FROM tasks WHERE id = ?", [id.to_string()])?;
        Ok(())
    }

    pub fn push_history(&self, task: &Task) -> Result<()> {
        let snapshot = serde_json::to_string(task)?;
        self.conn.execute(
            "INSERT INTO history (task_id, snapshot, timestamp) VALUES (?, ?, ?)",
            params![task.id.to_string(), snapshot, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn get_latest_history(&self) -> Result<Option<(i64, Task)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, snapshot FROM history ORDER BY id DESC LIMIT 1"
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let snapshot: String = row.get(1)?;
            let task: Task = serde_json::from_str(&snapshot)?;
            Ok(Some((id, task)))
        } else {
            Ok(None)
        }
    }

    pub fn delete_history_entry(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM history WHERE id = ?", [id])?;
        Ok(())
    }
}

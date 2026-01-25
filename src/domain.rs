use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    Done,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Todo
    }
}

// Helper to map string to enum for DB
impl From<String> for TaskStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Done" => TaskStatus::Done,
            _ => TaskStatus::Todo,
        }
    }
}

impl ToString for TaskStatus {
    fn to_string(&self) -> String {
        match self {
            TaskStatus::Todo => "Todo".to_string(),
            TaskStatus::Done => "Done".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub due_date: Option<DateTime<Utc>>,
    pub project_id: Option<i64>,
    pub created_at: DateTime<Utc>,
}

impl Task {
    pub fn new(title: String, project_id: Option<i64>) -> Self {
        Self {
            id: None,
            title,
            description: None,
            status: TaskStatus::Todo,
            due_date: None,
            project_id,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            id: None,
            name,
            created_at: Utc::now(),
        }
    }
}

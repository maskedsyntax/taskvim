use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    Doing,
    Done,
    Archived,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Todo
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TaskStatus::Todo => "Todo",
            TaskStatus::Doing => "Doing",
            TaskStatus::Done => "Done",
            TaskStatus::Archived => "Archived",
        };
        write!(f, "{}", s)
    }
}

impl From<String> for TaskStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Doing" => TaskStatus::Doing,
            "Done" => TaskStatus::Done,
            "Archived" => TaskStatus::Archived,
            _ => TaskStatus::Todo,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: i32,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub project: Option<String>,
    pub recurrence_rule: Option<String>,
    pub dependencies: Vec<Uuid>,
    pub position: i32,
}

impl Task {
    pub fn new(title: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description: None,
            status: TaskStatus::Todo,
            priority: 3,
            due_date: None,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            project: None,
            recurrence_rule: None,
            dependencies: Vec::new(),
            position: 0,
        }
    }
}

use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    MoveDown,
    MoveUp,
    MoveToTop,
    MoveToBottom,
    PageDown,
    PageUp,
    Delete,
    CycleStatus,
    IncreasePriority,
    DecreasePriority,
    EnterInsert,
    EnterInsertBelow,
    EnterInsertAbove,
    EnterVisual,
    EnterCommand,
    Cancel,
    Undo,
    Redo,
    ToggleCollapse,
    NextProject,
    PrevProject,
    Yank,
    Paste,
    EnterSearch,
}

impl FromStr for Action {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "quit" => Ok(Action::Quit),
            "move_down" => Ok(Action::MoveDown),
            "move_up" => Ok(Action::MoveUp),
            "move_top" => Ok(Action::MoveToTop),
            "move_bottom" => Ok(Action::MoveToBottom),
            "page_down" => Ok(Action::PageDown),
            "page_up" => Ok(Action::PageUp),
            "delete" | "delete_task" => Ok(Action::Delete),
            "cycle_status" => Ok(Action::CycleStatus),
            "increase_priority" => Ok(Action::IncreasePriority),
            "decrease_priority" => Ok(Action::DecreasePriority),
            "insert" => Ok(Action::EnterInsert),
            "insert_below" | "add_below" => Ok(Action::EnterInsertBelow),
            "insert_above" | "add_above" => Ok(Action::EnterInsertAbove),
            "visual" => Ok(Action::EnterVisual),
            "command" => Ok(Action::EnterCommand),
            "cancel" => Ok(Action::Cancel),
            "undo" => Ok(Action::Undo),
            "redo" => Ok(Action::Redo),
            "toggle_collapse" => Ok(Action::ToggleCollapse),
            "next_project" => Ok(Action::NextProject),
            "prev_project" => Ok(Action::PrevProject),
            "yank" => Ok(Action::Yank),
            "paste" => Ok(Action::Paste),
            "search" => Ok(Action::EnterSearch),
            _ => Err(()),
        }
    }
}

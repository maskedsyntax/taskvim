#[cfg(test)]
mod tests {
    use crate::core::AppState;
    use crate::storage::SqliteStorage;
    use tempfile::NamedTempFile;

    #[test]
    fn test_task_creation_and_positioning() {
        let tmp_file = NamedTempFile::new().unwrap();
        let path = tmp_file.path().to_str().unwrap();
        let storage = SqliteStorage::new(path).unwrap();
        let mut state = AppState::new(storage).unwrap();

        state.add_task("Task 1".to_string()).unwrap();
        state.add_task("Task 2".to_string()).unwrap();
        
        assert_eq!(state.tasks.len(), 2);
        assert_eq!(state.tasks[0].title, "Task 1");
        assert_eq!(state.tasks[1].title, "Task 2");
        assert_eq!(state.tasks[1].position, state.tasks[0].position + 1);

        state.selected_index = 0;
        state.add_task_below("Task 1.5".to_string()).unwrap();
        
        assert_eq!(state.tasks.len(), 3);
        assert_eq!(state.tasks[1].title, "Task 1.5");
        assert_eq!(state.tasks[2].title, "Task 2");
    }

    #[test]
    fn test_task_editing() {
        let tmp_file = NamedTempFile::new().unwrap();
        let path = tmp_file.path().to_str().unwrap();
        let storage = SqliteStorage::new(path).unwrap();
        let mut state = AppState::new(storage).unwrap();

        state.add_task("Original Title".to_string()).unwrap();
        let id = state.tasks[0].id;

        state.selected_index = 0;
        state.start_editing();
        
        assert_eq!(state.mode, crate::core::Mode::Insert);
        assert_eq!(state.command_buffer, "Original Title");
        assert_eq!(state.editing_task_id, Some(id));

        state.command_buffer = "Edited Title".to_string();
        state.commit_edit().unwrap();

        assert_eq!(state.tasks[0].title, "Edited Title");
        assert_eq!(state.editing_task_id, None);
    }
}

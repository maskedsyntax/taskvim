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
}

use serde::{Serialize, Deserialize};
use tokio::fs::{read_to_string, write};

use crate::task_manager::task::TaskManager;

type DBResult<T> = Result<T, DBError>;

#[derive(Debug)]
pub enum DBError {
    ConnectionError(String),
}

// Structure to hold the entire state of the application.
// This is what we'll be serializing and deserializing to and from the file.
#[derive(Serialize, Deserialize, Debug)]
pub struct AppState {
    pub task_manager: TaskManager,
}

pub struct DatabaseManager {
    file_path: String,
}

impl DatabaseManager {
    pub fn new(file_path: String) -> Self {
        Self { file_path }
    }

    pub async fn load_state(&self) -> DBResult<AppState> {
        let contents = read_to_string(&self.file_path).await.unwrap();
        let state: AppState = serde_yaml::from_str(contents.as_str()).unwrap();
        Ok(state)
    }

    pub async fn save_state(&self, state: &AppState) -> DBResult<()> {
        let contents = serde_yaml::to_string(state).unwrap();
        write(&self.file_path, contents.as_bytes()).await.unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_manager::task::Status;
    use tokio::runtime::Runtime;
    use tempfile::tempdir;
    
    fn create_task_manager() -> TaskManager {
        let mut manager = TaskManager::new();
        manager.add_task("Task 1".to_string());
        manager.add_task("Task 2".to_string());
        manager
    }

    #[test]
    fn test_save_and_load_state() {
        let rt = Runtime::new().unwrap();

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yaml").to_str().unwrap().to_string();

        let db = DatabaseManager::new(file_path.clone());

        let task_manager = create_task_manager();
        let mut app_state = AppState { task_manager };

        rt.block_on(async {
            db.save_state(&app_state).await.unwrap();
        });
        
        let mut loaded_state = rt.block_on(async {
            db.load_state().await.unwrap()
        });

        assert_eq!(app_state.task_manager.get_tasks_by_status(Status::Todo), loaded_state.task_manager.get_tasks_by_status(crate::task_manager::task::Status::Todo));
    }

}


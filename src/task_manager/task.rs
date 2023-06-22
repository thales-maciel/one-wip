use serde::{Serialize, Deserialize};

use crate::database::database::AppState;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Task {
    id: u64,
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskManager {
    next_id: u64,
    todo_tasks: Vec<Task>,
    wip_tasks: Vec<Task>,
    done_tasks: Vec<Task>,
}

impl From<AppState> for TaskManager {
    fn from(v: AppState) -> Self {
        TaskManager {
            next_id: v.task_manager.next_id,
            todo_tasks: v.task_manager.todo_tasks,
            wip_tasks: v.task_manager.wip_tasks,
            done_tasks: v.task_manager.done_tasks,
        }
    }
}

pub enum Status {
    Todo,
    Wip,
    Done
}

#[derive(Debug)]
pub enum TaskManagerError {
    TaskNotFound(u64),
}

type TMResult<T> = Result<T, TaskManagerError>;

impl TaskManager {
    pub fn new() -> TaskManager {
        TaskManager {
            next_id: 1,
            todo_tasks: Vec::new(),
            wip_tasks: Vec::new(),
            done_tasks: Vec::new(),
        }
    }

    pub fn add_task(&mut self, name: String) -> Task {
        let task = Task { id: self.next_id, name, };
        self.todo_tasks.push(task.clone());
        self.next_id += 1;
        task
    }

    pub fn update_task(&mut self, id: u64, name: String) -> TMResult<()> {
        let lists = [&mut self.todo_tasks, &mut self.wip_tasks, &mut self.done_tasks];

        for list in lists {
            if let Some(task) = list.iter_mut().find(|t| t.id == id) {
                task.name = name;
                return Ok(());
            }
        }

        Err(TaskManagerError::TaskNotFound(id))
    }

    pub fn move_task(&mut self, id: u64, new_status: Status) -> TMResult<()> {
        let lists = [&mut self.todo_tasks, &mut self.wip_tasks, &mut self.done_tasks];
        let mut opt_task: Option<Task> = None;
        for list in lists {
            if let Some(pos) = list.iter_mut().position(|t| t.id == id) {
                opt_task = Some(list.remove(pos));
            }
        }
        let Some(task) = opt_task else {
            return Err(TaskManagerError::TaskNotFound(id));
        };
        match new_status {
            Status::Todo => self.todo_tasks.push(task),
            Status::Wip => self.wip_tasks.push(task),
            Status::Done => self.done_tasks.push(task),
        }

        Ok(())
    }

    pub fn reorder_task(&mut self, id: u64, new_position: usize) -> TMResult<()> {
        let lists = [&mut self.todo_tasks, &mut self.wip_tasks, &mut self.done_tasks];
        for list in lists {
            if let Some(pos) = list.iter_mut().position(|t| t.id == id) {
                let task = list.remove(pos);
                list.insert(new_position, task);
                return Ok(())
            };
        }
        Err(TaskManagerError::TaskNotFound(id))
    }

    pub fn remove_task(&mut self, id: u64) -> TMResult<()> {
        let lists = [&mut self.todo_tasks, &mut self.wip_tasks, &mut self.done_tasks];
        for list in lists {
            if let Some(pos) = list.iter_mut().position(|t| t.id == id) {
                list.remove(pos);
                return Ok(());
            }
        }
        Err(TaskManagerError::TaskNotFound(id))
    }

    pub fn get_tasks_by_status(&mut self, status: Status) -> Vec<Task> {
        match status {
            Status::Todo => self.todo_tasks.clone(),
            Status::Wip => self.wip_tasks.clone(),
            Status::Done => self.done_tasks.clone(),
        }
    }

    pub fn get_task(&mut self, id: u64) -> Option<Task> {
        let lists = [&mut self.todo_tasks, &mut self.wip_tasks, &mut self.done_tasks];
        for list in lists {
            if let Some(task) = list.iter_mut().find(|t| t.id == id) {
                return Some(task.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_task() {
        let mut manager = TaskManager::new();

        manager.add_task("Task 1".to_owned());
        assert_eq!(manager.todo_tasks.len(), 1);
        assert_eq!(manager.todo_tasks.first().unwrap().name, "Task 1".to_owned());
    }

    #[test]
    fn test_update_task() {
        let mut manager = TaskManager::new();

        manager.add_task("Task 1".to_owned());
        let _ = manager.update_task(1, "Updated Task 1".to_owned());
        assert_eq!(manager.todo_tasks.len(), 1);
        assert_eq!(manager.todo_tasks.first().unwrap().name, "Updated Task 1".to_owned());
    }

    #[test]
    fn test_move_task() {
        let mut manager = TaskManager::new();

        manager.add_task("Task 1".to_owned());
        assert_eq!(manager.todo_tasks.len(), 1);
        let _ = manager.move_task(1, Status::Wip).unwrap();
        assert_eq!(manager.todo_tasks.len(), 0);
        assert_eq!(manager.wip_tasks.len(), 1);
        assert_eq!(manager.wip_tasks.first().unwrap().name, "Task 1".to_owned());
    }

    #[test]
    fn test_delete_task() {
        let mut manager = TaskManager::new();

        manager.add_task("Task 1".to_owned());
        assert_eq!(manager.todo_tasks.len(), 1);
        let _ = manager.remove_task(1).unwrap();
        assert_eq!(manager.todo_tasks.len(), 0);
    }

    #[test]
    fn test_order_task() {
        let mut manager = TaskManager::new();

        manager.add_task("Task 1".to_owned());
        manager.add_task("Task 2".to_owned());
        assert_eq!(manager.todo_tasks.first().unwrap().name, "Task 1".to_owned());
        assert_eq!(manager.todo_tasks.get(1).unwrap().name, "Task 2".to_owned());
        let _ = manager.reorder_task(2, 0).unwrap();
        assert_eq!(manager.todo_tasks.first().unwrap().name, "Task 2".to_owned());
        assert_eq!(manager.todo_tasks.get(1).unwrap().name, "Task 1".to_owned());
        let _ = manager.reorder_task(1, 0).unwrap();
        assert_eq!(manager.todo_tasks.first().unwrap().name, "Task 1".to_owned());
        assert_eq!(manager.todo_tasks.get(1).unwrap().name, "Task 2".to_owned());
    }
}

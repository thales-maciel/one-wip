use std::sync::Arc;

use crossterm::event::KeyCode;
use ratatui::{backend::Backend, Frame, layout::{Layout, Direction, Constraint}};
use tokio::sync::Mutex;

use crate::{task_manager::task::TaskManager, database::database::{DatabaseManager, AppState}};

use super::{task_input::TaskInput, board::Board};

pub enum AppMode {
    Viewing,
    Adding,
    Editing,
}

pub struct App {
    state: AppMode,
    board: Board,
    input: TaskInput,
    task_manager: Arc<Mutex<TaskManager>>,
    db_manager: Arc<Mutex<DatabaseManager>>,
}

impl App {
    pub async fn new() -> App {
        let db_manager = Arc::new(Mutex::new(DatabaseManager::new(".one_wip.yaml".to_string())));
        let tasks: AppState = db_manager.lock().await.load_state().await.unwrap();
        let task_manager = Arc::new(Mutex::new(TaskManager::from(tasks)));
        App {
            state: AppMode::Viewing,
            board: Board::new(),
            input: TaskInput::new(),
            task_manager,
            db_manager,
        }
    }

    pub fn handle_input(&mut self, keycode: KeyCode) {
        match self.state {
            AppMode::Viewing => match keycode {_ => {}},
            AppMode::Adding => match keycode {_ => {}},
            AppMode::Editing => match keycode {_ => {}}
        }
    }

    pub fn render<B: Backend> (&mut self, frame: &mut Frame<B>) {
        match self.state {
            AppMode::Viewing => self.board.render(frame, frame.size()),
            AppMode::Adding | AppMode::Editing => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(80),
                            Constraint::Percentage(20)
                        ]
                            .as_ref()
                    )
                        .split(frame.size());

                self.board.render(frame, chunks[0]);
                self.input.render(frame, chunks[1]);
            }
        }
    }

    pub async fn add_task(&mut self, task: String) {
        self.task_manager.lock().await.add_task(task.clone());
        self.board.add_task(task);
        self.save_tasks().await;
    }

    pub async fn delete_task(&mut)

    pub async fn save_tasks(&mut self) {
        let task_manager = self.task_manager.lock().await.clone();
        self.db_manager.lock().await.save_state(&AppState { task_manager }).await.unwrap();
    }
}


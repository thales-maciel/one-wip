use ratatui::{Frame, backend::Backend, layout::{Layout, Direction, Constraint, Rect}};

use super::task_list::TaskList;

pub struct Board {
    pub todo: TaskList,
    pub wip: TaskList,
    pub done: TaskList,
    pub selected_column: SelectedColumn,
}

pub enum SelectedColumn {
    Todo,
    Wip,
    Done,
}

impl Board {
    pub fn new() -> Board {
        Board {
            todo: TaskList::new(Vec::new()),
            wip: TaskList::new(Vec::new()),
            done: TaskList::new(Vec::new()),
            selected_column: SelectedColumn::Todo,
        }
    }

    pub fn selected_list(&mut self) -> &mut TaskList {
        match self.selected_column {
            SelectedColumn::Todo => &mut self.todo,
            SelectedColumn::Wip => &mut self.wip,
            SelectedColumn::Done => &mut self.done,
        }
    }

    pub fn move_selection_left(&mut self) {
        self.selected_column = match self.selected_column {
            SelectedColumn::Todo => SelectedColumn::Done,
            SelectedColumn::Wip => SelectedColumn::Todo,
            SelectedColumn::Done => SelectedColumn::Wip,
        }
    }

    pub fn move_selection_right(&mut self) {
        self.selected_column = match self.selected_column {
            SelectedColumn::Todo => SelectedColumn::Wip,
            SelectedColumn::Wip => SelectedColumn::Done,
            SelectedColumn::Done => SelectedColumn::Todo,
        }
    }

    pub fn add_task(&mut self, task: String) {
        self.selected_list().add_task(task)
    }

    pub fn remove_task(&mut self) {
        self.selected_list().delete_task();
    }

    pub fn update_task(&mut self, new_task: String) {
        self.selected_list().update_task(new_task)
    }

    pub fn move_task(&mut self, direction: MoveDirection) {
        if let Some(_) = self.selected_list().state.selected() {
            let task = self.selected_list().delete_task().unwrap();
            match direction {
                MoveDirection::Left => {
                    self.move_selection_left();
                    self.selected_list().add_task(task);
                }
                MoveDirection::Right => {
                    self.move_selection_right();
                    self.selected_list().add_task(task);
                }
            }
        }
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect){
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                    Constraint::Percentage(33),
                ]
                    .as_ref(),
            )
                .split(area);

        self.todo.render(frame, chunks[0]);
        self.wip.render(frame, chunks[1]);
        self.done.render(frame, chunks[2]);
    }
}

pub enum MoveDirection {
    Left,
    Right,
}


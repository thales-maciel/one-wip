use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum Column {
    Todo,
    Wip,
    Done,
}

#[derive(Debug, Clone)]
pub enum Mode {
    Add,
    Help,
    Focus,
    Overview,
    Edit(usize),
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct ListColumn<T> {
    pub index: Option<usize>,
    pub items: Vec<T>,
}

impl ListColumn<String> {
    fn new() -> ListColumn<String> {
        ListColumn {
            index: None,
            items: vec![],
        }
    }

    pub fn push(&mut self, item: String) {
        self.items.push(item);
        self.index = Some(self.items.len() - 1);
    }

    pub fn remove(&mut self) -> Option<String> {
        if let Some(item) = self.current_item() {
            let index = self.index.unwrap();
            self.items.remove(index);
            if self.items.len() == 0 {
                self.index = None;
            } else {
                if index + 1 > self.items.len() {
                    self.index = Some(self.items.len() - 1);
                } else {
                    self.index = Some(index);
                }
            }
            Some(item)
        } else {
            None
        }
    }

    pub fn current_item(&mut self) -> Option<String> {
        if let Some(index) = self.index {
            self.items.get(index).cloned()
        } else {
            None
        }
    }

    pub fn move_item_up(&mut self) {
        if let Some(index) = self.index {
            if index > 0 {
                self.index = Some(index - 1);
                self.items.swap(index, index - 1);
            }
        }
    }

    pub fn move_item_down(&mut self) {
        if let Some(index) = self.index {
            if index < self.items.len() - 1 {
                self.index = Some(index + 1);
                self.items.swap(index, index + 1);
            }
        }
    }

    pub fn down(&mut self) {
        self.index = self
            .index
            .filter(|&i| i + 1 < self.items.len())
            .map(|i| i + 1)
            .or_else(|| Some(self.items.len() - 1));
    }

    pub fn up(&mut self) {
        self.index = self.index.filter(|&i| i > 0).map(|i| i - 1);
    }

    pub fn replace_at(&mut self, index: usize, value: String) {
        self.items[index] = value;
    }
}

#[derive(Debug, Clone)]
pub struct App {
    pub input: String,
    pub current_mode: Mode,
    pub current_column: Column,
    pub todo: ListColumn<String>,
    pub wip: Option<String>,
    pub done: ListColumn<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Board {
    todo: Vec<String>,
    wip: Option<String>,
    done: Vec<String>,
}

impl From<Board> for App {
    fn from(board: Board) -> App {
        App {
            input: String::new(),
            current_mode: Mode::Overview,
            current_column: Column::Todo,
            todo: ListColumn::from(board.todo),
            wip: board.wip,
            done: ListColumn::from(board.done),
        }
    }
}

impl From<App> for Board {
    fn from(board: App) -> Board {
        Board {
            todo: board.todo.items,
            wip: board.wip,
            done: board.done.items,
        }
    }
}

impl<T> From<Vec<T>> for ListColumn<T> {
    fn from(items: Vec<T>) -> ListColumn<T> {
        let idx = match items.len() > 0 {
            true => Some(0),
            false => None,
        };
        ListColumn { index: idx, items }
    }
}

impl App {
    pub fn new() -> App {
        App {
            input: String::new(),
            current_mode: Mode::Add,
            current_column: Column::Todo,
            todo: ListColumn::new(),
            wip: None,
            done: ListColumn::new(),
        }
    }

    pub fn add_task(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }
        let cloned = self.input.clone();
        self.todo.push(cloned);
        self.input = String::new();
        self.current_column = Column::Todo;
        self.current_mode = Mode::Overview;
    }

    pub fn move_to_done(&mut self) {
        if let Some(task) = self.wip.take() {
            self.done.push(task);
            self.current_column = Column::Done;
            self.current_mode = Mode::Overview;
        }
    }

    pub fn on_down(&mut self) {
        match self.current_column {
            Column::Wip => {}
            Column::Todo => self.todo.down(),
            Column::Done => self.done.down(),
        }
    }

    pub fn on_up(&mut self) {
        match self.current_column {
            Column::Wip => {}
            Column::Todo => self.todo.up(),
            Column::Done => self.done.up(),
        }
    }

    pub fn on_left(&mut self) {
        match self.current_column {
            Column::Wip => self.current_column = Column::Todo,
            Column::Done => self.current_column = Column::Wip,
            _ => {}
        }
    }

    pub fn on_right(&mut self) {
        match self.current_column {
            Column::Todo => self.current_column = Column::Wip,
            Column::Wip => self.current_column = Column::Done,
            _ => {}
        }
    }

    pub fn on_move_up(&mut self) {
        match self.current_column {
            Column::Todo => self.todo.move_item_up(),
            Column::Done => self.done.move_item_up(),
            _ => {}
        }
    }

    pub fn on_move_down(&mut self) {
        match self.current_column {
            Column::Todo => self.todo.move_item_down(),
            Column::Done => self.done.move_item_down(),
            _ => {}
        }
    }

    pub fn on_move_right(&mut self) {
        match self.current_column {
            Column::Todo => {
                if let None = self.wip {
                    let task = self.todo.remove();
                    self.wip = task;
                    self.current_column = Column::Wip;
                }
            }
            Column::Wip => {
                if let Some(task) = self.wip.take() {
                    self.done.push(task);
                    self.current_column = Column::Done;
                }
            }
            Column::Done => {}
        }
    }

    pub fn on_move_left(&mut self) {
        match self.current_column {
            Column::Done => {
                if let None = self.wip {
                    let task = self.done.remove();
                    self.wip = task;
                    self.current_column = Column::Wip;
                }
            }
            Column::Wip => {
                if let Some(task) = self.wip.take() {
                    self.todo.push(task);
                    self.current_column = Column::Todo;
                }
            }
            Column::Todo => {}
        }
    }

    pub fn on_remove_task(&mut self) {
        match self.current_column {
            Column::Wip => {
                self.wip = None;
            }
            Column::Done => {
                self.done.remove();
            }
            Column::Todo => {
                self.todo.remove();
            }
        }
    }

    pub fn on_input(&mut self, char: char) {
        self.input.push(char);
    }

    pub fn on_backspace(&mut self) {
        self.input.pop();
    }

    pub fn on_cancel_input(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }
        self.current_mode = Mode::Overview;
        self.input.clear();
    }

    pub fn enter_focus(&mut self) {
        if self.wip.is_some() {
            self.current_mode = Mode::Focus;
        }
    }

    pub fn leave_help(&mut self) {
        self.current_mode = Mode::Overview;
    }

    pub fn enter_help(&mut self) {
        self.current_mode = Mode::Help;
    }

    pub fn leave_focus(&mut self) {
        self.current_mode = Mode::Overview;
        self.input = String::new();
    }

    pub fn enter_add_mode(&mut self) {
        self.current_mode = Mode::Add;
    }

    pub fn enter_edit_mode(&mut self) {
        match self.current_column {
            Column::Todo => {
                if let Some(task) = self.todo.current_item() {
                    self.input = task.clone();
                    self.current_mode = Mode::Edit(self.todo.index.unwrap());
                }
            }
            Column::Wip => {
                if let Some(task) = &self.wip {
                    self.input = task.clone();
                    self.current_mode = Mode::Edit(0);
                }
            }
            Column::Done => {
                if let Some(task) = self.done.current_item() {
                    self.input = task.clone();
                    self.current_mode = Mode::Edit(self.done.index.unwrap());
                }
            }
        }
    }

    pub fn edit_task(&mut self, index: usize) {
        if !self.input.trim().is_empty() {
            let value = self.input.clone();
            match self.current_column {
                Column::Todo => self.todo.replace_at(index, value),
                Column::Done => self.done.replace_at(index, value),
                Column::Wip => self.wip = Some(value),
            };
            self.input = String::new();
            self.current_mode = Mode::Overview;
        }
    }
}

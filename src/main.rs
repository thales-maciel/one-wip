use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    time::{Duration, Instant},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

#[derive(Debug, PartialEq)]
enum Column {
    Todo,
    InProgress,
    Done,
}

#[derive(Debug)]
enum Mode {
    Overview,
    Focus,
    Add,
    Edit(usize),
}

#[derive(Debug)]
pub struct ListColumn<T> {
    index: Option<usize>,
    items: Vec<T>,
}

impl ListColumn<String> {
    fn new() -> ListColumn<String> {
        ListColumn {
            index: None,
            items: vec![],
        }
    }

    fn push(&mut self, item: String) {
        self.items.push(item);
        self.index = Some(self.items.len() - 1);
    }

    fn remove(&mut self) -> Option<String> {
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

    fn current_item(&mut self) -> Option<String> {
        if let Some(index) = self.index {
            self.items.get(index).cloned()
        } else {
            None
        }
    }

    fn move_item_up(&mut self) {
        if let Some(index) = self.index {
            if index > 0 {
                self.index = Some(index - 1);
                self.items.swap(index, index - 1);
            }
        }
    }

    fn move_item_down(&mut self) {
        if let Some(index) = self.index {
            if index < self.items.len() - 1 {
                self.index = Some(index + 1);
                self.items.swap(index, index + 1);
            }
        }
    }

    fn down(&mut self) {
        self.index = self.index
            .filter(|&i| i + 1 < self.items.len())
            .map(|i| i + 1)
            .or_else(|| Some(self.items.len() - 1));
    }

    fn up(&mut self) {
        self.index = self.index
            .filter(|&i| i > 0)
            .map(|i| i - 1);
    }

    fn replace_at(&mut self, index: usize, value: String) {
        self.items[index] = value;
    }

    fn iter(&mut self) -> std::slice::Iter<String> {
        self.items.iter()
    }

}

#[derive(Debug)]
pub struct KanbanBoard {
    input: String,
    current_mode: Mode,
    current_column: Column,
    todo: ListColumn<String>,
    in_progress: Option<String>,
    done: ListColumn<String>,
}

impl KanbanBoard {
    fn new() -> KanbanBoard {
        KanbanBoard {
            input: String::new(),
            current_mode: Mode::Add,
            current_column: Column::Todo,
            todo: ListColumn::new(),
            in_progress: None,
            done: ListColumn::new(),
        }
    }

    fn add_task(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }
        let cloned = self.input.clone();
        self.todo.push(cloned);
        self.input = String::new();
        self.current_column = Column::Todo;
        self.current_mode = Mode::Overview;
    }

    fn move_to_done(&mut self) {
        if let Some(task) = self.in_progress.take() {
            self.done.push(task);
            self.current_column = Column::Done;
            self.current_mode = Mode::Overview;
        }
    }

    fn on_down(&mut self) {
        match self.current_column {
            Column::InProgress => {},
            Column::Todo => self.todo.down(),
            Column::Done => self.done.down(),
        }
    }

    fn on_up(&mut self) {
        match self.current_column {
            Column::InProgress => {},
            Column::Todo => self.todo.up(),
            Column::Done => self.done.up(),
        }
    }

    fn on_left(&mut self) {
        match self.current_column {
            Column::InProgress => self.current_column = Column::Todo,
            Column::Done => self.current_column = Column::InProgress,
            _ => {}
        }
    }

    fn on_right(&mut self) {
        match self.current_column {
            Column::Todo => self.current_column = Column::InProgress,
            Column::InProgress => self.current_column = Column::Done,
            _ => {}
        }
    }

    fn on_move_up(&mut self) {
        match self.current_column {
            Column::Todo => self.todo.move_item_up(),
            Column::Done => self.done.move_item_up(),
            _ => {},
        }
    }

    fn on_move_down(&mut self) {
        match self.current_column {
            Column::Todo => self.todo.move_item_down(),
            Column::Done => self.done.move_item_down(),
            _ => {},
        }
    }

    fn on_move_right(&mut self) {
        match self.current_column {
            Column::Todo => {
                if let None = self.in_progress {
                    let task = self.todo.remove();
                    self.in_progress = task;
                    self.current_column = Column::InProgress;
                }
            }
            Column::InProgress => {
                if let Some(task) = self.in_progress.take() {
                    self.done.push(task);
                    self.current_column = Column::Done;
                }
            }
            Column::Done => {}
        }
    }

    fn on_move_left(&mut self) {
        match self.current_column {
            Column::Done => {
                if let None = self.in_progress {
                    let task = self.done.remove();
                    self.in_progress = task;
                    self.current_column = Column::InProgress;
                }
            }
            Column::InProgress => {
                if let Some(task) = self.in_progress.take() {
                    self.todo.push(task);
                    self.current_column = Column::Todo;
                }
            }
            Column::Todo => {}
        }
    }

    fn on_remove_task(&mut self) {
        match self.current_column {
            Column::InProgress => {
                self.in_progress = None;
            }
            Column::Done => {
                self.done.remove();
            }
            Column::Todo => {
                self.todo.remove();
            }
        }
    }

    fn on_input(&mut self, char: char) {
        self.input.push(char);
    }

    fn on_backspace(&mut self) {
        self.input.pop();
    }

    fn on_cancel_input(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }
        self.current_mode = Mode::Overview;
        self.input.clear();
    }

    fn enter_focus(&mut self) {
        if self.in_progress.is_some() {
            self.current_mode = Mode::Focus;
        }
    }

    fn leave_focus(&mut self) {
        self.current_mode = Mode::Overview;
        self.input = String::new();
    }

    fn enter_add_mode(&mut self) {
        self.current_mode = Mode::Add;
    }

    fn enter_edit_mode(&mut self) {
        match self.current_column {
            Column::Todo => {
                if let Some(task) = self.todo.current_item() {
                    self.input = task.clone();
                    self.current_mode = Mode::Edit(self.todo.index.unwrap());
                }
            }
            Column::InProgress => {
                if let Some(task) = &self.in_progress {
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

    fn edit_task(&mut self, index: usize) {
        if self.input.trim().is_empty() {
            return;
        }
        let value = self.input.clone();
        match self.current_column {
            Column::Todo => self.todo.replace_at(index, value),
            Column::Done => self.done.replace_at(index, value),
            Column::InProgress => self.in_progress = Some(value)
        };
        self.input = String::new();
        self.current_mode = Mode::Overview;
    }
}

fn main() -> io::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let tick_rate = Duration::from_millis(250);
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = KanbanBoard::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: KanbanBoard,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let CEvent::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match app.current_mode {
                        Mode::Overview => match key.code {
                            // quit
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Esc => return Ok(()),

                            // move cursor
                            KeyCode::Char('k') => app.on_up(),
                            KeyCode::Char('l') => app.on_right(),
                            KeyCode::Char('j') => app.on_down(),
                            KeyCode::Char('h') => app.on_left(),

                            // move task
                            KeyCode::Char('J') => app.on_move_down(),
                            KeyCode::Char('K') => app.on_move_up(),
                            KeyCode::Char('L') => app.on_move_right(),
                            KeyCode::Char('H') => app.on_move_left(),

                            // add task
                            KeyCode::Char('a') => app.enter_add_mode(),
                            KeyCode::Char('A') => app.enter_add_mode(),

                            // edit task
                            KeyCode::Char('e') => app.enter_edit_mode(),
                            KeyCode::Char('E') => app.enter_edit_mode(),

                            // remove task
                            KeyCode::Backspace => app.on_remove_task(),
                            KeyCode::Delete => app.on_remove_task(),

                            // work
                            KeyCode::Char('f') => app.enter_focus(),
                            KeyCode::Char('F') => app.enter_focus(),
                            KeyCode::Char('W') => app.enter_focus(),
                            KeyCode::Char('w') => app.enter_focus(),

                            _ => {}
                        },
                        Mode::Add => match key.code {
                            KeyCode::Enter => app.add_task(),
                            KeyCode::Char(c) => app.on_input(c),
                            KeyCode::Backspace => app.on_backspace(),
                            KeyCode::Esc => app.on_cancel_input(),
                            _ => {}
                        },
                        Mode::Focus => match key.code {
                            KeyCode::Enter => app.move_to_done(),
                            KeyCode::Esc => app.leave_focus(),
                            KeyCode::Char('q') => app.leave_focus(),
                            _ => {}
                        },
                        Mode::Edit(idx) => match key.code {
                            KeyCode::Enter => app.edit_task(idx),
                            KeyCode::Char(c) => app.on_input(c),
                            KeyCode::Backspace => app.on_backspace(),
                            KeyCode::Esc => app.leave_focus(),
                            _ => {}
                        }
                    }
                }
            }
        }
        last_tick = Instant::now();
    }
}

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut KanbanBoard) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(f.size());
    match app.current_mode {
        Mode::Overview => {
            let mut todo_lines = vec![];
            if let Some(index) = app.todo.index {
                for (i, item) in app.todo.iter().enumerate() {
                    if app.current_column == Column::Todo && i == index {
                        todo_lines.push(
                            ListItem::new(item.to_string()).style(
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        );
                    } else {
                        todo_lines.push(ListItem::new(item.to_string()));
                    }
                } 
            }

            let todo_list = List::new(todo_lines)
                .block(Block::default().borders(Borders::ALL).style(Style::default().fg(if app.current_column == Column::Todo {Color::Cyan} else {Color::White})).title("Todo"))
                .highlight_style(
                    Style::default()
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">>");

            f.render_widget(todo_list, chunks[0]);

            let mut in_progress_lines = vec![];
            if let Some(item) = {
                let ref wip_item = app.in_progress;
                wip_item
            } {
                if app.current_column == Column::InProgress {
                    in_progress_lines.push(
                        ListItem::new(item.to_string()).style(
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                    );
                } else {
                    in_progress_lines.push(ListItem::new(item.to_string()));
                }
            }

            let in_progress = List::new(in_progress_lines)
                .block(Block::default().borders(Borders::ALL).style(Style::default().fg(if app.current_column == Column::InProgress {Color::Cyan} else {Color::White})).title("Wip"))
                .highlight_style(
                    Style::default()
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">>");
            f.render_widget(in_progress, chunks[1]);

            let mut done_lines = vec![];
            if let Some(index) = app.done.index {
                for (i, item) in app.done.iter().enumerate() {
                    if app.current_column == Column::Done && i == index {
                        done_lines.push(
                            ListItem::new(item.to_string()).style(
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        );
                    } else {
                        done_lines.push(ListItem::new(item.to_string()));
                    }
                }
            }

            let done = List::new(done_lines)
                .block(Block::default().borders(Borders::ALL).style(Style::default().fg(if app.current_column == Column::Done {Color::Cyan} else {Color::White})).title("Done"))
                .highlight_style(
                    Style::default()
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">>");
            f.render_widget(done, chunks[2]);
        }
        Mode::Focus => {
            let focus_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(50),
                        Constraint::Min(3),
                        Constraint::Percentage(50),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            if let Some(item) = &app.in_progress {
                let wip = Paragraph::new(item.to_string())
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::LightCyan))
                    .block(Block::default().borders(Borders::ALL).title("Wip"));
                f.render_widget(wip, focus_layout[1])
            }
        }
        _ => {
            let input_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(50),
                        Constraint::Min(3),
                        Constraint::Percentage(50),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            let input = Paragraph::new(app.input.clone())
                .style(
                    Style::default()
                        .add_modifier(Modifier::RAPID_BLINK)
                        .fg(Color::Yellow),
                )
                .block(Block::default().borders(Borders::ALL).title("Add Task"));
            f.render_widget(input, input_layout[1]);
            f.set_cursor(input_layout[1].x + app.input.len() as u16 + 1, input_layout[1].y + 1)
        }
    }
}

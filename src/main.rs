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
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

#[derive(Debug, PartialEq, Clone, Copy)]
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
    Edit(Column, usize),
}

#[derive(Debug)]
pub struct KanbanBoard {
    input: String,
    current_mode: Mode,
    current_column: Column,
    current_index: usize,
    todo: Vec<String>,
    in_progress: Option<String>,
    done: Vec<String>,
}

impl KanbanBoard {
    pub fn foo() -> KanbanBoard {
        todo!("implement me")
    }
}

impl KanbanBoard {
    fn new() -> KanbanBoard {
        KanbanBoard {
            input: String::new(),
            current_mode: Mode::Add,
            current_column: Column::Todo,
            current_index: 0,
            todo: vec!["Sample todo 1".into()],
            in_progress: None,
            done: Vec::new(),
        }
    }

    fn add_task(&mut self) {
        let cloned = self.input.clone();
        self.todo.push(cloned);
        self.input = String::new();
        self.current_index = self.todo.len() - 1;
        self.current_mode = Mode::Overview;
    }

    fn move_to_done(&mut self) {
        if let Some(task) = self.in_progress.take() {
            self.done.push(task);
            self.current_column = Column::Done;
            self.current_index = self.done.len() - 1;
        }
    }

    fn on_down(&mut self) {
        match self.current_column {
            Column::Todo => {
                if self.todo.get(self.current_index + 1).is_some() {
                    self.current_index += 1;
                }
            }
            Column::InProgress => {}
            Column::Done => {
                if self.done.get(self.current_index + 1).is_some() {
                    self.current_index += 1;
                }
            }
        }
    }

    fn on_up(&mut self) {
        if self.current_index == 0 {
            return;
        };
        match self.current_column {
            Column::InProgress => {}
            _ => {
                self.current_index -= 1;
            }
        }
    }

    fn on_left(&mut self) {
        match self.current_column {
            Column::Todo => {}
            Column::InProgress => {
                if self.todo.len() > 0 {
                    if let None = self.todo.get(self.current_index) {
                        self.current_index = self.todo.len() - 1;
                    }
                    self.current_column = Column::Todo;
                }
            }
            Column::Done => {
                if let Some(_) = self.in_progress {
                    self.current_column = Column::InProgress;
                }
            }
        }
    }

    fn on_right(&mut self) {
        match self.current_column {
            Column::Todo => {
                if let Some(_) = self.in_progress {
                    self.current_column = Column::InProgress;
                }
            }
            Column::InProgress => {
                if self.done.len() > 0 {
                    if let None = self.done.get(self.current_index) {
                        self.current_index = self.done.len() - 1;
                    }
                    self.current_column = Column::Done;
                }
            }
            Column::Done => {}
        }
    }

    fn on_move_up(&mut self) {
        match self.current_column {
            Column::Todo => {
                if self.todo.len() > 0 && self.current_index > 0 {
                    let current = self.todo.remove(self.current_index);
                    self.current_index -= 1;
                    self.todo.insert(self.current_index, current);
                }
            }
            Column::InProgress => {}
            Column::Done => {
                if self.done.len() > 0 && self.current_index > 0 {
                    let current = self.done.remove(self.current_index);
                    self.current_index -= 1;
                    self.done.insert(self.current_index, current);
                }
            }
        }
    }

    fn on_move_down(&mut self) {
        match self.current_column {
            Column::Todo => {
                if self.todo.len() > 0 && self.current_index < self.todo.len() - 1 {
                    let current = self.todo.remove(self.current_index);
                    self.current_index += 1;
                    self.todo.insert(self.current_index, current);
                }
            }
            Column::InProgress => {}
            Column::Done => {
                if self.done.len() > 0 && self.current_index < self.done.len() - 1 {
                    let current = self.done.remove(self.current_index);
                    self.current_index += 1;
                    self.done.insert(self.current_index, current);
                }
            }
        }
    }

    fn on_move_right(&mut self) {
        match self.current_column {
            Column::Todo => {
                if let None = self.in_progress {
                    let task = self.todo.remove(self.current_index);
                    self.in_progress = Some(task);
                    self.current_column = Column::InProgress;
                }
            }
            Column::InProgress => {
                if let Some(task) = self.in_progress.take() {
                    self.done.push(task);
                    self.current_column = Column::Done;
                    self.current_index = self.done.len() - 1;
                }
            }
            Column::Done => {}
        }
    }

    fn on_move_left(&mut self) {
        match self.current_column {
            Column::Done => {
                if let None = self.in_progress {
                    let task = self.done.remove(self.current_index);
                    self.in_progress = Some(task);
                    self.current_column = Column::InProgress;
                }
            }
            Column::InProgress => {
                if let Some(task) = self.in_progress.take() {
                    self.todo.push(task);
                    self.current_column = Column::Todo;
                    self.current_index = self.todo.len() - 1;
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
                if self.current_index == self.done.len() - 1 {
                    self.current_index -= 1;
                }
                self.done.remove(self.current_index);
            }
            Column::Todo => {
                if self.current_index == self.todo.len() - 1 {
                    self.current_index -= 1;
                }
                self.todo.remove(self.current_index);
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
    }

    fn enter_add_mode(&mut self) {
        self.current_mode = Mode::Add;
    }

    fn enter_edit_mode(&mut self) {
        match self.current_column {
            Column::Todo => {
                if let Some(task) = self.todo.get(self.current_index) {
                    self.input = task.clone();
                    self.current_mode = Mode::Edit(Column::Todo, self.current_index);
                }
            }
            Column::InProgress => {
                if let Some(task) = &self.in_progress {
                    self.input = task.clone();
                    self.current_mode = Mode::Edit(Column::InProgress, self.current_index);
                }
            }
            Column::Done => {
                if let Some(task) = self.done.get(self.current_index) {
                    self.input = task.clone();
                    self.current_mode = Mode::Edit(Column::Done, self.current_index);
                }
            }
        }
    }

    fn edit_task(&mut self, col: Column, index: usize) {
        let value = self.input.clone();
        match col {
            Column::Todo => self.todo[index] = value,
            Column::Done => self.done[index] = value,
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
                        Mode::Edit(col, idx) => match key.code {
                            KeyCode::Enter => app.edit_task(col, idx),
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
            for (i, item) in app.todo.iter().enumerate() {
                if app.current_column == Column::Todo && i == app.current_index {
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

            let todo_list = List::new(todo_lines)
                .block(Block::default().borders(Borders::ALL).title("Todo"))
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
                .block(Block::default().borders(Borders::ALL).title("Wip"))
                .highlight_style(
                    Style::default()
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">>");
            f.render_widget(in_progress, chunks[1]);

            let mut done_lines = vec![];
            for (i, item) in app.done.iter().enumerate() {
                if app.current_column == Column::Done && i == app.current_index {
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

            let done = List::new(done_lines)
                .block(Block::default().borders(Borders::ALL).title("Done"))
                .highlight_style(
                    Style::default()
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">>");
            f.render_widget(done, chunks[2]);
        }
        Mode::Focus => {
            if let Some(item) = &app.in_progress {
                // let layout = Layout::default()
                //     .direction(Direction::Horizontal)
                //     .constraints(
                //         [
                //             Constraint::Percentage(10),
                //             Constraint::Percentage(80),
                //             Constraint::Percentage(10)
                //         ].as_ref()
                //     )
                //     .split(f.size());
                let wip = Paragraph::new(item.to_string())
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::LightCyan))
                    .block(Block::default().borders(Borders::ALL).title("Wip"));
                // f.render_widget(wip, layout[1]);
                // let block = Block::default().title("test").borders(Borders::ALL);
                let area = centered_rect(60, 20, f.size());
                f.render_widget(wip, area)
            }
        }
        _ => {
            let input = Paragraph::new(app.input.clone())
                .style(
                    Style::default()
                        .add_modifier(Modifier::RAPID_BLINK)
                        .fg(Color::Yellow),
                )
                .block(Block::default().borders(Borders::ALL).title("Add Task"));
            f.render_widget(input, chunks[1]);
            f.set_cursor(chunks[1].x + app.input.len() as u16 + 1, chunks[1].y + 1)
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

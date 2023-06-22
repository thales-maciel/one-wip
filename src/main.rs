use app::{App, Board, Mode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    error::Error,
    fs::read_to_string,
    io,
    path::Path,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use ui::draw;

mod app;
mod ui;

fn get_initial_board(file_path: &str) -> Result<App, Box<dyn Error>> {
    // if file_path exists and its a file
    if Path::new(file_path).exists() && Path::new(file_path).is_file() {
        let contents = read_to_string(file_path)?;
        let board: Board = serde_yaml::from_str(contents.as_str())?;
        return Ok(App::from(board));
    }
    Ok(App::new())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = mpsc::channel::<Board>(100);
    let writer_task = tokio::spawn(async move {
        while let Some(board) = rx.recv().await {
            let content = serde_yaml::to_string(&board).unwrap();
            let _ = tokio::fs::write(".one_wip.yml", content).await;
        }
    });

    let mut app = get_initial_board(".one_wip.yml")?;

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let tick_rate = Duration::from_millis(250);
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let ui_task = tokio::spawn(async move {
        let mut last_tick = Instant::now();
        loop {
            terminal.draw(|f| draw(f, &mut app)).unwrap();
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if crossterm::event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    if key.kind == event::KeyEventKind::Press {
                        match app.current_mode {
                            Mode::Overview => match key.code {
                                // quit
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => break,

                                // move cursor
                                KeyCode::Char('k') => app.on_up(),
                                KeyCode::Char('l') => app.on_right(),
                                KeyCode::Char('j') => app.on_down(),
                                KeyCode::Char('h') => app.on_left(),

                                // move task
                                KeyCode::Char('J') => {
                                    app.on_move_down();
                                    tx.send(Board::from(app.clone())).await.unwrap();
                                }
                                KeyCode::Char('K') => {
                                    app.on_move_up();
                                    tx.send(Board::from(app.clone())).await.unwrap();
                                }
                                KeyCode::Char('L') => {
                                    app.on_move_right();
                                    tx.send(Board::from(app.clone())).await.unwrap();
                                }
                                KeyCode::Char('H') => {
                                    app.on_move_left();
                                    tx.send(Board::from(app.clone())).await.unwrap();
                                }

                                // add task
                                KeyCode::Char('a') => app.enter_add_mode(),
                                KeyCode::Char('A') => app.enter_add_mode(),

                                // edit task
                                KeyCode::Char('e') => app.enter_edit_mode(),
                                KeyCode::Char('E') => app.enter_edit_mode(),

                                // remove task
                                KeyCode::Char('d') => {
                                    app.on_remove_task();
                                    tx.send(Board::from(app.clone())).await.unwrap();
                                }
                                KeyCode::Char('D') => {
                                    app.on_remove_task();
                                    tx.send(Board::from(app.clone())).await.unwrap();
                                }
                                KeyCode::Backspace => {
                                    app.on_remove_task();
                                    tx.send(Board::from(app.clone())).await.unwrap();
                                }
                                KeyCode::Delete => {
                                    app.on_remove_task();
                                    tx.send(Board::from(app.clone())).await.unwrap();
                                }

                                // work
                                KeyCode::Char('f') => app.enter_focus(),
                                KeyCode::Char('F') => app.enter_focus(),
                                KeyCode::Char('W') => app.enter_focus(),
                                KeyCode::Char('w') => app.enter_focus(),

                                // help
                                KeyCode::Char('?') => app.enter_help(),

                                _ => {}
                            },
                            Mode::Add => match key.code {
                                KeyCode::Enter => {
                                    app.add_task();
                                    tx.send(Board::from(app.clone())).await.unwrap();
                                }
                                KeyCode::Char(c) => app.on_input(c),
                                KeyCode::Backspace => app.on_backspace(),
                                KeyCode::Esc => app.on_cancel_input(),
                                _ => {}
                            },
                            Mode::Focus => match key.code {
                                KeyCode::Enter => {
                                    app.move_to_done();
                                    tx.send(Board::from(app.clone())).await.unwrap();
                                }
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
                            },
                            Mode::Help => match key.code {
                                KeyCode::Char('q') => app.leave_help(),
                                KeyCode::Enter => app.leave_help(),
                                KeyCode::Esc => app.leave_help(),
                                _ => {}
                            },
                        }
                    }
                }
            };
            last_tick = Instant::now();
            tx.send(Board::from(app.clone())).await.unwrap();
        }
        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
        terminal.show_cursor().unwrap();
    });

    tokio::try_join!(writer_task, ui_task).unwrap();
    Ok(())
}

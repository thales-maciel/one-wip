use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{App, Column, Mode};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
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
        Mode::Overview => render_overview(app, &chunks, f),
        Mode::Add => render_input(f, app),
        Mode::Edit(_) => render_input(f, app),
        Mode::Focus => render_focus(f, app),
        Mode::Help => {
            let lines = Vec::from([
                "h - move left",
                "j - move down",
                "k - move up",
                "l - move right",
                "H - move task to the left",
                "J - move task to the down",
                "K - move task to the up",
                "L - move task to the right",
                "q - quit",
                "a - add task",
                "e - edit task",
                "d - delete task",
                "f - focus",
            ])
            .iter()
            .map(|s| ListItem::new(s.to_string()))
            .collect::<Vec<ListItem>>();
            let help = List::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(if app.current_column == Column::Done {
                            Color::Cyan
                        } else {
                            Color::White
                        }))
                        .title("Help"),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");
            f.render_widget(help, chunks[1]);
        }
    }
}

fn render_focus(f: &mut Frame<impl Backend>, app: &mut App) {
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
    if let Some(item) = &app.wip {
        let wip = Paragraph::new(item.to_string())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightCyan))
            .block(Block::default().borders(Borders::ALL).title("Wip"));
        f.render_widget(wip, focus_layout[1])
    }
}

fn render_input(f: &mut Frame<impl Backend>, app: &mut App) {
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
    f.set_cursor(
        input_layout[1].x + app.input.len() as u16 + 1,
        input_layout[1].y + 1,
    )
}

fn build_task_list<'a>(list: &Vec<String>, title: &str, is_selected: bool) -> List<'a> {
    let mut lines = vec![];
    for item in list.iter() {
        lines.push(ListItem::new(item.to_string()));
    }
    List::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(if is_selected {
                    Color::Cyan
                } else {
                    Color::White
                }))
                .title(title.to_string()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ")
}

fn render_overview(app: &mut App, chunks: &[Rect], f: &mut Frame<impl Backend>) {
    let todo_list = build_task_list(&app.todo.items, "Todo", app.current_column == Column::Todo);
    let mut todo_list_state = ListState::default().with_selected(app.todo.index);

    let wip_tasks: Vec<String> = match &app.wip {
        Some(task) => vec![task.to_string()],
        None => vec![],
    };

    let wip_list = build_task_list(&wip_tasks, "Wip", app.current_column == Column::Wip);
    let mut wip_list_state = ListState::default().with_selected(app.wip.as_ref().map(|_| 0));

    let done_list = build_task_list(&app.todo.items, "Todo", app.current_column == Column::Todo);
    let mut done_list_state = ListState::default().with_selected(app.todo.index);

    f.render_stateful_widget(todo_list, chunks[0], &mut todo_list_state);
    f.render_stateful_widget(wip_list, chunks[1], &mut wip_list_state);
    f.render_stateful_widget(done_list, chunks[2], &mut done_list_state);
}

use ratatui::{text::Span, widgets::{Paragraph, Block, Borders}, style::{Style, Color}, Frame, backend::Backend, layout::Rect};

pub struct TaskInput {
    input: String,
    position: usize,
}

impl TaskInput {
    pub fn new() -> Self {
        TaskInput { input: String::new(), position: 0 }
    }

    pub fn add_char(&mut self, c: char) {
        self.input.insert(self.position, c);
        self.position += 1;
    }

    pub fn remove_char(&mut self) {
        if self.position > 0 {
            self.input.remove(self.position - 1);
            self.position -= 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.position > 0 {
            self.position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.position < self.input.len() {
            self.position += 1;
        }
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let text = Span::raw(&self.input);
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(paragraph, area);
    }
}


use ratatui::{
    widgets::{ListState, List, ListItem},
    text::Span,
    style::{Style, Color, Modifier}, Frame, backend::Backend, layout::Rect,
};

pub struct TaskList {
    items: Vec<String>,
    pub(crate) state: ListState,
}

impl TaskList {
    pub fn new(items: Vec<String>) -> Self {
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }

        TaskList { items , state }
    }

    pub fn render<B: Backend> (&mut self, frame: &mut Frame<B>, area: Rect) {
        let items: Vec<_> = self.items
            .iter()
            .map(|item| {
                let content = Span::styled(
                    item, 
                    Style::default().fg(Color::White)
                );
                ListItem::new(content)
            })
                .collect();

        let item_list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        frame.render_stateful_widget(item_list, area, &mut self.state);
    }

    pub fn move_selection_up(&mut self) {
        if let Some(selected) = self.state.selected() {
            let new_selection = if selected > 0 {
                selected - 1
            } else {
                self.items.len() - 1
            };
            self.state.select(Some(new_selection));
        }
    }

    pub fn move_selection_down(&mut self) {
        if let Some(selected) = self.state.selected() {
            let new_selection = if selected < self.items.len() - 1 {
                selected + 1
            } else {
                0
            };
            self.state.select(Some(new_selection));
        }
    }

    pub fn add_task(&mut self, task: String) {
        self.items.push(task);
        self.state.select(Some(self.items.len() - 1));
    }

    pub fn delete_task(&mut self) -> Option<String> {
        if let Some(selected) = self.state.selected() {
            let task = self.items.remove(selected);
            let next_selected = if self.items.len() == 0 { None } else { Some(selected.saturating_sub(1)) };
            self.state.select(next_selected);
            return Some(task)
        }
        None
    }

    pub fn update_task(&mut self, new_task: String) {
        if let Some(selected) = self.state.selected() {
            self.items[selected] = new_task;
        }
    }

    pub fn get_task(&mut self) -> Option<&String> {
        if let Some(selected) = self.state.selected() {
            return self.items.get(selected).to_owned();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_list() {
        let mut task_list = TaskList::new(Vec::new());

        // Add some tasks
        task_list.add_task("Task 1".to_string());
        assert_eq!(task_list.items.len(), 1);
        assert_eq!(task_list.state.selected().unwrap(), 0);

        task_list.add_task("Task 2".to_string());
        assert_eq!(task_list.items.len(), 2);
        assert_eq!(task_list.state.selected().unwrap(), 1);

        // Move selection up
        task_list.move_selection_up();
        assert_eq!(task_list.state.selected().unwrap(), 0);

        // Move selection down
        task_list.move_selection_down();
        assert_eq!(task_list.state.selected().unwrap(), 1);

        // Delete a task
        task_list.delete_task();
        assert_eq!(task_list.items.len(), 1);
        assert_eq!(task_list.state.selected().unwrap(), 0);

        // Update a task
        task_list.update_task("Updated task".to_string());
        assert_eq!(task_list.items[0], "Updated task");
    }
}


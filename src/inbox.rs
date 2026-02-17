use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Widget},
};

use crate::email::Email;

#[derive(Debug)]
pub struct Inbox {
    pub emails: Vec<Email>,
    pub selected_emails: Vec<usize>,
    cursor: usize, 
    visual_anchor: Option<usize>,
    list_state: ListState,
}

impl Inbox {
    pub fn new(emails: Vec<Email>) -> Self {
        let mut list_state = ListState::default();
        let cursor = 0;
        if !emails.is_empty() {
            list_state.select(Some(cursor));
        }
        Self {
            emails,
            selected_emails: vec![],
            cursor,
            visual_anchor: None,
            list_state,
        }
    }

    pub fn is_visual(&self) -> bool {
        self.visual_anchor.is_some() 
    }

    pub fn toggle_visual(&mut self) {
        if self.visual_anchor.is_some() {
            self.visual_anchor = None;
            self.selected_emails.clear();
        } else {
            self.visual_anchor = Some(self.cursor);
            self.selected_emails = vec![self.cursor];
        }
    }

    pub fn move_down(&mut self) {
        if self.emails.is_empty() {
            return;
        }
        self.cursor = (self.cursor + 1).min(self.emails.len() - 1);
        self.list_state.select(Some(self.cursor));
        self.sync_visual_selection();
    }
    
    pub fn move_up(&mut self) {
        if self.emails.is_empty() {
            return;
        }
        self.cursor = self.cursor.saturating_sub(1);
        self.list_state.select(Some(self.cursor));
        self.sync_visual_selection();
    }

    pub fn sync_visual_selection(&mut self) {
        let Some(anchor) = self.visual_anchor else { return };
        let (a, b) = if anchor <= self.cursor {
            (anchor, self.cursor)
        } else {
            (self.cursor, anchor)
        };
        self.selected_emails.clear();
        self.selected_emails.extend(a..=b);
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let is_selected = |i: usize| self.selected_emails.contains(&i);

        let items = self
            .emails
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let unread = if e.unread { "●" } else { " " };
                let line = format!("{unread} {:<18} {}", e.from, e.subject);

                let mut style = Style::default();
                if is_selected(i) {
                    style = style.bg(Color::Blue).fg(Color::White);
                }
                ListItem::new(line).style(style)
            })
            .collect::<Vec<_>>();

        let title = if self.is_visual() { "Inbox --VISUAL-- " } else { "Inbox" };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("▶ ");

        // stateful render: List uses list_state to know the cursor row
        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }
}

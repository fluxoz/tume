use std::{any, default, io};
use color_eyre::eyre::Context;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    DefaultTerminal, Frame, buffer::Buffer, layout::{Constraint, Direction, Flex, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, symbols::border, text::{Line, Text}, widgets::{Block, List, Paragraph, RatatuiLogo, RatatuiMascot, Widget, StatefulWidget, ListItem, ListState}
};


pub mod inbox;
pub mod email;
pub mod db;

use crate::{email::Email, inbox::Inbox};
use crate::db::Db;

#[derive(Debug)]
pub struct App {
    counter: u8,
    exit: bool,
    inbox: Inbox,
    view: View,
}

impl Default for App {
    fn default() -> Self {
        let emails = Email::from_slice(&["test1", "test2", "test3"]);
        Self {
            counter: 0,
            exit: false,
            inbox: Inbox::new(emails),
            view: View::Inbox(false),
        }
    }
}

impl App {
    pub fn with_emails(emails: Vec<Email>) -> Self {
        Self {
            counter: 0,
            exit: false,
            inbox: Inbox::new(emails),
            view: View::Inbox(false),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum View {
    Inbox(bool),
    Compose,
    Onboarding,
}

impl Default for View {
    fn default() -> Self {
        View::Inbox(false)
    }
}


impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn decrement_counter(&mut self) {
        self.counter = self.counter.wrapping_sub(1);
    }

    fn increment_counter(&mut self) {
        self.counter =  self.counter.wrapping_add(1);
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn toggle_preview(&mut self) {
        if let View::Inbox(show_preview) = self.view {
            self.view = View::Inbox(!show_preview);
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('p') => self.toggle_preview(),
            
            // inbox navigation
            KeyCode::Char('j') => self.inbox.move_down(),
            KeyCode::Char('k') => self.inbox.move_up(),
            KeyCode::Char('v') => self.inbox.toggle_visual(),
            _ => {}
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn render_inbox(&mut self, area: Rect, buf: &mut Buffer) {
        assert!(matches!(self.view, View::Inbox(_)));
        let show_preview = match self.view {
            View::Inbox(b) => b,
            _ => unreachable!("assert above guarantees this is inbox")
        };
        match show_preview {
            true => {
                let layout = Layout::horizontal([Constraint::Percentage(100)]);
                let [inbox_area] = layout.areas(area);
                self.inbox.render(inbox_area, buf);
            },
            false => {
                let layout = Layout::horizontal([Constraint::Percentage(80), Constraint::Percentage(20)]);
                let [inbox_area, preview_area] = layout.areas(area);
                self.inbox.render(inbox_area, buf);
                // preview
                let preview_block = Block::bordered().title("Preview");
                let preview = Paragraph::new("This is the preview area!").block(preview_block);
                preview.render(preview_area, buf);
            },
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.view {
            View::Inbox(_) => self.render_inbox(area, buf),
            _ => self.render_inbox(area, buf),
        }
    }
}

fn main() -> io::Result<()> {
    use futures::executor::block_on;
    
    // Load emails from database
    let emails = block_on(async {
        match Db::open_local("tume.db").await {
            Ok(db) => {
                // Ensure schema is set up - if migration fails, we can't load emails
                if db.migrate().await.is_err() {
                    return vec![];
                }
                // Load emails from database
                db.load_emails().await.unwrap_or_else(|_| vec![])
            }
            Err(_) => vec![]
        }
    });
    
    let mut app = if emails.is_empty() {
        App::default()
    } else {
        App::with_emails(emails)
    };
    
    ratatui::run(|terminal| app.run(terminal))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    #[test]
    fn render() {
        let mut app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        assert_eq!(buf, expected);
    }

    #[test]
    fn handle_key_event() {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.counter, 1);

        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.counter, 0);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert!(app.exit);
    }
}

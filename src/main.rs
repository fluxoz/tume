use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::style::Modifier;
use ratatui::text::Span;
use ratatui::{
    DefaultTerminal, 
    Frame,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    buffer::Buffer,
    widgets::{Block, Paragraph, Widget},
};

mod mailbox;
mod email;
mod folder;

#[derive(Default)]
struct Main {
    counter: u8,
    should_quit: bool
}

impl Main {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.should_quit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }
    
    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }
}

fn main() -> io::Result<()>{
    ratatui::run(|terminal| Main::default().run(terminal))
}

impl Widget for &Main {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("TUME".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
                "Value: ".into(),
                self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

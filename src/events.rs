use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::io;

use crate::app::{Action, App, View};

pub fn handle_events(app: &mut App) -> io::Result<()> {
    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                handle_key_event(app, key);
            }
        }
    }
    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyEvent) {
    // Clear status message on any key press
    app.status_message = None;

    match app.current_view {
        View::InboxList => handle_inbox_keys(app, key),
        View::EmailDetail => handle_detail_keys(app, key),
    }
}

fn handle_inbox_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        // Vim-style navigation
        KeyCode::Char('j') | KeyCode::Down => app.next_email(),
        KeyCode::Char('k') | KeyCode::Up => app.previous_email(),
        
        // Open email
        KeyCode::Enter | KeyCode::Char('l') => app.open_email(),
        
        // Actions
        KeyCode::Char('d') => app.perform_action(Action::Delete),
        KeyCode::Char('a') => app.perform_action(Action::Archive),
        KeyCode::Char('r') => app.perform_action(Action::Reply),
        KeyCode::Char('c') => app.perform_action(Action::Compose),
        KeyCode::Char('f') => app.perform_action(Action::Forward),
        
        // Quit
        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
        
        _ => {}
    }
}

fn handle_detail_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        // Go back
        KeyCode::Char('h') | KeyCode::Esc => app.close_email(),
        
        // Actions (same as inbox)
        KeyCode::Char('d') => app.perform_action(Action::Delete),
        KeyCode::Char('a') => app.perform_action(Action::Archive),
        KeyCode::Char('r') => app.perform_action(Action::Reply),
        KeyCode::Char('f') => app.perform_action(Action::Forward),
        
        // Quit
        KeyCode::Char('q') => app.quit(),
        
        _ => {}
    }
}

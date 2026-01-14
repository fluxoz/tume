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
        View::Compose => handle_compose_keys(app, key),
        View::CredentialsSetup => handle_credentials_setup_keys(app, key),
        View::CredentialsUnlock => handle_credentials_unlock_keys(app, key),
        View::CredentialsManagement => handle_credentials_management_keys(app, key),
    }
}

fn handle_inbox_keys(app: &mut App, key: KeyEvent) {
    // Check if in visual mode
    if app.visual_mode {
        handle_visual_mode_keys(app, key);
        return;
    }

    match key.code {
        // Vim-style navigation
        KeyCode::Char('j') | KeyCode::Down => app.next_email(),
        KeyCode::Char('k') | KeyCode::Up => app.previous_email(),

        // Open email
        KeyCode::Enter | KeyCode::Char('l') => app.open_email(),

        // Toggle preview panel
        KeyCode::Char('p') => app.toggle_preview_panel(),

        // Enter visual mode with Shift+V (uppercase V)
        KeyCode::Char('V') => {
            app.enter_visual_mode();
        }

        // Account switching (1-9)
        KeyCode::Char(c @ '1'..='9') => {
            let index = (c as u8 - b'1') as usize;
            app.switch_to_account(index);
        }

        // Next/Previous account
        KeyCode::Char(']') => app.next_account(),
        KeyCode::Char('[') => app.prev_account(),

        // Tab to cycle through accounts
        KeyCode::Tab => app.next_account(),

        // Actions
        KeyCode::Char('d') => app.perform_action(Action::Delete),
        KeyCode::Char('a') => app.perform_action(Action::Archive),
        KeyCode::Char('r') => app.perform_action(Action::Reply),
        KeyCode::Char('c') => app.perform_action(Action::Compose),
        KeyCode::Char('f') => app.perform_action(Action::Forward),

        // Credentials management
        KeyCode::Char('m') => app.enter_credentials_management(),

        // Quit
        KeyCode::Char('q') => app.quit(),

        _ => {}
    }
}

fn handle_visual_mode_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        // Vim-style navigation (extend selection)
        KeyCode::Char('j') | KeyCode::Down => app.next_email(),
        KeyCode::Char('k') | KeyCode::Up => app.previous_email(),

        // Batch actions
        KeyCode::Char('d') => app.perform_batch_action(Action::Delete),
        KeyCode::Char('a') => app.perform_batch_action(Action::Archive),

        // Exit visual mode
        KeyCode::Esc | KeyCode::Char('v') | KeyCode::Char('V') => app.exit_visual_mode(),

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

fn handle_compose_keys(app: &mut App, key: KeyEvent) {
    use crate::app::ComposeMode;

    if let Some(ref compose) = app.compose_state {
        match compose.mode {
            ComposeMode::Normal => handle_compose_normal_keys(app, key),
            ComposeMode::Insert => handle_compose_insert_keys(app, key),
        }
    }
}

fn handle_compose_normal_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        // Enter insert mode
        KeyCode::Char('i') => app.compose_enter_insert_mode(),

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => app.compose_next_field(),
        KeyCode::Char('k') | KeyCode::Up => app.compose_previous_field(),

        // Clear current field
        KeyCode::Char('d') => app.compose_clear_field(),

        // Toggle preview
        KeyCode::Char('p') => app.compose_toggle_preview(),

        // Save draft
        KeyCode::Char('w') => app.save_current_draft(),

        // Exit compose mode
        KeyCode::Esc => app.exit_compose_mode(),
        KeyCode::Char('q') => app.exit_compose_mode(),

        _ => {}
    }
}

fn handle_compose_insert_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        // Exit insert mode
        KeyCode::Esc => app.compose_exit_insert_mode(),

        // Text input
        KeyCode::Char(c) => app.compose_insert_char(c),

        // Backspace
        KeyCode::Backspace => app.compose_delete_char(),

        // Enter (newline for body only)
        KeyCode::Enter => app.compose_insert_newline(),

        // Cursor movement
        KeyCode::Left => app.compose_move_cursor_left(),
        KeyCode::Right => app.compose_move_cursor_right(),

        _ => {}
    }
}

fn handle_credentials_setup_keys(app: &mut App, key: KeyEvent) {
    // Check if we're in provider selection mode
    let in_provider_selection = app.credentials_setup_state
        .as_ref()
        .map(|s| s.provider_selection_mode)
        .unwrap_or(false);

    if in_provider_selection {
        // Provider selection mode keys
        match key.code {
            // Navigation
            KeyCode::Char('j') | KeyCode::Down => {
                app.credentials_setup_next_provider();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.credentials_setup_prev_provider();
            }

            // Select provider
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                app.credentials_setup_select_provider();
            }

            // Cancel
            KeyCode::Esc | KeyCode::Char('q') => {
                app.credentials_setup_cancel();
            }

            _ => {}
        }
    } else {
        // Field entry mode - check Normal vs Insert
        let in_insert_mode = app.credentials_setup_state
            .as_ref()
            .map(|s| s.mode == crate::app::CredentialsMode::Insert)
            .unwrap_or(false);

        if in_insert_mode {
            handle_credentials_setup_insert_keys(app, key);
        } else {
            handle_credentials_setup_normal_keys(app, key);
        }
    }
}

fn handle_credentials_setup_normal_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        // Enter insert mode
        KeyCode::Char('i') => {
            app.credentials_setup_enter_insert_mode();
        }

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            app.credentials_setup_next_field();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.credentials_setup_prev_field();
        }
        KeyCode::Tab => {
            app.credentials_setup_next_field();
        }
        KeyCode::BackTab => {
            app.credentials_setup_prev_field();
        }

        // Back to provider selection
        KeyCode::Char('h') | KeyCode::Left => {
            if app.credentials_setup_state
                .as_ref()
                .map(|s| s.can_navigate_back_to_providers())
                .unwrap_or(false) 
            {
                app.credentials_setup_back_to_providers();
            }
        }

        // Toggle password visibility
        KeyCode::Char('P') => {
            app.credentials_setup_toggle_password_visibility();
        }

        // Save
        KeyCode::Enter => {
            app.credentials_setup_save();
        }

        // Cancel
        KeyCode::Esc | KeyCode::Char('q') => {
            app.credentials_setup_cancel();
        }

        _ => {}
    }
}

fn handle_credentials_setup_insert_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        // Exit insert mode
        KeyCode::Esc => {
            app.credentials_setup_exit_insert_mode();
        }

        // Text input
        KeyCode::Char(c) => {
            app.credentials_setup_insert_char(c);
        }

        // Backspace
        KeyCode::Backspace => {
            app.credentials_setup_delete_char();
        }

        // Cursor movement
        KeyCode::Left => {
            app.credentials_setup_cursor_left();
        }
        KeyCode::Right => {
            app.credentials_setup_cursor_right();
        }

        _ => {}
    }
}

fn handle_credentials_unlock_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        // Submit
        KeyCode::Enter => {
            app.credentials_unlock_submit();
        }

        // Cancel (quit)
        KeyCode::Esc => {
            app.credentials_unlock_cancel();
        }

        // Text input
        KeyCode::Char(c) => {
            app.credentials_unlock_insert_char(c);
        }

        // Backspace
        KeyCode::Backspace => {
            app.credentials_unlock_delete_char();
        }

        // Cursor movement
        KeyCode::Left => {
            app.credentials_unlock_cursor_left();
        }
        KeyCode::Right => {
            app.credentials_unlock_cursor_right();
        }

        _ => {}
    }
}

fn handle_credentials_management_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        // Reset credentials
        KeyCode::Char('r') => {
            app.credentials_reset();
        }

        // Go back
        KeyCode::Esc => {
            app.exit_credentials_management();
        }

        _ => {}
    }
}

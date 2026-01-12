use std::fmt;

#[derive(Debug, Clone)]
pub struct Email {
    pub from: String,
    pub subject: String,
    pub preview: String,
    pub body: String,
    pub date: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    InboxList,
    EmailDetail,
    Compose,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComposeMode {
    Normal,
    Insert,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComposeField {
    Recipients,
    Subject,
    Body,
}

#[derive(Debug, Clone)]
pub struct ComposeState {
    pub recipients: String,
    pub subject: String,
    pub body: String,
    pub current_field: ComposeField,
    pub mode: ComposeMode,
    pub show_preview: bool,
    pub cursor_position: usize,
    pub initial_traversal_complete: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Delete,
    Archive,
    Reply,
    Compose,
    Forward,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Delete => write!(f, "Delete (d)"),
            Action::Archive => write!(f, "Archive (a)"),
            Action::Reply => write!(f, "Reply (r)"),
            Action::Compose => write!(f, "Compose (c)"),
            Action::Forward => write!(f, "Forward (f)"),
        }
    }
}

pub struct App {
    pub emails: Vec<Email>,
    pub current_view: View,
    pub selected_index: usize,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub compose_state: Option<ComposeState>,
}

impl App {
    pub fn new() -> Self {
        Self {
            emails: Self::mock_emails(),
            current_view: View::InboxList,
            selected_index: 0,
            should_quit: false,
            status_message: None,
            compose_state: None,
        }
    }

    fn mock_emails() -> Vec<Email> {
        vec![
            Email {
                from: "alice@example.com".to_string(),
                subject: "Project Update: Q1 Planning".to_string(),
                preview: "Hi team, I wanted to share some updates on our Q1 planning...".to_string(),
                body: "Hi team,\n\nI wanted to share some updates on our Q1 planning. We've made significant progress on the roadmap and I'd like to schedule a meeting to discuss next steps.\n\nLooking forward to your feedback.\n\nBest regards,\nAlice".to_string(),
                date: "2026-01-10 14:30".to_string(),
            },
            Email {
                from: "bob@example.com".to_string(),
                subject: "Meeting notes from yesterday".to_string(),
                preview: "Here are the notes from our meeting yesterday...".to_string(),
                body: "Here are the notes from our meeting yesterday:\n\n1. Discussed new feature requirements\n2. Reviewed timeline for implementation\n3. Assigned tasks to team members\n\nPlease review and let me know if I missed anything.\n\nBob".to_string(),
                date: "2026-01-10 09:15".to_string(),
            },
            Email {
                from: "notifications@github.com".to_string(),
                subject: "[fluxoz/tume] New issue opened: Create TUI stub".to_string(),
                preview: "A new issue has been opened in your repository...".to_string(),
                body: "A new issue has been opened in your repository fluxoz/tume:\n\nTitle: Create a TUI stub for this project\n\nThis project is meant to be a TUI email client...".to_string(),
                date: "2026-01-09 22:45".to_string(),
            },
            Email {
                from: "charlie@example.com".to_string(),
                subject: "Re: Budget approval request".to_string(),
                preview: "Thanks for submitting the budget request...".to_string(),
                body: "Thanks for submitting the budget request. I've reviewed the numbers and everything looks good. Approved!\n\nCharlie".to_string(),
                date: "2026-01-09 16:20".to_string(),
            },
            Email {
                from: "newsletter@techblog.com".to_string(),
                subject: "Weekly Tech Digest: Rust 1.92 Released".to_string(),
                preview: "This week in tech: Rust 1.92 brings exciting new features...".to_string(),
                body: "This week in tech:\n\n- Rust 1.92 Released with improved compile times\n- New TUI libraries gaining popularity\n- Terminal applications making a comeback\n\nRead more at techblog.com".to_string(),
                date: "2026-01-09 08:00".to_string(),
            },
        ]
    }

    pub fn next_email(&mut self) {
        if !self.emails.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.emails.len() - 1);
        }
    }

    pub fn previous_email(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn open_email(&mut self) {
        if !self.emails.is_empty() && self.current_view == View::InboxList {
            self.current_view = View::EmailDetail;
        }
    }

    pub fn close_email(&mut self) {
        if self.current_view == View::EmailDetail {
            self.current_view = View::InboxList;
        }
    }

    pub fn perform_action(&mut self, action: Action) {
        match action {
            Action::Delete => {
                if !self.emails.is_empty() {
                    let email = &self.emails[self.selected_index];
                    self.status_message = Some(format!("Deleted email: {}", email.subject));
                }
            }
            Action::Archive => {
                if !self.emails.is_empty() {
                    let email = &self.emails[self.selected_index];
                    self.status_message = Some(format!("Archived email: {}", email.subject));
                }
            }
            Action::Reply => {
                if !self.emails.is_empty() {
                    let email = &self.emails[self.selected_index];
                    self.status_message = Some(format!("Replying to: {}", email.from));
                }
            }
            Action::Compose => {
                self.enter_compose_mode();
            }
            Action::Forward => {
                if !self.emails.is_empty() {
                    let email = &self.emails[self.selected_index];
                    self.status_message = Some(format!("Forwarding email: {}", email.subject));
                }
            }
        }
    }

    pub fn enter_compose_mode(&mut self) {
        self.compose_state = Some(ComposeState {
            recipients: String::new(),
            subject: String::new(),
            body: String::new(),
            current_field: ComposeField::Recipients,
            mode: ComposeMode::Normal,
            show_preview: false,
            cursor_position: 0,
            initial_traversal_complete: false,
        });
        self.current_view = View::Compose;
    }

    pub fn exit_compose_mode(&mut self) {
        self.compose_state = None;
        self.current_view = View::InboxList;
    }

    pub fn compose_next_field(&mut self) {
        if let Some(ref mut compose) = self.compose_state {
            if compose.mode == ComposeMode::Normal {
                compose.current_field = match compose.current_field {
                    ComposeField::Recipients => ComposeField::Subject,
                    ComposeField::Subject => ComposeField::Body,
                    ComposeField::Body => ComposeField::Recipients,
                };
                // Reset cursor to end of field when switching
                compose.cursor_position = match compose.current_field {
                    ComposeField::Recipients => compose.recipients.len(),
                    ComposeField::Subject => compose.subject.len(),
                    ComposeField::Body => compose.body.len(),
                };
            }
        }
    }

    pub fn compose_previous_field(&mut self) {
        if let Some(ref mut compose) = self.compose_state {
            if compose.mode == ComposeMode::Normal {
                compose.current_field = match compose.current_field {
                    ComposeField::Recipients => ComposeField::Body,
                    ComposeField::Subject => ComposeField::Recipients,
                    ComposeField::Body => ComposeField::Subject,
                };
                // Reset cursor to end of field when switching
                compose.cursor_position = match compose.current_field {
                    ComposeField::Recipients => compose.recipients.len(),
                    ComposeField::Subject => compose.subject.len(),
                    ComposeField::Body => compose.body.len(),
                };
            }
        }
    }

    pub fn compose_enter_insert_mode(&mut self) {
        if let Some(ref mut compose) = self.compose_state {
            if compose.mode == ComposeMode::Normal {
                compose.mode = ComposeMode::Insert;
                // Set cursor to end of current field
                compose.cursor_position = match compose.current_field {
                    ComposeField::Recipients => compose.recipients.len(),
                    ComposeField::Subject => compose.subject.len(),
                    ComposeField::Body => compose.body.len(),
                };
            }
        }
    }

    pub fn compose_exit_insert_mode(&mut self) {
        if let Some(ref mut compose) = self.compose_state {
            if compose.mode == ComposeMode::Insert {
                compose.mode = ComposeMode::Normal;
                
                // Auto-advance during initial traversal until we reach Body field
                if !compose.initial_traversal_complete {
                    // Check if we're on Body field - if so, mark traversal as complete
                    if compose.current_field == ComposeField::Body {
                        compose.initial_traversal_complete = true;
                    } else {
                        // Auto-advance to next field during initial setup
                        self.compose_next_field();
                    }
                }
                // After initial traversal, stay on current field when exiting insert mode
            }
        }
    }

    pub fn compose_toggle_preview(&mut self) {
        if let Some(ref mut compose) = self.compose_state {
            if compose.mode == ComposeMode::Normal {
                compose.show_preview = !compose.show_preview;
            }
        }
    }

    pub fn compose_insert_char(&mut self, c: char) {
        if let Some(ref mut compose) = self.compose_state {
            if compose.mode == ComposeMode::Insert {
                let text = match compose.current_field {
                    ComposeField::Recipients => &mut compose.recipients,
                    ComposeField::Subject => &mut compose.subject,
                    ComposeField::Body => &mut compose.body,
                };
                
                // Insert character at cursor position
                if compose.cursor_position <= text.len() {
                    text.insert(compose.cursor_position, c);
                    compose.cursor_position += 1;
                }
            }
        }
    }

    pub fn compose_delete_char(&mut self) {
        if let Some(ref mut compose) = self.compose_state {
            if compose.mode == ComposeMode::Insert && compose.cursor_position > 0 {
                let text = match compose.current_field {
                    ComposeField::Recipients => &mut compose.recipients,
                    ComposeField::Subject => &mut compose.subject,
                    ComposeField::Body => &mut compose.body,
                };
                
                // Remove character before cursor
                compose.cursor_position -= 1;
                text.remove(compose.cursor_position);
            }
        }
    }

    pub fn compose_insert_newline(&mut self) {
        if let Some(ref mut compose) = self.compose_state {
            if compose.mode == ComposeMode::Insert && compose.current_field == ComposeField::Body {
                compose.body.insert(compose.cursor_position, '\n');
                compose.cursor_position += 1;
            }
        }
    }
    
    pub fn compose_move_cursor_left(&mut self) {
        if let Some(ref mut compose) = self.compose_state {
            if compose.mode == ComposeMode::Insert && compose.cursor_position > 0 {
                compose.cursor_position -= 1;
            }
        }
    }
    
    pub fn compose_move_cursor_right(&mut self) {
        if let Some(ref mut compose) = self.compose_state {
            if compose.mode == ComposeMode::Insert {
                let max_pos = match compose.current_field {
                    ComposeField::Recipients => compose.recipients.len(),
                    ComposeField::Subject => compose.subject.len(),
                    ComposeField::Body => compose.body.len(),
                };
                if compose.cursor_position < max_pos {
                    compose.cursor_position += 1;
                }
            }
        }
    }

    pub fn compose_clear_field(&mut self) {
        if let Some(ref mut compose) = self.compose_state {
            if compose.mode == ComposeMode::Normal {
                match compose.current_field {
                    ComposeField::Recipients => compose.recipients.clear(),
                    ComposeField::Subject => compose.subject.clear(),
                    ComposeField::Body => compose.body.clear(),
                }
            }
        }
    }

    // Stub methods for GPG and Yubikey hooks
    pub fn compose_encrypt_with_gpg(&mut self) {
        self.status_message = Some("GPG encryption hook (stub)".to_string());
    }

    pub fn compose_sign_with_yubikey(&mut self) {
        self.status_message = Some("Yubikey signing hook (stub)".to_string());
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn get_selected_email(&self) -> Option<&Email> {
        self.emails.get(self.selected_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_initialization() {
        let app = App::new();
        assert_eq!(app.current_view, View::InboxList);
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.should_quit, false);
        assert_eq!(app.emails.len(), 5);
    }

    #[test]
    fn test_navigation() {
        let mut app = App::new();
        assert_eq!(app.selected_index, 0);
        
        app.next_email();
        assert_eq!(app.selected_index, 1);
        
        app.next_email();
        assert_eq!(app.selected_index, 2);
        
        app.previous_email();
        assert_eq!(app.selected_index, 1);
        
        app.previous_email();
        assert_eq!(app.selected_index, 0);
        
        // Should not go below 0
        app.previous_email();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_navigation_bounds() {
        let mut app = App::new();
        
        // Move to the last email
        for _ in 0..10 {
            app.next_email();
        }
        
        // Should not exceed last index
        assert_eq!(app.selected_index, 4);
    }

    #[test]
    fn test_view_switching() {
        let mut app = App::new();
        assert_eq!(app.current_view, View::InboxList);
        
        app.open_email();
        assert_eq!(app.current_view, View::EmailDetail);
        
        app.close_email();
        assert_eq!(app.current_view, View::InboxList);
        
        // Opening from detail view should not change
        app.open_email();
        app.open_email();
        assert_eq!(app.current_view, View::EmailDetail);
    }

    #[test]
    fn test_actions() {
        let mut app = App::new();
        
        app.perform_action(Action::Delete);
        assert!(app.status_message.is_some());
        assert!(app.status_message.as_ref().unwrap().contains("Deleted"));
        
        app.perform_action(Action::Archive);
        assert!(app.status_message.as_ref().unwrap().contains("Archived"));
        
        app.perform_action(Action::Reply);
        assert!(app.status_message.as_ref().unwrap().contains("Replying"));
        
        app.perform_action(Action::Compose);
        assert_eq!(app.current_view, View::Compose);
        assert!(app.compose_state.is_some());
        
        app.exit_compose_mode();
        app.perform_action(Action::Forward);
        assert!(app.status_message.as_ref().unwrap().contains("Forwarding"));
    }

    #[test]
    fn test_quit() {
        let mut app = App::new();
        assert_eq!(app.should_quit, false);
        
        app.quit();
        assert_eq!(app.should_quit, true);
    }

    #[test]
    fn test_get_selected_email() {
        let mut app = App::new();
        
        let email = app.get_selected_email();
        assert!(email.is_some());
        assert_eq!(email.unwrap().from, "alice@example.com");
        
        app.next_email();
        let email = app.get_selected_email();
        assert!(email.is_some());
        assert_eq!(email.unwrap().from, "bob@example.com");
    }

    #[test]
    fn test_compose_mode_enter_exit() {
        let mut app = App::new();
        assert_eq!(app.current_view, View::InboxList);
        assert!(app.compose_state.is_none());

        app.enter_compose_mode();
        assert_eq!(app.current_view, View::Compose);
        assert!(app.compose_state.is_some());

        app.exit_compose_mode();
        assert_eq!(app.current_view, View::InboxList);
        assert!(app.compose_state.is_none());
    }

    #[test]
    fn test_compose_field_navigation() {
        let mut app = App::new();
        app.enter_compose_mode();

        let compose = app.compose_state.as_ref().unwrap();
        assert_eq!(compose.current_field, ComposeField::Recipients);

        app.compose_next_field();
        assert_eq!(app.compose_state.as_ref().unwrap().current_field, ComposeField::Subject);

        app.compose_next_field();
        assert_eq!(app.compose_state.as_ref().unwrap().current_field, ComposeField::Body);

        app.compose_next_field();
        assert_eq!(app.compose_state.as_ref().unwrap().current_field, ComposeField::Recipients);

        app.compose_previous_field();
        assert_eq!(app.compose_state.as_ref().unwrap().current_field, ComposeField::Body);
    }

    #[test]
    fn test_compose_insert_mode() {
        let mut app = App::new();
        app.enter_compose_mode();

        assert_eq!(app.compose_state.as_ref().unwrap().mode, ComposeMode::Normal);

        app.compose_enter_insert_mode();
        assert_eq!(app.compose_state.as_ref().unwrap().mode, ComposeMode::Insert);

        app.compose_exit_insert_mode();
        assert_eq!(app.compose_state.as_ref().unwrap().mode, ComposeMode::Normal);
        // Should auto-advance to Subject during initial traversal
        assert_eq!(app.compose_state.as_ref().unwrap().current_field, ComposeField::Subject);
        
        // Continue to Body
        app.compose_enter_insert_mode();
        app.compose_exit_insert_mode();
        assert_eq!(app.compose_state.as_ref().unwrap().current_field, ComposeField::Body);
        
        // Now that we're on Body, traversal is complete - Esc should stay on Body
        app.compose_enter_insert_mode();
        app.compose_exit_insert_mode();
        assert_eq!(app.compose_state.as_ref().unwrap().current_field, ComposeField::Body);
    }

    #[test]
    fn test_compose_text_input() {
        let mut app = App::new();
        app.enter_compose_mode();
        app.compose_enter_insert_mode();

        app.compose_insert_char('t');
        app.compose_insert_char('e');
        app.compose_insert_char('s');
        app.compose_insert_char('t');

        assert_eq!(app.compose_state.as_ref().unwrap().recipients, "test");

        app.compose_delete_char();
        assert_eq!(app.compose_state.as_ref().unwrap().recipients, "tes");
    }

    #[test]
    fn test_compose_preview_toggle() {
        let mut app = App::new();
        app.enter_compose_mode();

        assert_eq!(app.compose_state.as_ref().unwrap().show_preview, false);

        app.compose_toggle_preview();
        assert_eq!(app.compose_state.as_ref().unwrap().show_preview, true);

        app.compose_toggle_preview();
        assert_eq!(app.compose_state.as_ref().unwrap().show_preview, false);
    }

    #[test]
    fn test_compose_clear_field() {
        let mut app = App::new();
        app.enter_compose_mode();
        app.compose_enter_insert_mode();

        // Add text to recipients
        app.compose_insert_char('t');
        app.compose_insert_char('e');
        app.compose_insert_char('s');
        app.compose_insert_char('t');
        assert_eq!(app.compose_state.as_ref().unwrap().recipients, "test");

        // Exit insert mode and clear
        app.compose_exit_insert_mode();
        // Auto-advances to Subject during initial traversal
        assert_eq!(app.compose_state.as_ref().unwrap().current_field, ComposeField::Subject);
        app.compose_previous_field(); // Go back to Recipients
        app.compose_clear_field();
        assert_eq!(app.compose_state.as_ref().unwrap().recipients, "");

        // Test clearing subject
        app.compose_next_field(); // Move to subject
        app.compose_enter_insert_mode();
        app.compose_insert_char('s');
        app.compose_insert_char('u');
        app.compose_insert_char('b');
        assert_eq!(app.compose_state.as_ref().unwrap().subject, "sub");
        app.compose_exit_insert_mode(); // Auto-advances to Body during initial traversal
        assert_eq!(app.compose_state.as_ref().unwrap().current_field, ComposeField::Body);
        app.compose_previous_field(); // Go back to Subject
        app.compose_clear_field();
        assert_eq!(app.compose_state.as_ref().unwrap().subject, "");
    }
}

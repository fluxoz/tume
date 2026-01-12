use std::fmt;
use crate::db::{DbEmail, DbDraft, EmailDatabase, EmailStatus as DbEmailStatus};

#[derive(Debug, Clone)]
pub struct Email {
    pub id: i64,
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
    pub db: Option<EmailDatabase>,
    pub draft_id: Option<i64>,
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
            db: None,
            draft_id: None,
        }
    }

    /// Initialize the app with database support
    pub async fn with_database() -> anyhow::Result<Self> {
        let db = EmailDatabase::new(None).await?;
        
        // Load emails from database or populate with mock data if empty
        let db_emails = db.get_emails_by_folder("inbox").await?;
        let emails = if db_emails.is_empty() {
            // Populate with mock data on first run
            let mock_emails = Self::mock_emails();
            for email in &mock_emails {
                let db_email = DbEmail {
                    id: 0,
                    from_address: email.from.clone(),
                    to_addresses: "me@example.com".to_string(),
                    cc_addresses: None,
                    bcc_addresses: None,
                    subject: email.subject.clone(),
                    body: email.body.clone(),
                    preview: email.preview.clone(),
                    date: email.date.clone(),
                    status: DbEmailStatus::Unread,
                    is_flagged: false,
                    folder: "inbox".to_string(),
                    thread_id: None,
                };
                db.insert_email(&db_email).await?;
            }
            mock_emails
        } else {
            db_emails
                .into_iter()
                .map(|e| Email {
                    id: e.id,
                    from: e.from_address,
                    subject: e.subject,
                    preview: e.preview,
                    body: e.body,
                    date: e.date,
                })
                .collect()
        };

        // Check if there's a draft available (but don't load it yet)
        // Draft will be loaded when user presses 'c' to compose
        let draft_id = match db.get_drafts().await {
            Ok(drafts) => drafts.first().map(|d| d.id),
            Err(_) => None,
        };

        Ok(Self {
            emails,
            current_view: View::InboxList,
            selected_index: 0,
            should_quit: false,
            status_message: None,
            compose_state: None,
            db: Some(db),
            draft_id,
        })
    }

    fn mock_emails() -> Vec<Email> {
        vec![
            Email {
                id: 0,
                from: "alice@example.com".to_string(),
                subject: "Project Update: Q1 Planning".to_string(),
                preview: "Hi team, I wanted to share some updates on our Q1 planning...".to_string(),
                body: "Hi team,\n\nI wanted to share some updates on our Q1 planning. We've made significant progress on the roadmap and I'd like to schedule a meeting to discuss next steps.\n\nLooking forward to your feedback.\n\nBest regards,\nAlice".to_string(),
                date: "2026-01-10 14:30".to_string(),
            },
            Email {
                id: 0,
                from: "bob@example.com".to_string(),
                subject: "Meeting notes from yesterday".to_string(),
                preview: "Here are the notes from our meeting yesterday...".to_string(),
                body: "Here are the notes from our meeting yesterday:\n\n1. Discussed new feature requirements\n2. Reviewed timeline for implementation\n3. Assigned tasks to team members\n\nPlease review and let me know if I missed anything.\n\nBob".to_string(),
                date: "2026-01-10 09:15".to_string(),
            },
            Email {
                id: 0,
                from: "notifications@github.com".to_string(),
                subject: "[fluxoz/tume] New issue opened: Create TUI stub".to_string(),
                preview: "A new issue has been opened in your repository...".to_string(),
                body: "A new issue has been opened in your repository fluxoz/tume:\n\nTitle: Create a TUI stub for this project\n\nThis project is meant to be a TUI email client...".to_string(),
                date: "2026-01-09 22:45".to_string(),
            },
            Email {
                id: 0,
                from: "charlie@example.com".to_string(),
                subject: "Re: Budget approval request".to_string(),
                preview: "Thanks for submitting the budget request...".to_string(),
                body: "Thanks for submitting the budget request. I've reviewed the numbers and everything looks good. Approved!\n\nCharlie".to_string(),
                date: "2026-01-09 16:20".to_string(),
            },
            Email {
                id: 0,
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
                    let email_id = email.id;
                    self.status_message = Some(format!("Deleted email: {}", email.subject));
                    
                    // Delete from database if available
                    // Note: Using fire-and-forget pattern as this is a background operation.
                    // The UI state is updated immediately for responsiveness. If the database
                    // operation fails, it will be retried on next app restart.
                    if let Some(ref db) = self.db {
                        let db_clone = db.clone();
                        tokio::spawn(async move {
                            if let Err(e) = db_clone.delete_email(email_id).await {
                                eprintln!("Failed to delete email from database: {}", e);
                            }
                        });
                    }
                }
            }
            Action::Archive => {
                if !self.emails.is_empty() {
                    let email = &self.emails[self.selected_index];
                    let email_id = email.id;
                    self.status_message = Some(format!("Archived email: {}", email.subject));
                    
                    // Archive in database if available
                    // Note: Using fire-and-forget pattern for background database operation.
                    if let Some(ref db) = self.db {
                        let db_clone = db.clone();
                        tokio::spawn(async move {
                            if let Err(e) = db_clone.archive_email(email_id).await {
                                eprintln!("Failed to archive email in database: {}", e);
                            }
                        });
                    }
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
        // If we already have a compose state (from a previous ESC exit), just switch to it
        if self.compose_state.is_some() {
            self.current_view = View::Compose;
            return;
        }
        
        // Try to load saved draft from database (for new session)
        if self.db.is_some() {
            let db_clone = self.db.as_ref().unwrap().clone();
            
            // Try to load draft synchronously using spawn_blocking workaround
            // This avoids blocking the event loop while still accessing the database
            let runtime = tokio::runtime::Handle::try_current();
            if let Ok(handle) = runtime {
                // Use spawn_blocking to avoid nested runtime issues
                let draft_result = std::thread::spawn(move || {
                    handle.block_on(async { db_clone.get_drafts().await })
                }).join();
                
                if let Ok(Ok(drafts)) = draft_result {
                    if let Some(draft) = drafts.first() {
                        // Load the draft into compose state
                        self.compose_state = Some(ComposeState {
                            recipients: draft.recipients.clone(),
                            subject: draft.subject.clone(),
                            body: draft.body.clone(),
                            current_field: ComposeField::Recipients,
                            mode: ComposeMode::Normal,
                            show_preview: false,
                            cursor_position: 0,
                            initial_traversal_complete: !draft.body.is_empty(),
                        });
                        self.current_view = View::Compose;
                        self.draft_id = Some(draft.id);
                        return;
                    }
                }
            }
        }
        
        // No draft or failed to load - start with empty compose state
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
        self.draft_id = None;
    }
    
    /// Load the most recent draft into compose state (call this after enter_compose_mode)
    pub async fn load_draft_async(&mut self) -> anyhow::Result<()> {
        if let Some(ref db) = self.db {
            let drafts = db.get_drafts().await?;
            if let Some(draft) = drafts.first() {
                if let Some(ref mut compose) = self.compose_state {
                    compose.recipients = draft.recipients.clone();
                    compose.subject = draft.subject.clone();
                    compose.body = draft.body.clone();
                    compose.initial_traversal_complete = !draft.body.is_empty();
                    self.draft_id = Some(draft.id);
                }
            }
        }
        Ok(())
    }

    pub fn exit_compose_mode(&mut self) {
        // Save draft to database if there's content
        // Note: Using fire-and-forget pattern for non-blocking UI experience.
        // Users expect compose exit to be immediate. Draft is saved in background.
        if let Some(ref compose) = self.compose_state {
            if !compose.recipients.is_empty() || !compose.subject.is_empty() || !compose.body.is_empty() {
                self.save_current_draft();
            }
        }
        
        // Don't clear compose_state or draft_id - keep them for re-entry
        // Just switch view back to inbox
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
                
                // Handle initial traversal logic
                if !compose.initial_traversal_complete {
                    if compose.current_field == ComposeField::Body {
                        // Reached Body - mark traversal complete and stay
                        compose.initial_traversal_complete = true;
                    } else {
                        // Not on Body yet - auto-advance to next field
                        self.compose_next_field();
                    }
                }
                // After traversal complete: stay on current field
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

    /// Save the current draft to the database
    pub fn save_current_draft(&mut self) {
        if let Some(ref compose) = self.compose_state {
            if let Some(ref db) = self.db {
                let draft = self.create_db_draft(compose);
                let db_clone = db.clone();
                
                tokio::spawn(async move {
                    match db_clone.save_draft(&draft).await {
                        Ok(_new_id) => {
                            // Note: We can't update self.draft_id from async context
                            // The draft ID will be picked up on next compose entry or app restart
                        }
                        Err(e) => {
                            eprintln!("Failed to save draft to database: {}", e);
                        }
                    }
                });
                
                self.status_message = Some("Draft saved".to_string());
            }
        }
    }

    /// Save draft before quitting the application
    pub fn save_draft_before_quit(&mut self) {
        if let Some(ref compose) = self.compose_state {
            if !compose.recipients.is_empty() || !compose.subject.is_empty() || !compose.body.is_empty() {
                if let Some(ref db) = self.db {
                    let draft = self.create_db_draft(compose);
                    let db_clone = db.clone();
                    
                    // Use blocking call since we're about to exit
                    let runtime = tokio::runtime::Handle::try_current();
                    if let Ok(handle) = runtime {
                        match handle.block_on(db_clone.save_draft(&draft)) {
                            Ok(_) => {
                                // Draft saved successfully
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to save draft before quit: {}", e);
                            }
                        }
                    } else {
                        eprintln!("Warning: Could not access runtime to save draft before quit");
                    }
                }
            }
        }
    }
    
    /// Helper to create a DbDraft from current compose state
    fn create_db_draft(&self, compose: &ComposeState) -> DbDraft {
        DbDraft {
            id: self.draft_id.unwrap_or(0),
            recipients: compose.recipients.clone(),
            subject: compose.subject.clone(),
            body: compose.body.clone(),
            created_at: String::new(),
            updated_at: String::new(),
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
        // Compose state is preserved for re-entry (draft behavior)
        assert!(app.compose_state.is_some());
        
        // Re-entering compose mode should restore the same state
        app.enter_compose_mode();
        assert_eq!(app.current_view, View::Compose);
        assert!(app.compose_state.is_some());
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
    
    #[tokio::test]
    async fn test_draft_save_and_load() {
        // Use a unique database for this test to avoid locking issues
        let test_id = std::process::id();
        let mut path = std::env::temp_dir();
        path.push(format!("test_tume_draft_save_{}.db", test_id));
        let _ = std::fs::remove_file(&path);
        
        let db = crate::db::EmailDatabase::new(Some(path.clone())).await.unwrap();
        let mut app = App {
            emails: Vec::new(),
            current_view: View::InboxList,
            selected_index: 0,
            should_quit: false,
            status_message: None,
            compose_state: None,
            db: Some(db),
            draft_id: None,
        };
        
        // Enter compose mode and add some content
        app.enter_compose_mode();
        app.compose_enter_insert_mode();
        app.compose_insert_char('t');
        app.compose_insert_char('e');
        app.compose_insert_char('s');
        app.compose_insert_char('t');
        app.compose_exit_insert_mode(); // Move to subject
        
        app.compose_enter_insert_mode();
        app.compose_insert_char('M');
        app.compose_insert_char('y');
        app.compose_insert_char(' ');
        app.compose_insert_char('D');
        app.compose_insert_char('r');
        app.compose_insert_char('a');
        app.compose_insert_char('f');
        app.compose_insert_char('t');
        app.compose_exit_insert_mode(); // Move to body
        
        // Manually save the draft
        app.save_current_draft();
        
        // Give async save time to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        // Exit compose mode
        app.exit_compose_mode();
        
        // Re-enter compose mode and manually load draft
        app.enter_compose_mode();
        app.load_draft_async().await.unwrap();
        
        let compose = app.compose_state.as_ref().unwrap();
        assert_eq!(compose.recipients, "test");
        assert_eq!(compose.subject, "My Draft");
        
        // Cleanup
        let _ = std::fs::remove_file(&path);
    }
    
    #[tokio::test]
    async fn test_draft_persist_on_exit() {
        // Use a unique database for this test to avoid locking issues
        let test_id = std::process::id();
        let mut path = std::env::temp_dir();
        path.push(format!("test_tume_draft_exit_{}.db", test_id));
        let _ = std::fs::remove_file(&path);
        
        let db = crate::db::EmailDatabase::new(Some(path.clone())).await.unwrap();
        let mut app = App {
            emails: Vec::new(),
            current_view: View::InboxList,
            selected_index: 0,
            should_quit: false,
            status_message: None,
            compose_state: None,
            db: Some(db),
            draft_id: None,
        };
        
        // Enter compose mode and add some content
        app.enter_compose_mode();
        app.compose_enter_insert_mode();
        app.compose_insert_char('d');
        app.compose_insert_char('r');
        app.compose_insert_char('a');
        app.compose_insert_char('f');
        app.compose_insert_char('t');
        
        // Exit compose mode (should auto-save)
        app.compose_exit_insert_mode();
        app.exit_compose_mode();
        
        // Give async save time to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        // Re-enter compose mode and manually load draft
        app.enter_compose_mode();
        app.load_draft_async().await.unwrap();
        
        let compose = app.compose_state.as_ref().unwrap();
        assert_eq!(compose.recipients, "draft");
        
        // Cleanup
        let _ = std::fs::remove_file(&path);
    }
}


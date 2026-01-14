use crate::credentials::{Credentials, CredentialsManager, StorageBackend};
use crate::config::Config;
use crate::db::{DbAccount, DbDraft, DbEmail, EmailDatabase, EmailStatus as DbEmailStatus};
use std::collections::HashSet;
use std::fmt;

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
    CredentialsSetup,
    CredentialsUnlock,
    CredentialsManagement,
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

/// Field being edited in credentials setup
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CredentialField {
    ImapServer,
    ImapPort,
    ImapUsername,
    ImapPassword,
    SmtpServer,
    SmtpPort,
    SmtpUsername,
    SmtpPassword,
    MasterPassword,
    MasterPasswordConfirm,
}

/// Editing mode for credentials setup (similar to compose)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CredentialsMode {
    Normal,
    Insert,
}

/// State for credentials setup view
#[derive(Debug, Clone)]
pub struct CredentialsSetupState {
    pub imap_server: String,
    pub imap_port: String,
    pub imap_username: String,
    pub imap_password: String,
    pub smtp_server: String,
    pub smtp_port: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub master_password: String,
    pub master_password_confirm: String,
    pub current_field: CredentialField,
    pub cursor_position: usize,
    pub show_passwords: bool,
    pub selected_provider: Option<String>, // Provider ID if one was selected
    pub provider_selection_mode: bool, // Whether we're in provider selection mode
    pub provider_list_index: usize, // Selected index in provider list
    pub mode: CredentialsMode, // Normal or Insert mode
}

impl CredentialsSetupState {
    pub fn new(_backend: StorageBackend) -> Self {
        Self {
            imap_server: String::new(),
            imap_port: "993".to_string(),
            imap_username: String::new(),
            imap_password: String::new(),
            smtp_server: String::new(),
            smtp_port: "587".to_string(),
            smtp_username: String::new(),
            smtp_password: String::new(),
            master_password: String::new(),
            master_password_confirm: String::new(),
            current_field: CredentialField::ImapServer,
            cursor_position: 0,
            show_passwords: false,
            selected_provider: None,
            provider_selection_mode: true, // Start in provider selection mode
            provider_list_index: 0,
            mode: CredentialsMode::Normal, // Start in normal mode
        }
    }

    /// Apply a provider preset to this setup state
    pub fn apply_provider(&mut self, provider: &crate::providers::EmailProvider) {
        self.selected_provider = Some(provider.id.to_string());
        self.imap_server = provider.imap_server.to_string();
        self.imap_port = provider.imap_port.to_string();
        self.smtp_server = provider.smtp_server.to_string();
        self.smtp_port = provider.smtp_port.to_string();
        self.provider_selection_mode = false;
    }

    /// Check if user can navigate back to provider selection
    /// Only allowed in Normal mode, on the first field
    pub fn can_navigate_back_to_providers(&self) -> bool {
        !self.provider_selection_mode 
            && self.mode == CredentialsMode::Normal
            && self.current_field == CredentialField::ImapServer
    }
}

/// State for credentials unlock view (for encrypted file backend)
#[derive(Debug, Clone)]
pub struct CredentialsUnlockState {
    pub master_password: String,
    pub cursor_position: usize,
    pub error_message: Option<String>,
}

impl CredentialsUnlockState {
    pub fn new() -> Self {
        Self {
            master_password: String::new(),
            cursor_position: 0,
            error_message: None,
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
    pub show_preview_panel: bool,
    pub visual_mode: bool,
    pub visual_selections: HashSet<usize>,
    pub visual_anchor: Option<usize>,
    pub credentials_manager: Option<CredentialsManager>,
    pub credentials: Option<Credentials>,
    pub credentials_setup_state: Option<CredentialsSetupState>,
    pub credentials_unlock_state: Option<CredentialsUnlockState>,
    pub config: Config,
    pub accounts: Vec<DbAccount>,
    pub current_account_id: Option<i64>,
    pub email_sync_manager: Option<crate::email_sync::EmailSyncManager>,
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
            show_preview_panel: false,
            visual_mode: false,
            visual_selections: HashSet::new(),
            visual_anchor: None,
            credentials_manager: None,
            credentials: None,
            credentials_setup_state: None,
            credentials_unlock_state: None,
            config: Config::default(),
            accounts: Vec::new(),
            current_account_id: None,
            email_sync_manager: None,
        }
    }

    /// Initialize the app with database support
    pub async fn with_database(dev_mode: bool) -> anyhow::Result<Self> {
        let db = EmailDatabase::new(None).await?;

        // Load configuration
        let config = Config::load().unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load config: {}. Using defaults.", e);
            Config::default()
        });
        
        eprintln!("DEBUG: Config loaded. Accounts in config: {}", config.accounts.len());
        for (key, account) in &config.accounts {
            eprintln!("DEBUG: Config account '{}': {} ({})", key, account.name, account.email);
        }

        // Load accounts from database
        let accounts = db.get_accounts().await?;
        
        eprintln!("DEBUG: Accounts from DB: {}", accounts.len());
        for account in &accounts {
            eprintln!("DEBUG: DB account: {} ({})", account.name, account.email);
        }

        // Sync accounts from config to database if needed
        let accounts = Self::sync_accounts_from_config(&db, &config, accounts).await?;

        // Determine current account (default or first available)
        let current_account_id = accounts
            .iter()
            .find(|a| a.is_default)
            .or_else(|| accounts.first())
            .map(|a| a.id);

        // In dev mode, clear and reseed the inbox for testing
        if dev_mode {
            db.clear_inbox().await?;
        }

        // Load emails from database or populate with mock data if empty
        let db_emails = if let Some(acc_id) = current_account_id {
            db.get_emails_by_folder_and_account("inbox", Some(acc_id)).await?
        } else {
            db.get_emails_by_folder("inbox").await?
        };

        let emails = if db_emails.is_empty() {
            // Populate with mock data on first run or after clearing in dev mode
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
                    account_id: current_account_id,
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

        // Initialize credentials manager
        let credentials_manager = CredentialsManager::new();
        
        // Check if we have a real mailbox configured (in config or database)
        let has_configured_mailbox = !config.accounts.is_empty() || !accounts.is_empty();
        
        eprintln!("DEBUG: has_configured_mailbox = {} (config.accounts={}, db.accounts={})", 
            has_configured_mailbox, config.accounts.len(), accounts.len());
        eprintln!("DEBUG: credentials_exist = {}", credentials_manager.credentials_exist());
        
        // Determine initial view based on credentials and mailbox configuration
        let (initial_view, credentials, credentials_setup_state, credentials_unlock_state) = 
            if has_configured_mailbox && credentials_manager.credentials_exist() {
                // Have mailbox and credentials - check if they need unlocking
                if credentials_manager.backend() == StorageBackend::EncryptedFile {
                    // Credentials exist but need to be unlocked
                    (
                        View::CredentialsUnlock,
                        None,
                        None,
                        Some(CredentialsUnlockState::new()),
                    )
                } else {
                    // Credentials exist in keyring - load them automatically
                    match credentials_manager.load_credentials(None) {
                        Ok(creds) => (View::InboxList, Some(creds), None, None),
                        Err(_) => {
                            // Failed to load - show setup to re-enter credentials
                            (
                                View::CredentialsSetup,
                                None,
                                Some(CredentialsSetupState::new(credentials_manager.backend())),
                                None,
                            )
                        }
                    }
                }
            } else {
                // No configured mailbox or no credentials - show setup screen
                (
                    View::CredentialsSetup,
                    None,
                    Some(CredentialsSetupState::new(credentials_manager.backend())),
                    None,
                )
            };

        Ok(Self {
            emails,
            current_view: initial_view,
            selected_index: 0,
            should_quit: false,
            status_message: None,
            compose_state: None,
            db: Some(db),
            draft_id,
            show_preview_panel: false,
            visual_mode: false,
            visual_selections: HashSet::new(),
            visual_anchor: None,
            credentials_manager: Some(credentials_manager),
            credentials: credentials.clone(),
            credentials_setup_state,
            credentials_unlock_state,
            config,
            accounts,
            current_account_id,
            email_sync_manager: Some(crate::email_sync::EmailSyncManager::new(credentials)),
        })
    }

    /// Sync accounts from config to database
    async fn sync_accounts_from_config(
        db: &EmailDatabase,
        config: &Config,
        mut db_accounts: Vec<DbAccount>,
    ) -> anyhow::Result<Vec<DbAccount>> {
        // Add accounts from config that don't exist in database
        for (_, config_account) in &config.accounts {
            let exists = db_accounts.iter().any(|a| a.email == config_account.email);
            if !exists {
                let db_account = DbAccount {
                    id: 0,
                    name: config_account.name.clone(),
                    email: config_account.email.clone(),
                    provider: config_account.provider.clone(),
                    is_default: config_account.default,
                    color: config_account.color.clone(),
                    display_order: config_account.display_order.unwrap_or(999),
                };
                let id = db.save_account(&db_account).await?;
                db_accounts.push(DbAccount {
                    id,
                    ..db_account
                });
            }
        }

        Ok(db_accounts)
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
            // Update visual selection if in visual mode
            if self.visual_mode {
                self.update_visual_selection();
            }
        }
    }

    pub fn previous_email(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            // Update visual selection if in visual mode
            if self.visual_mode {
                self.update_visual_selection();
            }
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
                    let email_subject = email.subject.clone();
                    
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
                    
                    // Remove email from the vector
                    self.emails.remove(self.selected_index);
                    
                    // Adjust selected_index if needed
                    if !self.emails.is_empty() {
                        self.selected_index = self.selected_index.min(self.emails.len() - 1);
                    } else {
                        self.selected_index = 0;
                    }
                    
                    self.status_message = Some(format!("Deleted email: {}", email_subject));
                }
            }
            Action::Archive => {
                if !self.emails.is_empty() {
                    let email = &self.emails[self.selected_index];
                    let email_id = email.id;
                    let email_subject = email.subject.clone();
                    
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
                    
                    // Remove email from the vector
                    self.emails.remove(self.selected_index);
                    
                    // Adjust selected_index if needed
                    if !self.emails.is_empty() {
                        self.selected_index = self.selected_index.min(self.emails.len() - 1);
                    } else {
                        self.selected_index = 0;
                    }
                    
                    self.status_message = Some(format!("Archived email: {}", email_subject));
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
                })
                .join();

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
            if !compose.recipients.is_empty()
                || !compose.subject.is_empty()
                || !compose.body.is_empty()
            {
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

    /// Save draft before quitting the application (async version)
    pub async fn save_draft_before_quit_async(&self) -> anyhow::Result<()> {
        if let Some(ref compose) = self.compose_state {
            if !compose.recipients.is_empty()
                || !compose.subject.is_empty()
                || !compose.body.is_empty()
            {
                if let Some(ref db) = self.db {
                    let draft = self.create_db_draft(compose);
                    db.save_draft(&draft).await?;
                }
            }
        }
        Ok(())
    }

    /// Check if there's a draft that needs saving
    pub fn has_unsaved_draft(&self) -> bool {
        if let Some(ref compose) = self.compose_state {
            !compose.recipients.is_empty()
                || !compose.subject.is_empty()
                || !compose.body.is_empty()
        } else {
            false
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
            account_id: self.current_account_id,
        }
    }

    // Stub methods for GPG and Yubikey hooks
    pub fn compose_encrypt_with_gpg(&mut self) {
        self.status_message = Some("GPG encryption hook (stub)".to_string());
    }

    pub fn compose_sign_with_yubikey(&mut self) {
        self.status_message = Some("Yubikey signing hook (stub)".to_string());
    }

    pub fn toggle_preview_panel(&mut self) {
        self.show_preview_panel = !self.show_preview_panel;
    }

    // Visual mode methods
    pub fn enter_visual_mode(&mut self) {
        if self.current_view == View::InboxList && !self.visual_mode {
            self.visual_mode = true;
            self.visual_anchor = Some(self.selected_index);
            self.visual_selections.clear();
            self.visual_selections.insert(self.selected_index);
            self.status_message = Some("-- VISUAL LINE --".to_string());
        }
    }

    pub fn exit_visual_mode(&mut self) {
        self.visual_mode = false;
        self.visual_selections.clear();
        self.visual_anchor = None;
        // Don't clear status message here - it may contain action results
    }

    pub fn update_visual_selection(&mut self) {
        if let Some(anchor) = self.visual_anchor {
            self.visual_selections.clear();
            let start = anchor.min(self.selected_index);
            let end = anchor.max(self.selected_index);
            for i in start..=end {
                self.visual_selections.insert(i);
            }
        }
    }

    pub fn perform_batch_action(&mut self, action: Action) {
        if !self.visual_mode || self.visual_selections.is_empty() {
            return;
        }

        let count = self.visual_selections.len();
        let action_name = match action {
            Action::Delete => "Deleted",
            Action::Archive => "Archived",
            _ => return, // Only delete and archive are supported for batch
        };

        // Get the email IDs and indices to operate on (sorted in reverse to safely remove)
        let mut indices: Vec<usize> = self.visual_selections.iter().copied().collect();
        indices.sort_by(|a, b| b.cmp(a)); // Sort descending

        // Collect email IDs and perform database operations
        let mut email_ids = Vec::new();
        for &index in &indices {
            if let Some(email) = self.emails.get(index) {
                email_ids.push(email.id);
            }
        }

        // Perform database operations
        if let Some(ref db) = self.db {
            let db_clone = db.clone();
            match action {
                Action::Delete => {
                    tokio::spawn(async move {
                        for email_id in email_ids {
                            if let Err(e) = db_clone.delete_email(email_id).await {
                                eprintln!("Failed to delete email from database: {}", e);
                            }
                        }
                    });
                }
                Action::Archive => {
                    tokio::spawn(async move {
                        for email_id in email_ids {
                            if let Err(e) = db_clone.archive_email(email_id).await {
                                eprintln!("Failed to archive email in database: {}", e);
                            }
                        }
                    });
                }
                _ => {}
            }
        }

        // Remove emails from the vector (in reverse order to maintain valid indices)
        for &index in &indices {
            if index < self.emails.len() {
                self.emails.remove(index);
            }
        }

        self.status_message = Some(format!("{} {} emails", action_name, count));
        
        // Adjust selected_index if needed
        if !self.emails.is_empty() {
            self.selected_index = self.selected_index.min(self.emails.len() - 1);
        } else {
            self.selected_index = 0;
        }
        
        // Exit visual mode after performing action
        self.exit_visual_mode();
    }

    pub fn is_email_selected(&self, index: usize) -> bool {
        self.visual_selections.contains(&index)
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Attempt to sync emails (stub - shows not implemented message)
    pub fn attempt_email_sync(&mut self) {
        if let Some(ref sync_manager) = self.email_sync_manager {
            if sync_manager.is_configured() {
                self.status_message = Some(
                    "Email sync not yet implemented. IMAP/SMTP integration coming soon. \
                    Currently displaying mock data. See project issues for implementation status.".to_string()
                );
            } else {
                self.status_message = Some(
                    "No credentials configured. Please set up email credentials first.".to_string()
                );
            }
        } else {
            self.status_message = Some(
                "Email sync not available. Please restart the app after configuring credentials.".to_string()
            );
        }
    }

    pub fn get_selected_email(&self) -> Option<&Email> {
        self.emails.get(self.selected_index)
    }

    // ============ Credentials Management Methods ============

    /// Navigate to next provider in selection list
    pub fn credentials_setup_next_provider(&mut self) {
        if let Some(ref mut setup) = self.credentials_setup_state {
            if setup.provider_selection_mode {
                let providers = crate::providers::EmailProvider::all();
                setup.provider_list_index = (setup.provider_list_index + 1) % providers.len();
            }
        }
    }

    /// Navigate to previous provider in selection list
    pub fn credentials_setup_prev_provider(&mut self) {
        if let Some(ref mut setup) = self.credentials_setup_state {
            if setup.provider_selection_mode {
                let providers = crate::providers::EmailProvider::all();
                setup.provider_list_index = if setup.provider_list_index == 0 {
                    providers.len() - 1
                } else {
                    setup.provider_list_index - 1
                };
            }
        }
    }

    /// Select the currently highlighted provider and move to field entry
    pub fn credentials_setup_select_provider(&mut self) {
        if let Some(ref mut setup) = self.credentials_setup_state {
            if setup.provider_selection_mode {
                let providers = crate::providers::EmailProvider::all();
                if let Some(provider) = providers.get(setup.provider_list_index) {
                    setup.apply_provider(provider);
                }
            }
        }
    }

    /// Go back to provider selection from field entry
    pub fn credentials_setup_back_to_providers(&mut self) {
        if let Some(ref mut setup) = self.credentials_setup_state {
            if !setup.provider_selection_mode {
                setup.provider_selection_mode = true;
                setup.selected_provider = None;
            }
        }
    }

    /// Enter insert mode for editing credentials fields
    pub fn credentials_setup_enter_insert_mode(&mut self) {
        if let Some(ref mut setup) = self.credentials_setup_state {
            if !setup.provider_selection_mode {
                setup.mode = CredentialsMode::Insert;
            }
        }
    }

    /// Exit insert mode and return to normal mode
    pub fn credentials_setup_exit_insert_mode(&mut self) {
        if let Some(ref mut setup) = self.credentials_setup_state {
            setup.mode = CredentialsMode::Normal;
        }
    }

    /// Navigate to next field in credentials setup
    pub fn credentials_setup_next_field(&mut self) {
        if let Some(ref mut setup) = self.credentials_setup_state {
            // Determine if we should go to master password field
            let use_encrypted_file = self.credentials_manager
                .as_ref()
                .map(|m| m.backend() == StorageBackend::EncryptedFile)
                .unwrap_or(false);

            setup.current_field = match setup.current_field {
                CredentialField::ImapServer => CredentialField::ImapPort,
                CredentialField::ImapPort => CredentialField::ImapUsername,
                CredentialField::ImapUsername => CredentialField::ImapPassword,
                CredentialField::ImapPassword => CredentialField::SmtpServer,
                CredentialField::SmtpServer => CredentialField::SmtpPort,
                CredentialField::SmtpPort => CredentialField::SmtpUsername,
                CredentialField::SmtpUsername => CredentialField::SmtpPassword,
                CredentialField::SmtpPassword => {
                    if use_encrypted_file {
                        CredentialField::MasterPassword
                    } else {
                        CredentialField::ImapServer
                    }
                }
                CredentialField::MasterPassword => CredentialField::MasterPasswordConfirm,
                CredentialField::MasterPasswordConfirm => CredentialField::ImapServer,
            };
            
            // Update cursor position to end of new field
            setup.cursor_position = match setup.current_field {
                CredentialField::ImapServer => setup.imap_server.len(),
                CredentialField::ImapPort => setup.imap_port.len(),
                CredentialField::ImapUsername => setup.imap_username.len(),
                CredentialField::ImapPassword => setup.imap_password.len(),
                CredentialField::SmtpServer => setup.smtp_server.len(),
                CredentialField::SmtpPort => setup.smtp_port.len(),
                CredentialField::SmtpUsername => setup.smtp_username.len(),
                CredentialField::SmtpPassword => setup.smtp_password.len(),
                CredentialField::MasterPassword => setup.master_password.len(),
                CredentialField::MasterPasswordConfirm => setup.master_password_confirm.len(),
            };
        }
    }

    /// Navigate to previous field in credentials setup
    pub fn credentials_setup_prev_field(&mut self) {
        if let Some(ref mut setup) = self.credentials_setup_state {
            // Determine if we should go to master password field
            let use_encrypted_file = self.credentials_manager
                .as_ref()
                .map(|m| m.backend() == StorageBackend::EncryptedFile)
                .unwrap_or(false);

            setup.current_field = match setup.current_field {
                CredentialField::ImapServer => {
                    if use_encrypted_file {
                        CredentialField::MasterPasswordConfirm
                    } else {
                        CredentialField::SmtpPassword
                    }
                }
                CredentialField::ImapPort => CredentialField::ImapServer,
                CredentialField::ImapUsername => CredentialField::ImapPort,
                CredentialField::ImapPassword => CredentialField::ImapUsername,
                CredentialField::SmtpServer => CredentialField::ImapPassword,
                CredentialField::SmtpPort => CredentialField::SmtpServer,
                CredentialField::SmtpUsername => CredentialField::SmtpPort,
                CredentialField::SmtpPassword => CredentialField::SmtpUsername,
                CredentialField::MasterPassword => CredentialField::SmtpPassword,
                CredentialField::MasterPasswordConfirm => CredentialField::MasterPassword,
            };
            
            // Update cursor position to end of new field
            setup.cursor_position = match setup.current_field {
                CredentialField::ImapServer => setup.imap_server.len(),
                CredentialField::ImapPort => setup.imap_port.len(),
                CredentialField::ImapUsername => setup.imap_username.len(),
                CredentialField::ImapPassword => setup.imap_password.len(),
                CredentialField::SmtpServer => setup.smtp_server.len(),
                CredentialField::SmtpPort => setup.smtp_port.len(),
                CredentialField::SmtpUsername => setup.smtp_username.len(),
                CredentialField::SmtpPassword => setup.smtp_password.len(),
                CredentialField::MasterPassword => setup.master_password.len(),
                CredentialField::MasterPasswordConfirm => setup.master_password_confirm.len(),
            };
        }
    }

    /// Insert character into current credentials setup field
    pub fn credentials_setup_insert_char(&mut self, c: char) {
        if let Some(ref mut setup) = self.credentials_setup_state {
            let text = match setup.current_field {
                CredentialField::ImapServer => &mut setup.imap_server,
                CredentialField::ImapPort => &mut setup.imap_port,
                CredentialField::ImapUsername => &mut setup.imap_username,
                CredentialField::ImapPassword => &mut setup.imap_password,
                CredentialField::SmtpServer => &mut setup.smtp_server,
                CredentialField::SmtpPort => &mut setup.smtp_port,
                CredentialField::SmtpUsername => &mut setup.smtp_username,
                CredentialField::SmtpPassword => &mut setup.smtp_password,
                CredentialField::MasterPassword => &mut setup.master_password,
                CredentialField::MasterPasswordConfirm => &mut setup.master_password_confirm,
            };

            if setup.cursor_position <= text.len() {
                text.insert(setup.cursor_position, c);
                setup.cursor_position += 1;
            }
        }
    }

    /// Delete character from current credentials setup field
    pub fn credentials_setup_delete_char(&mut self) {
        if let Some(ref mut setup) = self.credentials_setup_state {
            if setup.cursor_position > 0 {
                let text = match setup.current_field {
                    CredentialField::ImapServer => &mut setup.imap_server,
                    CredentialField::ImapPort => &mut setup.imap_port,
                    CredentialField::ImapUsername => &mut setup.imap_username,
                    CredentialField::ImapPassword => &mut setup.imap_password,
                    CredentialField::SmtpServer => &mut setup.smtp_server,
                    CredentialField::SmtpPort => &mut setup.smtp_port,
                    CredentialField::SmtpUsername => &mut setup.smtp_username,
                    CredentialField::SmtpPassword => &mut setup.smtp_password,
                    CredentialField::MasterPassword => &mut setup.master_password,
                    CredentialField::MasterPasswordConfirm => &mut setup.master_password_confirm,
                };

                setup.cursor_position -= 1;
                text.remove(setup.cursor_position);
            }
        }
    }

    /// Move cursor left in credentials setup
    pub fn credentials_setup_cursor_left(&mut self) {
        if let Some(ref mut setup) = self.credentials_setup_state {
            if setup.cursor_position > 0 {
                setup.cursor_position -= 1;
            }
        }
    }

    /// Move cursor right in credentials setup
    pub fn credentials_setup_cursor_right(&mut self) {
        if let Some(ref mut setup) = self.credentials_setup_state {
            let max_pos = match setup.current_field {
                CredentialField::ImapServer => setup.imap_server.len(),
                CredentialField::ImapPort => setup.imap_port.len(),
                CredentialField::ImapUsername => setup.imap_username.len(),
                CredentialField::ImapPassword => setup.imap_password.len(),
                CredentialField::SmtpServer => setup.smtp_server.len(),
                CredentialField::SmtpPort => setup.smtp_port.len(),
                CredentialField::SmtpUsername => setup.smtp_username.len(),
                CredentialField::SmtpPassword => setup.smtp_password.len(),
                CredentialField::MasterPassword => setup.master_password.len(),
                CredentialField::MasterPasswordConfirm => setup.master_password_confirm.len(),
            };
            if setup.cursor_position < max_pos {
                setup.cursor_position += 1;
            }
        }
    }

    /// Toggle password visibility in credentials setup
    pub fn credentials_setup_toggle_password_visibility(&mut self) {
        if let Some(ref mut setup) = self.credentials_setup_state {
            setup.show_passwords = !setup.show_passwords;
        }
    }

    /// Save credentials from setup form
    pub fn credentials_setup_save(&mut self) {
        let setup = match &self.credentials_setup_state {
            Some(s) => s.clone(),
            None => return,
        };

        let manager = match &self.credentials_manager {
            Some(m) => m,
            None => return,
        };

        // Validate fields
        if setup.imap_server.is_empty() || setup.imap_username.is_empty() 
            || setup.smtp_server.is_empty() || setup.smtp_username.is_empty() {
            self.status_message = Some("Please fill in all required fields".to_string());
            return;
        }

        // Parse ports
        let imap_port = match setup.imap_port.parse::<u16>() {
            Ok(p) => p,
            Err(_) => {
                self.status_message = Some("Invalid IMAP port number".to_string());
                return;
            }
        };

        let smtp_port = match setup.smtp_port.parse::<u16>() {
            Ok(p) => p,
            Err(_) => {
                self.status_message = Some("Invalid SMTP port number".to_string());
                return;
            }
        };

        // For encrypted file backend, validate master password
        let master_password = if manager.backend() == StorageBackend::EncryptedFile {
            if setup.master_password.is_empty() {
                self.status_message = Some("Master password is required".to_string());
                return;
            }
            if setup.master_password != setup.master_password_confirm {
                self.status_message = Some("Master passwords do not match".to_string());
                return;
            }
            if setup.master_password.len() < 8 {
                self.status_message = Some("Master password must be at least 8 characters".to_string());
                return;
            }
            Some(setup.master_password.as_str())
        } else {
            None
        };

        // Create credentials object
        let credentials = Credentials {
            imap_server: setup.imap_server.clone(),
            imap_port,
            imap_username: setup.imap_username.clone(),
            imap_password: setup.imap_password.clone(),
            smtp_server: setup.smtp_server.clone(),
            smtp_port,
            smtp_username: setup.smtp_username.clone(),
            smtp_password: setup.smtp_password.clone(),
        };

        // Save credentials
        match manager.save_credentials(&credentials, master_password) {
            Ok(_) => {
                self.credentials = Some(credentials.clone());
                
                // Save account configuration to config file
                // Use selected provider or fallback to "custom"
                let provider_id = setup.selected_provider.as_ref()
                    .map(|s| s.clone())
                    .unwrap_or_else(|| "custom".to_string());
                    
                let provider_name = crate::providers::EmailProvider::by_id(&provider_id)
                    .map(|p| p.name)
                    .unwrap_or("Custom");
                
                // Create account entry
                let account = crate::config::Account {
                    name: format!("{} Account", provider_name),
                    email: setup.imap_username.clone(),
                    provider: provider_id.clone(),
                    default: true, // First account is default
                    color: Some("blue".to_string()),
                    display_order: Some(1),
                };
                
                // Add to config and save
                let account_key = provider_id.replace(" ", "_").to_lowercase();
                self.config.accounts.insert(account_key, account.clone());
                
                // Try to save config - if it fails, still continue but show error
                let config_saved = match self.config.save() {
                    Ok(_) => {
                        eprintln!("DEBUG: Config saved successfully to {:?}", crate::config::Config::config_path());
                        true
                    },
                    Err(e) => {
                        eprintln!("ERROR: Failed to save config file: {}", e);
                        self.status_message = Some(format!("ERROR: Failed to save config file: {}. Account will be lost on restart!", e));
                        false
                    }
                };
                
                // Always try to add to in-memory accounts list
                let db_account = crate::db::DbAccount {
                    id: 0,
                    name: account.name.clone(),
                    email: account.email.clone(),
                    provider: account.provider.clone(),
                    is_default: account.default,
                    color: account.color.clone(),
                    display_order: account.display_order.unwrap_or(999),
                };
                self.accounts.push(db_account.clone());
                self.current_account_id = Some(db_account.id);
                
                if config_saved {
                    self.status_message = Some(format!(
                        "Credentials and account configuration saved successfully using {}. Email sync not yet implemented - using mock data.",
                        manager.backend().as_str()
                    ));
                } else {
                    // Error message already set above
                }
                
                self.credentials_setup_state = None;
                self.current_view = View::InboxList;
                
                // Initialize email sync manager with credentials
                self.email_sync_manager = Some(crate::email_sync::EmailSyncManager::new(Some(credentials)));
            }
            Err(e) => {
                self.status_message = Some(format!("Failed to save credentials: {}", e));
            }
        }
    }

    /// Cancel credentials setup
    pub fn credentials_setup_cancel(&mut self) {
        // If credentials don't exist yet, quit the app
        if let Some(ref manager) = self.credentials_manager {
            if !manager.credentials_exist() {
                self.should_quit = true;
                return;
            }
        }

        // Otherwise, clear setup state and return to inbox
        self.credentials_setup_state = None;
        self.current_view = View::InboxList;
    }

    /// Insert character into unlock password field
    pub fn credentials_unlock_insert_char(&mut self, c: char) {
        if let Some(ref mut unlock) = self.credentials_unlock_state {
            if unlock.cursor_position <= unlock.master_password.len() {
                unlock.master_password.insert(unlock.cursor_position, c);
                unlock.cursor_position += 1;
            }
        }
    }

    /// Delete character from unlock password field
    pub fn credentials_unlock_delete_char(&mut self) {
        if let Some(ref mut unlock) = self.credentials_unlock_state {
            if unlock.cursor_position > 0 {
                unlock.cursor_position -= 1;
                unlock.master_password.remove(unlock.cursor_position);
            }
        }
    }

    /// Move cursor left in unlock password field
    pub fn credentials_unlock_cursor_left(&mut self) {
        if let Some(ref mut unlock) = self.credentials_unlock_state {
            if unlock.cursor_position > 0 {
                unlock.cursor_position -= 1;
            }
        }
    }

    /// Move cursor right in unlock password field
    pub fn credentials_unlock_cursor_right(&mut self) {
        if let Some(ref mut unlock) = self.credentials_unlock_state {
            if unlock.cursor_position < unlock.master_password.len() {
                unlock.cursor_position += 1;
            }
        }
    }

    /// Attempt to unlock credentials with provided password
    pub fn credentials_unlock_submit(&mut self) {
        let password = match &self.credentials_unlock_state {
            Some(state) => state.master_password.clone(),
            None => return,
        };

        let manager = match &self.credentials_manager {
            Some(m) => m,
            None => return,
        };

        // Attempt to load credentials
        match manager.load_credentials(Some(&password)) {
            Ok(credentials) => {
                self.credentials = Some(credentials);
                self.credentials_unlock_state = None;
                self.current_view = View::InboxList;
                self.status_message = Some("Credentials unlocked successfully".to_string());
            }
            Err(e) => {
                if let Some(ref mut unlock) = self.credentials_unlock_state {
                    unlock.error_message = Some(format!("Failed to unlock: {}", e));
                    unlock.master_password.clear();
                    unlock.cursor_position = 0;
                }
            }
        }
    }

    /// Cancel credential unlock (quit app)
    pub fn credentials_unlock_cancel(&mut self) {
        self.should_quit = true;
    }

    /// Enter credentials management view
    pub fn enter_credentials_management(&mut self) {
        self.current_view = View::CredentialsManagement;
    }

    /// Exit credentials management view
    pub fn exit_credentials_management(&mut self) {
        self.current_view = View::InboxList;
    }

    /// Reset credentials (delete and return to setup)
    pub fn credentials_reset(&mut self) {
        if let Some(ref manager) = self.credentials_manager {
            match manager.delete_credentials() {
                Ok(_) => {
                    self.credentials = None;
                    self.credentials_setup_state = Some(CredentialsSetupState::new(manager.backend()));
                    self.current_view = View::CredentialsSetup;
                    self.status_message = Some("Credentials reset. Please set up new credentials.".to_string());
                }
                Err(e) => {
                    self.status_message = Some(format!("Failed to reset credentials: {}", e));
                }
            }
        }
    }

    /// Get backend info for display
    pub fn get_backend_info(&self) -> Option<(StorageBackend, String)> {
        self.credentials_manager.as_ref().map(|m| {
            let backend = m.backend();
            let description = backend.description().to_string();
            (backend, description)
        })
    }
    /// Get the current account name for display
    pub fn get_current_account_name(&self) -> Option<String> {
        self.current_account_id.and_then(|id| {
            self.accounts
                .iter()
                .find(|a| a.id == id)
                .map(|a| a.name.clone())
        })
    }

    /// Switch to a specific account by index (0-based)
    pub fn switch_to_account(&mut self, index: usize) {
        if index < self.accounts.len() {
            let account_id = self.accounts[index].id;
            let account_name = self.accounts[index].name.clone();
            self.current_account_id = Some(account_id);
            self.reload_emails_for_current_account();
            self.status_message = Some(format!("Switched to account: {}", account_name));
        }
    }

    /// Switch to next account
    pub fn next_account(&mut self) {
        if self.accounts.is_empty() {
            return;
        }

        let current_idx = self.current_account_id.and_then(|id| {
            self.accounts.iter().position(|a| a.id == id)
        });

        let next_idx = match current_idx {
            Some(idx) => (idx + 1) % self.accounts.len(),
            None => 0,
        };

        self.switch_to_account(next_idx);
    }

    /// Switch to previous account
    pub fn prev_account(&mut self) {
        if self.accounts.is_empty() {
            return;
        }

        let current_idx = self.current_account_id.and_then(|id| {
            self.accounts.iter().position(|a| a.id == id)
        });

        let prev_idx = match current_idx {
            Some(0) => self.accounts.len() - 1,
            Some(idx) => idx - 1,
            None => 0,
        };

        self.switch_to_account(prev_idx);
    }

    /// Reload emails for the current account
    fn reload_emails_for_current_account(&mut self) {
        if let Some(ref db) = self.db {
            let db_clone = db.clone();
            let account_id = self.current_account_id;
            
            // Use spawn_blocking to avoid nested runtime issues
            let runtime = tokio::runtime::Handle::try_current();
            if let Ok(handle) = runtime {
                let emails_result = std::thread::spawn(move || {
                    handle.block_on(async {
                        if let Some(acc_id) = account_id {
                            db_clone.get_emails_by_folder_and_account("inbox", Some(acc_id)).await
                        } else {
                            db_clone.get_emails_by_folder("inbox").await
                        }
                    })
                })
                .join();

                if let Ok(Ok(db_emails)) = emails_result {
                    self.emails = db_emails
                        .into_iter()
                        .map(|e| Email {
                            id: e.id,
                            from: e.from_address,
                            subject: e.subject,
                            preview: e.preview,
                            body: e.body,
                            date: e.date,
                        })
                        .collect();
                    self.selected_index = 0;
                }
            }
        }
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
        let initial_count = app.emails.len();

        app.perform_action(Action::Delete);
        assert!(app.status_message.is_some());
        assert!(app.status_message.as_ref().unwrap().contains("Deleted"));
        // Delete should remove the email from the list
        assert_eq!(app.emails.len(), initial_count - 1);

        app.perform_action(Action::Archive);
        assert!(app.status_message.as_ref().unwrap().contains("Archived"));
        // Archive should also remove the email from the list
        assert_eq!(app.emails.len(), initial_count - 2);

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
        assert_eq!(
            app.compose_state.as_ref().unwrap().current_field,
            ComposeField::Subject
        );

        app.compose_next_field();
        assert_eq!(
            app.compose_state.as_ref().unwrap().current_field,
            ComposeField::Body
        );

        app.compose_next_field();
        assert_eq!(
            app.compose_state.as_ref().unwrap().current_field,
            ComposeField::Recipients
        );

        app.compose_previous_field();
        assert_eq!(
            app.compose_state.as_ref().unwrap().current_field,
            ComposeField::Body
        );
    }

    #[test]
    fn test_compose_insert_mode() {
        let mut app = App::new();
        app.enter_compose_mode();

        assert_eq!(
            app.compose_state.as_ref().unwrap().mode,
            ComposeMode::Normal
        );

        app.compose_enter_insert_mode();
        assert_eq!(
            app.compose_state.as_ref().unwrap().mode,
            ComposeMode::Insert
        );

        app.compose_exit_insert_mode();
        assert_eq!(
            app.compose_state.as_ref().unwrap().mode,
            ComposeMode::Normal
        );
        // Should auto-advance to Subject during initial traversal
        assert_eq!(
            app.compose_state.as_ref().unwrap().current_field,
            ComposeField::Subject
        );

        // Continue to Body
        app.compose_enter_insert_mode();
        app.compose_exit_insert_mode();
        assert_eq!(
            app.compose_state.as_ref().unwrap().current_field,
            ComposeField::Body
        );

        // Now that we're on Body, traversal is complete - Esc should stay on Body
        app.compose_enter_insert_mode();
        app.compose_exit_insert_mode();
        assert_eq!(
            app.compose_state.as_ref().unwrap().current_field,
            ComposeField::Body
        );
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
    fn test_preview_panel_toggle() {
        let mut app = App::new();
        assert_eq!(app.show_preview_panel, false);

        app.toggle_preview_panel();
        assert_eq!(app.show_preview_panel, true);

        app.toggle_preview_panel();
        assert_eq!(app.show_preview_panel, false);
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
        assert_eq!(
            app.compose_state.as_ref().unwrap().current_field,
            ComposeField::Subject
        );
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
        assert_eq!(
            app.compose_state.as_ref().unwrap().current_field,
            ComposeField::Body
        );
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

        let db = crate::db::EmailDatabase::new(Some(path.clone()))
            .await
            .unwrap();
        let mut app = App {
            emails: Vec::new(),
            current_view: View::InboxList,
            selected_index: 0,
            should_quit: false,
            status_message: None,
            compose_state: None,
            db: Some(db),
            draft_id: None,
            show_preview_panel: false,
            visual_mode: false,
            visual_selections: HashSet::new(),
            visual_anchor: None,
            credentials_manager: None,
            credentials: None,
            credentials_setup_state: None,
            credentials_unlock_state: None,
            config: Config::default(),
            accounts: Vec::new(),
            current_account_id: None,
            email_sync_manager: None,
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

        let db = crate::db::EmailDatabase::new(Some(path.clone()))
            .await
            .unwrap();
        let mut app = App {
            emails: Vec::new(),
            current_view: View::InboxList,
            selected_index: 0,
            should_quit: false,
            status_message: None,
            compose_state: None,
            db: Some(db),
            draft_id: None,
            show_preview_panel: false,
            visual_mode: false,
            visual_selections: HashSet::new(),
            visual_anchor: None,
            credentials_manager: None,
            credentials: None,
            credentials_setup_state: None,
            credentials_unlock_state: None,
            config: Config::default(),
            accounts: Vec::new(),
            current_account_id: None,
            email_sync_manager: None,
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

    #[test]
    fn test_visual_mode_enter_exit() {
        let mut app = App::new();
        assert_eq!(app.visual_mode, false);
        assert_eq!(app.visual_selections.len(), 0);

        // Enter visual mode
        app.enter_visual_mode();
        assert_eq!(app.visual_mode, true);
        assert_eq!(app.visual_selections.len(), 1);
        assert!(app.visual_selections.contains(&0));
        assert_eq!(app.visual_anchor, Some(0));

        // Exit visual mode
        app.exit_visual_mode();
        assert_eq!(app.visual_mode, false);
        assert_eq!(app.visual_selections.len(), 0);
        assert_eq!(app.visual_anchor, None);
    }

    #[test]
    fn test_visual_mode_selection_extension() {
        let mut app = App::new();
        
        // Enter visual mode at index 0
        app.enter_visual_mode();
        assert_eq!(app.visual_selections.len(), 1);
        assert!(app.visual_selections.contains(&0));

        // Move down (extends selection)
        app.next_email();
        assert_eq!(app.selected_index, 1);
        assert_eq!(app.visual_selections.len(), 2);
        assert!(app.visual_selections.contains(&0));
        assert!(app.visual_selections.contains(&1));

        // Move down again
        app.next_email();
        assert_eq!(app.selected_index, 2);
        assert_eq!(app.visual_selections.len(), 3);
        assert!(app.visual_selections.contains(&0));
        assert!(app.visual_selections.contains(&1));
        assert!(app.visual_selections.contains(&2));

        // Move back up (shrinks selection)
        app.previous_email();
        assert_eq!(app.selected_index, 1);
        assert_eq!(app.visual_selections.len(), 2);
        assert!(app.visual_selections.contains(&0));
        assert!(app.visual_selections.contains(&1));
        assert!(!app.visual_selections.contains(&2));
    }

    #[test]
    fn test_visual_mode_batch_delete() {
        let mut app = App::new();
        let initial_count = app.emails.len();
        
        // Enter visual mode and select multiple emails
        app.enter_visual_mode();
        app.next_email();
        app.next_email();
        assert_eq!(app.visual_selections.len(), 3);

        // Perform batch delete
        app.perform_batch_action(Action::Delete);
        
        // Visual mode should be exited
        assert_eq!(app.visual_mode, false);
        assert_eq!(app.visual_selections.len(), 0);
        
        // Emails should be removed from the list
        assert_eq!(app.emails.len(), initial_count - 3);
        
        // Status message should be set
        assert!(app.status_message.is_some());
        assert!(app.status_message.as_ref().unwrap().contains("Deleted"));
        assert!(app.status_message.as_ref().unwrap().contains("3"));
    }

    #[test]
    fn test_visual_mode_batch_archive() {
        let mut app = App::new();
        let initial_count = app.emails.len();
        
        // Enter visual mode and select multiple emails
        app.enter_visual_mode();
        app.next_email();
        assert_eq!(app.visual_selections.len(), 2);

        // Perform batch archive
        app.perform_batch_action(Action::Archive);
        
        // Visual mode should be exited
        assert_eq!(app.visual_mode, false);
        assert_eq!(app.visual_selections.len(), 0);
        
        // Emails should be removed from the list
        assert_eq!(app.emails.len(), initial_count - 2);
        
        // Status message should be set
        assert!(app.status_message.is_some());
        assert!(app.status_message.as_ref().unwrap().contains("Archived"));
        assert!(app.status_message.as_ref().unwrap().contains("2"));
    }

    #[test]
    fn test_single_delete_action() {
        let mut app = App::new();
        let initial_count = app.emails.len();
        
        // Select the first email
        assert_eq!(app.selected_index, 0);
        let first_email_subject = app.emails[0].subject.clone();
        
        // Perform single delete
        app.perform_action(Action::Delete);
        
        // Verify email was removed
        assert_eq!(app.emails.len(), initial_count - 1);
        
        // Verify the correct email was removed
        assert_ne!(app.emails[0].subject, first_email_subject);
        
        // Verify selected_index is still valid
        assert_eq!(app.selected_index, 0);
        
        // Verify status message
        assert!(app.status_message.is_some());
        assert!(app.status_message.as_ref().unwrap().contains("Deleted"));
        assert!(app.status_message.as_ref().unwrap().contains(&first_email_subject));
    }

    #[test]
    fn test_single_delete_last_email() {
        let mut app = App::new();
        let initial_count = app.emails.len();
        
        // Move to the last email
        app.selected_index = initial_count - 1;
        
        // Perform delete
        app.perform_action(Action::Delete);
        
        // Verify email was removed
        assert_eq!(app.emails.len(), initial_count - 1);
        
        // Verify selected_index was adjusted
        assert_eq!(app.selected_index, initial_count - 2);
    }

    #[test]
    fn test_single_archive_action() {
        let mut app = App::new();
        let initial_count = app.emails.len();
        
        // Select the second email
        app.selected_index = 1;
        let email_subject = app.emails[1].subject.clone();
        
        // Perform single archive
        app.perform_action(Action::Archive);
        
        // Verify email was removed from inbox
        assert_eq!(app.emails.len(), initial_count - 1);
        
        // Verify selected_index is still valid
        assert_eq!(app.selected_index, 1);
        
        // Verify status message
        assert!(app.status_message.is_some());
        assert!(app.status_message.as_ref().unwrap().contains("Archived"));
        assert!(app.status_message.as_ref().unwrap().contains(&email_subject));
    }

    #[test]
    fn test_is_email_selected() {
        let mut app = App::new();
        
        // Initially nothing is selected
        assert!(!app.is_email_selected(0));
        assert!(!app.is_email_selected(1));

        // Enter visual mode
        app.enter_visual_mode();
        assert!(app.is_email_selected(0));
        assert!(!app.is_email_selected(1));

        // Extend selection
        app.next_email();
        assert!(app.is_email_selected(0));
        assert!(app.is_email_selected(1));
        assert!(!app.is_email_selected(2));
    }

    #[test]
    fn test_visual_mode_only_in_inbox() {
        let mut app = App::new();
        
        // Switch to detail view
        app.open_email();
        assert_eq!(app.current_view, View::EmailDetail);
        
        // Try to enter visual mode (should not work)
        app.enter_visual_mode();
        assert_eq!(app.visual_mode, false);
        
        // Go back to inbox
        app.close_email();
        assert_eq!(app.current_view, View::InboxList);
        
        // Now it should work
        app.enter_visual_mode();
        assert_eq!(app.visual_mode, true);
    }
}

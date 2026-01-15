/// Email synchronization module
/// 
/// This module provides:
/// - Inbox rules for automatic email filtering and organization
/// - IMAP email fetching with TLS support
/// - SMTP email sending with TLS support
/// 
/// ## Implementation Status:
/// - ✅ Inbox rules engine (fully implemented)
/// - ✅ IMAP email fetching (working implementation)
/// - ✅ SMTP email sending (working implementation)
/// - ⏳ Folder management (requires IMAP integration)
/// - ⏳ OAuth2 support (not started - needed for Gmail/Outlook)

use crate::credentials::Credentials;
use crate::db::{DbEmail, EmailStatus as DbEmailStatus};
use anyhow::{Result, anyhow, Context};

/// Inbox rule for automatic filtering and organization
#[derive(Debug, Clone)]
pub struct InboxRule {
    pub id: i64,
    pub name: String,
    pub condition: RuleCondition,
    pub action: RuleAction,
    pub enabled: bool,
}

/// Condition for inbox rules
#[derive(Debug, Clone)]
pub enum RuleCondition {
    FromContains(String),
    SubjectContains(String),
    BodyContains(String),
    FromEquals(String),
    And(Box<RuleCondition>, Box<RuleCondition>),
    Or(Box<RuleCondition>, Box<RuleCondition>),
}

impl RuleCondition {
    /// Check if an email matches this condition
    pub fn matches(&self, email: &DbEmail) -> bool {
        match self {
            RuleCondition::FromContains(pattern) => {
                email.from_address.to_lowercase().contains(&pattern.to_lowercase())
            }
            RuleCondition::SubjectContains(pattern) => {
                email.subject.to_lowercase().contains(&pattern.to_lowercase())
            }
            RuleCondition::BodyContains(pattern) => {
                email.body.to_lowercase().contains(&pattern.to_lowercase())
            }
            RuleCondition::FromEquals(addr) => {
                email.from_address.to_lowercase() == addr.to_lowercase()
            }
            RuleCondition::And(left, right) => {
                left.matches(email) && right.matches(email)
            }
            RuleCondition::Or(left, right) => {
                left.matches(email) || right.matches(email)
            }
        }
    }
}

/// Action to take when rule matches
#[derive(Debug, Clone)]
pub enum RuleAction {
    MoveToFolder(String),
    MarkAsRead,
    Flag,
    Delete,
    Archive,
}

/// Status of email sync operation
#[derive(Debug, Clone)]
pub enum SyncStatus {
    Success { fetched: usize, sent: usize },
    Error(String),
}

/// IMAP email fetcher
#[derive(Clone, Debug)]
pub struct ImapClient {
    credentials: Credentials,
}

impl ImapClient {
    /// Create a new IMAP client with credentials
    pub fn new(credentials: Credentials) -> Self {
        Self { credentials }
    }

    /// Fetch emails from IMAP server
    pub async fn fetch_emails(&self, folder: &str, limit: Option<usize>) -> Result<Vec<DbEmail>> {
        let credentials = self.credentials.clone();
        let folder = folder.to_string();
        
        // Use spawn_blocking to run blocking IMAP operations in a thread pool
        tokio::task::spawn_blocking(move || {
            Self::fetch_emails_blocking(&credentials, &folder, limit)
        })
        .await
        .context("Task join error")?
    }

    /// Blocking IMAP fetch implementation
    fn fetch_emails_blocking(
        credentials: &Credentials,
        folder: &str,
        limit: Option<usize>,
    ) -> Result<Vec<DbEmail>> {
        // Connect to IMAP server with TLS
        let domain = &credentials.imap_server;
        let port = credentials.imap_port;
        
        let tls = native_tls::TlsConnector::builder()
            .build()
            .context("Failed to build TLS connector")?;
        
        let client = imap::connect((domain.as_str(), port), domain, &tls)
            .context(format!("Failed to connect to {}:{}", domain, port))?;

        // Login
        let mut session = client
            .login(&credentials.imap_username, &credentials.imap_password)
            .map_err(|e| anyhow!("IMAP login failed: {:?}", e.0))?;

        // Select mailbox
        session.select(folder)
            .context(format!("Failed to select folder: {}", folder))?;

        // Search for all messages
        let message_ids = session.search("ALL")
            .context("Failed to search messages")?;

        // Convert HashSet to Vec and limit results if requested
        let mut message_vec: Vec<u32> = message_ids.into_iter().collect();
        message_vec.sort_unstable();
        message_vec.reverse(); // Most recent first
        
        let message_ids: Vec<u32> = if let Some(limit) = limit {
            message_vec.into_iter().take(limit).collect()
        } else {
            message_vec
        };

        let mut emails = Vec::new();

        // Fetch each message
        for msg_id in message_ids {
            match session.fetch(msg_id.to_string(), "(FLAGS RFC822)") {
                Ok(messages) => {
                    for fetch in messages.iter() {
                        if let Some(body) = fetch.body() {
                            match Self::parse_email(body, fetch.flags(), folder) {
                                Ok(email) => emails.push(email),
                                Err(e) => {
                                    eprintln!("Failed to parse email {}: {}", msg_id, e);
                                    continue;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch message {}: {}", msg_id, e);
                    continue;
                }
            }
        }

        // Logout
        session.logout().ok();

        Ok(emails)
    }

    /// Parse email from raw RFC822 bytes
    fn parse_email(body: &[u8], flags: &[imap::types::Flag], folder: &str) -> Result<DbEmail> {
        let parsed = mail_parser::MessageParser::default()
            .parse(body)
            .ok_or_else(|| anyhow!("Failed to parse email"))?;

        let from = parsed
            .from()
            .and_then(|addrs| addrs.first())
            .and_then(|addr| addr.address())
            .unwrap_or("unknown@unknown.com")
            .to_string();

        let to = parsed
            .to()
            .and_then(|addrs| addrs.first())
            .and_then(|addr| addr.address())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "".to_string());

        let subject = parsed
            .subject()
            .unwrap_or("(No Subject)")
            .to_string();

        let body_text = parsed
            .body_text(0)
            .or_else(|| parsed.body_html(0))
            .map(|s| s.to_string())
            .unwrap_or_else(|| "".to_string());

        let preview = body_text
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(100)
            .collect::<String>();

        let date = parsed
            .date()
            .map(|dt| format!("{}", dt))
            .unwrap_or_else(|| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                format!("timestamp: {}", timestamp)
            });

        // Check if message is unread
        let is_unread = !flags.iter().any(|f| matches!(f, imap::types::Flag::Seen));
        let is_flagged = flags.iter().any(|f| matches!(f, imap::types::Flag::Flagged));

        Ok(DbEmail {
            id: 0,
            from_address: from,
            to_addresses: to,
            cc_addresses: None,
            bcc_addresses: None,
            subject,
            body: body_text,
            preview,
            date,
            status: if is_unread { DbEmailStatus::Unread } else { DbEmailStatus::Read },
            is_flagged,
            folder: folder.to_string(),
            thread_id: None,
            account_id: None,
        })
    }

    /// Connect to IMAP server and test connection
    pub async fn test_connection(&self) -> Result<()> {
        let credentials = self.credentials.clone();
        
        tokio::task::spawn_blocking(move || {
            let domain = &credentials.imap_server;
            let port = credentials.imap_port;
            
            let tls = native_tls::TlsConnector::builder()
                .build()
                .context("Failed to build TLS connector")?;
            
            let client = imap::connect((domain.as_str(), port), domain, &tls)
                .context(format!("Failed to connect to {}:{}", domain, port))?;

            let mut session = client
                .login(&credentials.imap_username, &credentials.imap_password)
                .map_err(|e| anyhow!("IMAP login failed: {:?}", e.0))?;

            session.logout().ok();
            Ok(())
        })
        .await
        .context("Task join error")?
    }

    /// Sync a specific folder
    pub async fn sync_folder(&self, folder: &str) -> Result<usize> {
        let emails = self.fetch_emails(folder, None).await?;
        Ok(emails.len())
    }
}

/// SMTP email sender (stub implementation)
#[derive(Clone, Debug)]
pub struct SmtpClient {
    credentials: Credentials,
}

impl SmtpClient {
    /// Create a new SMTP client with credentials
    pub fn new(credentials: Credentials) -> Self {
        Self { credentials }
    }

    /// Send an email via SMTP
    pub async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<()> {
        let credentials = self.credentials.clone();
        let to = to.to_string();
        let subject = subject.to_string();
        let body = body.to_string();
        
        tokio::task::spawn_blocking(move || {
            Self::send_email_blocking(&credentials, &to, &subject, &body)
        })
        .await
        .context("Task join error")?
    }

    /// Blocking SMTP send implementation
    fn send_email_blocking(
        credentials: &Credentials,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<()> {
        use lettre::message::header::ContentType;
        use lettre::transport::smtp::authentication::Credentials as LettreCredentials;
        use lettre::{Message, SmtpTransport, Transport};

        // Build email message
        let email = Message::builder()
            .from(credentials.smtp_username.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .context("Failed to build email")?;

        // Configure SMTP transport
        let creds = LettreCredentials::new(
            credentials.smtp_username.clone(),
            credentials.smtp_password.clone(),
        );

        let mailer = SmtpTransport::relay(&credentials.smtp_server)
            .context("Failed to create SMTP transport")?
            .credentials(creds)
            .port(credentials.smtp_port)
            .build();

        // Send email
        mailer
            .send(&email)
            .context("Failed to send email via SMTP")?;

        Ok(())
    }

    /// Test SMTP connection
    pub async fn test_connection(&self) -> Result<()> {
        let credentials = self.credentials.clone();
        
        tokio::task::spawn_blocking(move || {
            use lettre::transport::smtp::authentication::Credentials as LettreCredentials;
            use lettre::{SmtpTransport, Transport};

            let creds = LettreCredentials::new(
                credentials.smtp_username.clone(),
                credentials.smtp_password.clone(),
            );

            let mailer = SmtpTransport::relay(&credentials.smtp_server)
                .context("Failed to create SMTP transport")?
                .credentials(creds)
                .port(credentials.smtp_port)
                .build();

            mailer
                .test_connection()
                .context("SMTP connection test failed")?;

            Ok(())
        })
        .await
        .context("Task join error")?
    }
}

/// Email sync manager that coordinates IMAP/SMTP operations and inbox rules
#[derive(Clone, Debug)]
pub struct EmailSyncManager {
    imap_client: Option<ImapClient>,
    smtp_client: Option<SmtpClient>,
    // Note: Vec is used for simplicity. For large rule sets, consider HashMap<i64, InboxRule>
    // for O(1) lookups in remove_rule, update_rule, and set_rule_enabled operations.
    rules: Vec<InboxRule>,
}

impl EmailSyncManager {
    /// Create a new sync manager with credentials
    pub fn new(credentials: Option<Credentials>) -> Self {
        let (imap_client, smtp_client) = if let Some(creds) = credentials {
            (
                Some(ImapClient::new(creds.clone())),
                Some(SmtpClient::new(creds)),
            )
        } else {
            (None, None)
        };

        Self {
            imap_client,
            smtp_client,
            rules: Vec::new(),
        }
    }

    // ============ Inbox Rules Management ============

    /// Add an inbox rule
    pub fn add_rule(&mut self, rule: InboxRule) {
        self.rules.push(rule);
    }

    /// Remove a rule by ID
    pub fn remove_rule(&mut self, rule_id: i64) {
        self.rules.retain(|r| r.id != rule_id);
    }

    /// Update an existing rule
    pub fn update_rule(&mut self, rule: InboxRule) {
        if let Some(existing) = self.rules.iter_mut().find(|r| r.id == rule.id) {
            *existing = rule;
        }
    }

    /// Get all rules
    pub fn get_rules(&self) -> &[InboxRule] {
        &self.rules
    }

    /// Enable or disable a rule
    pub fn set_rule_enabled(&mut self, rule_id: i64, enabled: bool) {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = enabled;
        }
    }

    /// Apply rules to an email and return actions to perform
    pub fn apply_rules(&self, email: &DbEmail) -> Vec<RuleAction> {
        let mut actions = Vec::new();

        for rule in &self.rules {
            if rule.enabled && rule.condition.matches(email) {
                actions.push(rule.action.clone());
            }
        }

        actions
    }

    /// Apply all enabled rules to a batch of emails
    pub fn apply_rules_batch(&self, emails: &[DbEmail]) -> Vec<(usize, Vec<RuleAction>)> {
        emails
            .iter()
            .enumerate()
            .map(|(idx, email)| (idx, self.apply_rules(email)))
            .filter(|(_, actions)| !actions.is_empty())
            .collect()
    }

    // ============ IMAP/SMTP Operations ============

    /// Perform full email sync from IMAP inbox
    pub async fn sync(&self, folder: &str, limit: Option<usize>) -> Result<SyncStatus> {
        if self.imap_client.is_none() {
            return Ok(SyncStatus::Error(
                "No credentials configured. Please set up email credentials first.".to_string()
            ));
        }

        let client = self.imap_client.as_ref().unwrap();
        
        match client.fetch_emails(folder, limit).await {
            Ok(emails) => {
                let count = emails.len();
                Ok(SyncStatus::Success { fetched: count, sent: 0 })
            }
            Err(e) => Ok(SyncStatus::Error(format!("Sync failed: {}", e))),
        }
    }

    /// Get IMAP client for direct operations
    pub fn imap_client(&self) -> Option<&ImapClient> {
        self.imap_client.as_ref()
    }

    /// Test both IMAP and SMTP connections
    pub async fn test_connections(&self) -> Result<(bool, bool)> {
        let imap_ok = if let Some(ref client) = self.imap_client {
            client.test_connection().await.is_ok()
        } else {
            false
        };

        let smtp_ok = if let Some(ref client) = self.smtp_client {
            client.test_connection().await.is_ok()
        } else {
            false
        };

        Ok((imap_ok, smtp_ok))
    }

    /// Check if credentials are configured
    pub fn is_configured(&self) -> bool {
        self.imap_client.is_some() && self.smtp_client.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_credentials() -> Credentials {
        Credentials {
            imap_server: "imap.example.com".to_string(),
            imap_port: 993,
            imap_username: "user@example.com".to_string(),
            imap_password: "password".to_string(),
            smtp_server: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_username: "user@example.com".to_string(),
            smtp_password: "password".to_string(),
        }
    }

    fn create_test_email(from: &str, subject: &str, body: &str) -> DbEmail {
        // Use current timestamp to avoid test expiration issues
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let date = format!("Test date: {}", timestamp);
        
        DbEmail {
            id: 1,
            from_address: from.to_string(),
            to_addresses: "me@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: subject.to_string(),
            body: body.to_string(),
            preview: body.chars().take(100).collect(),
            date,
            status: DbEmailStatus::Unread,
            is_flagged: false,
            folder: "inbox".to_string(),
            thread_id: None,
            account_id: None,
        }
    }

    #[tokio::test]
    async fn test_imap_client_connection_requires_valid_server() {
        let client = ImapClient::new(create_test_credentials());
        // This will fail because we're using fake credentials
        // But it tests that the code path exists
        let result = client.test_connection().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_smtp_client_requires_valid_server() {
        let client = SmtpClient::new(create_test_credentials());
        // This will fail because we're using fake credentials
        // But it tests that the code path exists
        let result = client.send_email("to@example.com", "Test", "Body").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_manager_configured() {
        let manager = EmailSyncManager::new(Some(create_test_credentials()));
        assert!(manager.is_configured());

        let manager_no_creds = EmailSyncManager::new(None);
        assert!(!manager_no_creds.is_configured());
    }

    // Rules tests

    #[test]
    fn test_rule_condition_from_contains() {
        let email = create_test_email("alice@example.com", "Test", "Body");
        
        let condition = RuleCondition::FromContains("alice".to_string());
        assert!(condition.matches(&email));

        let condition = RuleCondition::FromContains("bob".to_string());
        assert!(!condition.matches(&email));
    }

    #[test]
    fn test_rule_condition_subject_contains() {
        let email = create_test_email("alice@example.com", "Important Meeting", "Body");
        
        let condition = RuleCondition::SubjectContains("meeting".to_string());
        assert!(condition.matches(&email));

        let condition = RuleCondition::SubjectContains("party".to_string());
        assert!(!condition.matches(&email));
    }

    #[test]
    fn test_rule_condition_and() {
        let email = create_test_email("alice@example.com", "Important Meeting", "Body");
        
        let condition = RuleCondition::And(
            Box::new(RuleCondition::FromContains("alice".to_string())),
            Box::new(RuleCondition::SubjectContains("meeting".to_string())),
        );
        assert!(condition.matches(&email));

        let condition = RuleCondition::And(
            Box::new(RuleCondition::FromContains("bob".to_string())),
            Box::new(RuleCondition::SubjectContains("meeting".to_string())),
        );
        assert!(!condition.matches(&email));
    }

    #[test]
    fn test_rule_condition_or() {
        let email = create_test_email("alice@example.com", "Test", "Body");
        
        let condition = RuleCondition::Or(
            Box::new(RuleCondition::FromContains("alice".to_string())),
            Box::new(RuleCondition::SubjectContains("party".to_string())),
        );
        assert!(condition.matches(&email));

        let condition = RuleCondition::Or(
            Box::new(RuleCondition::FromContains("bob".to_string())),
            Box::new(RuleCondition::SubjectContains("party".to_string())),
        );
        assert!(!condition.matches(&email));
    }

    #[test]
    fn test_sync_manager_add_rule() {
        let mut manager = EmailSyncManager::new(Some(create_test_credentials()));
        
        let rule = InboxRule {
            id: 1,
            name: "Move newsletters".to_string(),
            condition: RuleCondition::FromContains("newsletter".to_string()),
            action: RuleAction::MoveToFolder("newsletters".to_string()),
            enabled: true,
        };
        
        manager.add_rule(rule);
        assert_eq!(manager.get_rules().len(), 1);
    }

    #[test]
    fn test_sync_manager_remove_rule() {
        let mut manager = EmailSyncManager::new(Some(create_test_credentials()));
        
        let rule = InboxRule {
            id: 1,
            name: "Test rule".to_string(),
            condition: RuleCondition::FromContains("test".to_string()),
            action: RuleAction::Flag,
            enabled: true,
        };
        
        manager.add_rule(rule);
        assert_eq!(manager.get_rules().len(), 1);
        
        manager.remove_rule(1);
        assert_eq!(manager.get_rules().len(), 0);
    }

    #[test]
    fn test_sync_manager_apply_rules() {
        let mut manager = EmailSyncManager::new(Some(create_test_credentials()));
        
        let rule = InboxRule {
            id: 1,
            name: "Flag important".to_string(),
            condition: RuleCondition::SubjectContains("important".to_string()),
            action: RuleAction::Flag,
            enabled: true,
        };
        
        manager.add_rule(rule);
        
        let email = create_test_email("alice@example.com", "Important Meeting", "Body");
        let actions = manager.apply_rules(&email);
        
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], RuleAction::Flag));
    }

    #[test]
    fn test_sync_manager_disabled_rule() {
        let mut manager = EmailSyncManager::new(Some(create_test_credentials()));
        
        let rule = InboxRule {
            id: 1,
            name: "Test rule".to_string(),
            condition: RuleCondition::FromContains("test".to_string()),
            action: RuleAction::Flag,
            enabled: false,  // Disabled
        };
        
        manager.add_rule(rule);
        
        let email = create_test_email("test@example.com", "Test", "Body");
        let actions = manager.apply_rules(&email);
        
        // Should not apply disabled rule
        assert_eq!(actions.len(), 0);
    }

    #[test]
    fn test_sync_manager_multiple_rules() {
        let mut manager = EmailSyncManager::new(Some(create_test_credentials()));
        
        manager.add_rule(InboxRule {
            id: 1,
            name: "Flag important".to_string(),
            condition: RuleCondition::SubjectContains("important".to_string()),
            action: RuleAction::Flag,
            enabled: true,
        });
        
        manager.add_rule(InboxRule {
            id: 2,
            name: "Mark as read".to_string(),
            condition: RuleCondition::SubjectContains("important".to_string()),
            action: RuleAction::MarkAsRead,
            enabled: true,
        });
        
        let email = create_test_email("alice@example.com", "Important Meeting", "Body");
        let actions = manager.apply_rules(&email);
        
        // Both rules should match
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn test_apply_rules_batch() {
        let mut manager = EmailSyncManager::new(Some(create_test_credentials()));
        
        manager.add_rule(InboxRule {
            id: 1,
            name: "Archive newsletters".to_string(),
            condition: RuleCondition::FromContains("newsletter".to_string()),
            action: RuleAction::Archive,
            enabled: true,
        });
        
        let emails = vec![
            create_test_email("newsletter@example.com", "Weekly News", "Body"),
            create_test_email("alice@example.com", "Meeting", "Body"),
            create_test_email("newsletter@company.com", "Updates", "Body"),
        ];
        
        let results = manager.apply_rules_batch(&emails);
        
        // Should match emails at index 0 and 2
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0);
        assert_eq!(results[1].0, 2);
    }
}

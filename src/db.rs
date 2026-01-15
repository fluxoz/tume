use anyhow::{Context, Result};
use libsql::Connection;
use std::path::PathBuf;

/// Represents the local email database using Turso/libSQL
#[derive(Clone)]
pub struct EmailDatabase {
    conn: Connection,
}

/// Email status enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmailStatus {
    Unread,
    Read,
    Archived,
    Deleted,
}

impl EmailStatus {
    pub fn as_str(&self) -> &str {
        match self {
            EmailStatus::Unread => "unread",
            EmailStatus::Read => "read",
            EmailStatus::Archived => "archived",
            EmailStatus::Deleted => "deleted",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "read" => EmailStatus::Read,
            "archived" => EmailStatus::Archived,
            "deleted" => EmailStatus::Deleted,
            _ => EmailStatus::Unread,
        }
    }
}

/// Database representation of an email
#[derive(Debug, Clone)]
pub struct DbEmail {
    pub id: i64,
    pub from_address: String,
    pub to_addresses: String,
    pub cc_addresses: Option<String>,
    pub bcc_addresses: Option<String>,
    pub subject: String,
    pub body: String,
    pub preview: String,
    pub date: String,
    pub status: EmailStatus,
    pub is_flagged: bool,
    pub folder: String,
    pub thread_id: Option<String>,
    pub account_id: Option<i64>,
    pub message_id: Option<String>,
    pub imap_uid: Option<u32>,
}

/// Database representation of a draft email
#[derive(Debug, Clone)]
pub struct DbDraft {
    pub id: i64,
    pub recipients: String,
    pub subject: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
    pub account_id: Option<i64>,
}

/// Database representation of a folder/label
#[derive(Debug, Clone)]
pub struct DbFolder {
    pub id: i64,
    pub name: String,
    pub display_order: i64,
}

/// Database representation of an account
#[derive(Debug, Clone)]
pub struct DbAccount {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub provider: String,
    pub is_default: bool,
    pub color: Option<String>,
    pub display_order: i64,
}

impl EmailDatabase {
    /// Create a new database connection at the specified path
    pub async fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let path = db_path.unwrap_or_else(|| {
            let mut path = dirs::home_dir().expect("Could not find home directory");
            path.push(".local");
            path.push("share");
            path.push("tume");
            std::fs::create_dir_all(&path).expect("Could not create tume directory");
            path.push("mail.db");
            path
        });

        let path_str = path.to_str().expect("Invalid path").to_string();
        let db = libsql::Builder::new_local(path_str)
            .build()
            .await
            .context("Failed to open database")?;

        let conn = db.connect().context("Failed to connect to database")?;

        let db = Self { conn };
        db.initialize_schema().await?;

        Ok(db)
    }

    /// Initialize database schema with all necessary tables
    async fn initialize_schema(&self) -> Result<()> {
        // Create emails table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS emails (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    from_address TEXT NOT NULL,
                    to_addresses TEXT NOT NULL,
                    cc_addresses TEXT,
                    bcc_addresses TEXT,
                    subject TEXT NOT NULL,
                    body TEXT NOT NULL,
                    preview TEXT NOT NULL,
                    date TEXT NOT NULL,
                    status TEXT NOT NULL DEFAULT 'unread',
                    is_flagged INTEGER NOT NULL DEFAULT 0,
                    folder TEXT NOT NULL DEFAULT 'inbox',
                    thread_id TEXT,
                    message_id TEXT,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )",
                (),
            )
            .await
            .context("Failed to create emails table")?;

        // Create drafts table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS drafts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    recipients TEXT NOT NULL,
                    subject TEXT NOT NULL,
                    body TEXT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )",
                (),
            )
            .await
            .context("Failed to create drafts table")?;

        // Create folders table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS folders (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    display_order INTEGER NOT NULL DEFAULT 0,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )",
                (),
            )
            .await
            .context("Failed to create folders table")?;

        // Create attachments table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS attachments (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    email_id INTEGER NOT NULL,
                    filename TEXT NOT NULL,
                    content_type TEXT NOT NULL,
                    size INTEGER NOT NULL,
                    data BLOB NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (email_id) REFERENCES emails(id) ON DELETE CASCADE
                )",
                (),
            )
            .await
            .context("Failed to create attachments table")?;

        // Create accounts table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS accounts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    email TEXT NOT NULL UNIQUE,
                    provider TEXT NOT NULL,
                    is_default INTEGER NOT NULL DEFAULT 0,
                    color TEXT,
                    display_order INTEGER NOT NULL DEFAULT 0,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )",
                (),
            )
            .await
            .context("Failed to create accounts table")?;

        // Create inbox_rules table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS inbox_rules (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    condition_type TEXT NOT NULL,
                    condition_value TEXT NOT NULL,
                    action_type TEXT NOT NULL,
                    action_value TEXT,
                    enabled INTEGER NOT NULL DEFAULT 1,
                    account_id INTEGER,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
                )",
                (),
            )
            .await
            .context("Failed to create inbox_rules table")?;

        // Add account_id column to emails if it doesn't exist (migration)
        // Note: SQLite doesn't support ALTER TABLE ADD COLUMN IF NOT EXISTS directly,
        // so we need to check if the column exists first
        let column_exists = self.check_column_exists("emails", "account_id").await?;
        if !column_exists {
            self.conn
                .execute(
                    "ALTER TABLE emails ADD COLUMN account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL",
                    (),
                )
                .await
                .context("Failed to add account_id to emails table")?;
        }

        // Add account_id column to drafts if it doesn't exist (migration)
        let draft_column_exists = self.check_column_exists("drafts", "account_id").await?;
        if !draft_column_exists {
            self.conn
                .execute(
                    "ALTER TABLE drafts ADD COLUMN account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL",
                    (),
                )
                .await
                .context("Failed to add account_id to drafts table")?;
        }

        // Add message_id column to emails if it doesn't exist (migration for deduplication)
        let message_id_column_exists = self.check_column_exists("emails", "message_id").await?;
        if !message_id_column_exists {
            self.conn
                .execute(
                    "ALTER TABLE emails ADD COLUMN message_id TEXT",
                    (),
                )
                .await
                .context("Failed to add message_id to emails table")?;
        }

        // Add imap_uid column to emails if it doesn't exist (migration for remote delete support)
        let imap_uid_column_exists = self.check_column_exists("emails", "imap_uid").await?;
        if !imap_uid_column_exists {
            self.conn
                .execute(
                    "ALTER TABLE emails ADD COLUMN imap_uid INTEGER",
                    (),
                )
                .await
                .context("Failed to add imap_uid to emails table")?;
        }

        // Create indexes for better query performance
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_emails_status ON emails(status)",
                (),
            )
            .await
            .context("Failed to create status index")?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_emails_folder ON emails(folder)",
                (),
            )
            .await
            .context("Failed to create folder index")?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_emails_date ON emails(date DESC)",
                (),
            )
            .await
            .context("Failed to create date index")?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_emails_account_id ON emails(account_id)",
                (),
            )
            .await
            .context("Failed to create account_id index")?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_emails_message_id ON emails(message_id)",
                (),
            )
            .await
            .context("Failed to create message_id index")?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_emails_imap_uid ON emails(imap_uid)",
                (),
            )
            .await
            .context("Failed to create imap_uid index")?;

        // Initialize default folders if they don't exist
        self.initialize_default_folders().await?;

        Ok(())
    }

    /// Check if a column exists in a table
    async fn check_column_exists(&self, table: &str, column: &str) -> Result<bool> {
        // Whitelist of allowed table names to prevent SQL injection
        let allowed_tables = ["emails", "drafts", "accounts", "folders", "attachments", "inbox_rules"];
        if !allowed_tables.contains(&table) {
            anyhow::bail!("Invalid table name: {}", table);
        }

        let mut rows = self
            .conn
            .query(
                &format!("PRAGMA table_info({})", table),
                (),
            )
            .await
            .context("Failed to query table info")?;

        while let Some(row) = rows.next().await? {
            let col_name: String = row.get(1)?;
            if col_name == column {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Initialize default folders (inbox, sent, drafts, trash, archive)
    async fn initialize_default_folders(&self) -> Result<()> {
        let default_folders = vec![
            ("inbox", 0),
            ("sent", 1),
            ("drafts", 2),
            ("archive", 3),
            ("trash", 4),
        ];

        for (name, order) in default_folders {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO folders (name, display_order) VALUES (?1, ?2)",
                    libsql::params![name, order],
                )
                .await
                .context("Failed to insert default folder")?;
        }

        Ok(())
    }

    /// Insert a new email into the database
    pub async fn insert_email(&self, email: &DbEmail) -> Result<i64> {
        self.conn
            .execute(
                "INSERT INTO emails (
                    from_address, to_addresses, cc_addresses, bcc_addresses, 
                    subject, body, preview, date, status, is_flagged, 
                    folder, thread_id, account_id, message_id, imap_uid
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                libsql::params![
                    email.from_address.as_str(),
                    email.to_addresses.as_str(),
                    email.cc_addresses.as_deref(),
                    email.bcc_addresses.as_deref(),
                    email.subject.as_str(),
                    email.body.as_str(),
                    email.preview.as_str(),
                    email.date.as_str(),
                    email.status.as_str(),
                    email.is_flagged as i64,
                    email.folder.as_str(),
                    email.thread_id.as_deref(),
                    email.account_id,
                    email.message_id.as_deref(),
                    email.imap_uid,
                ],
            )
            .await
            .context("Failed to insert email")?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get all emails from a specific folder
    pub async fn get_emails_by_folder(&self, folder: &str) -> Result<Vec<DbEmail>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, from_address, to_addresses, cc_addresses, bcc_addresses,
                        subject, body, preview, date, status, is_flagged, folder, thread_id, account_id, message_id, imap_uid
                 FROM emails
                 WHERE folder = ?1 AND status != 'deleted'
                 ORDER BY date DESC",
                libsql::params![folder],
            )
            .await
            .context("Failed to query emails")?;

        let mut emails = Vec::new();
        while let Some(row) = rows.next().await? {
            emails.push(DbEmail {
                id: row.get(0)?,
                from_address: row.get(1)?,
                to_addresses: row.get(2)?,
                cc_addresses: row.get(3)?,
                bcc_addresses: row.get(4)?,
                subject: row.get(5)?,
                body: row.get(6)?,
                preview: row.get(7)?,
                date: row.get(8)?,
                status: EmailStatus::from_str(&row.get::<String>(9)?),
                is_flagged: row.get::<i64>(10)? != 0,
                folder: row.get(11)?,
                thread_id: row.get(12)?,
                account_id: row.get(13)?,
                message_id: row.get(14)?,
                imap_uid: row.get(15)?,
            });
        }

        Ok(emails)
    }

    /// Get a single email by ID
    pub async fn get_email_by_id(&self, id: i64) -> Result<Option<DbEmail>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, from_address, to_addresses, cc_addresses, bcc_addresses,
                        subject, body, preview, date, status, is_flagged, folder, thread_id, account_id, message_id, imap_uid
                 FROM emails
                 WHERE id = ?1",
                libsql::params![id],
            )
            .await
            .context("Failed to query email")?;

        if let Some(row) = rows.next().await? {
            Ok(Some(DbEmail {
                id: row.get(0)?,
                from_address: row.get(1)?,
                to_addresses: row.get(2)?,
                cc_addresses: row.get(3)?,
                bcc_addresses: row.get(4)?,
                subject: row.get(5)?,
                body: row.get(6)?,
                preview: row.get(7)?,
                date: row.get(8)?,
                status: EmailStatus::from_str(&row.get::<String>(9)?),
                is_flagged: row.get::<i64>(10)? != 0,
                folder: row.get(11)?,
                thread_id: row.get(12)?,
                account_id: row.get(13)?,
                message_id: row.get(14)?,
                imap_uid: row.get(15)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Update email status
    pub async fn update_email_status(&self, id: i64, status: EmailStatus) -> Result<()> {
        self.conn
            .execute(
                "UPDATE emails SET status = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                libsql::params![status.as_str(), id],
            )
            .await
            .context("Failed to update email status")?;

        Ok(())
    }

    /// Move email to a different folder
    pub async fn move_email_to_folder(&self, id: i64, folder: &str) -> Result<()> {
        self.conn
            .execute(
                "UPDATE emails SET folder = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                libsql::params![folder, id],
            )
            .await
            .context("Failed to move email to folder")?;

        Ok(())
    }

    /// Toggle email flag
    pub async fn toggle_email_flag(&self, id: i64) -> Result<bool> {
        // First get current flag status
        let mut rows = self
            .conn
            .query(
                "SELECT is_flagged FROM emails WHERE id = ?1",
                libsql::params![id],
            )
            .await
            .context("Failed to query email flag")?;

        let current_flag = if let Some(row) = rows.next().await? {
            row.get::<i64>(0)? != 0
        } else {
            return Ok(false);
        };

        let new_flag = !current_flag;
        self.conn
            .execute(
                "UPDATE emails SET is_flagged = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                libsql::params![new_flag as i64, id],
            )
            .await
            .context("Failed to toggle email flag")?;

        Ok(new_flag)
    }

    /// Delete email (soft delete by moving to trash)
    pub async fn delete_email(&self, id: i64) -> Result<()> {
        self.update_email_status(id, EmailStatus::Deleted).await?;
        self.move_email_to_folder(id, "trash").await?;
        Ok(())
    }

    /// Archive email
    pub async fn archive_email(&self, id: i64) -> Result<()> {
        self.update_email_status(id, EmailStatus::Archived).await?;
        self.move_email_to_folder(id, "archive").await?;
        Ok(())
    }

    /// Save a draft
    pub async fn save_draft(&self, draft: &DbDraft) -> Result<i64> {
        if draft.id == 0 {
            // Insert new draft
            self.conn
                .execute(
                    "INSERT INTO drafts (recipients, subject, body, account_id) VALUES (?1, ?2, ?3, ?4)",
                    libsql::params![
                        draft.recipients.as_str(),
                        draft.subject.as_str(),
                        draft.body.as_str(),
                        draft.account_id,
                    ],
                )
                .await
                .context("Failed to insert draft")?;
            Ok(self.conn.last_insert_rowid())
        } else {
            // Update existing draft
            self.conn
                .execute(
                    "UPDATE drafts SET recipients = ?1, subject = ?2, body = ?3, account_id = ?4, updated_at = CURRENT_TIMESTAMP WHERE id = ?5",
                    libsql::params![draft.recipients.as_str(), draft.subject.as_str(), draft.body.as_str(), draft.account_id, draft.id],
                )
                .await
                .context("Failed to update draft")?;
            Ok(draft.id)
        }
    }

    /// Get all drafts
    pub async fn get_drafts(&self) -> Result<Vec<DbDraft>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, recipients, subject, body, created_at, updated_at, account_id
                 FROM drafts
                 ORDER BY updated_at DESC",
                (),
            )
            .await
            .context("Failed to query drafts")?;

        let mut drafts = Vec::new();
        while let Some(row) = rows.next().await? {
            drafts.push(DbDraft {
                id: row.get(0)?,
                recipients: row.get(1)?,
                subject: row.get(2)?,
                body: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                account_id: row.get(6)?,
            });
        }

        Ok(drafts)
    }

    /// Delete a draft
    pub async fn delete_draft(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM drafts WHERE id = ?1", libsql::params![id])
            .await
            .context("Failed to delete draft")?;

        Ok(())
    }

    /// Get all folders
    pub async fn get_folders(&self) -> Result<Vec<DbFolder>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, name, display_order FROM folders ORDER BY display_order",
                (),
            )
            .await
            .context("Failed to query folders")?;

        let mut folders = Vec::new();
        while let Some(row) = rows.next().await? {
            folders.push(DbFolder {
                id: row.get(0)?,
                name: row.get(1)?,
                display_order: row.get(2)?,
            });
        }

        Ok(folders)
    }

    /// Search emails by query string (searches in subject, body, and from address)
    pub async fn search_emails(&self, query: &str) -> Result<Vec<DbEmail>> {
        let search_pattern = format!("%{}%", query);
        let mut rows = self
            .conn
            .query(
                "SELECT id, from_address, to_addresses, cc_addresses, bcc_addresses,
                        subject, body, preview, date, status, is_flagged, folder, thread_id, account_id, message_id, imap_uid
                 FROM emails
                 WHERE (subject LIKE ?1 OR body LIKE ?1 OR from_address LIKE ?1)
                   AND status != 'deleted'
                 ORDER BY date DESC",
                libsql::params![search_pattern.as_str()],
            )
            .await
            .context("Failed to search emails")?;

        let mut emails = Vec::new();
        while let Some(row) = rows.next().await? {
            emails.push(DbEmail {
                id: row.get(0)?,
                from_address: row.get(1)?,
                to_addresses: row.get(2)?,
                cc_addresses: row.get(3)?,
                bcc_addresses: row.get(4)?,
                subject: row.get(5)?,
                body: row.get(6)?,
                preview: row.get(7)?,
                date: row.get(8)?,
                status: EmailStatus::from_str(&row.get::<String>(9)?),
                is_flagged: row.get::<i64>(10)? != 0,
                folder: row.get(11)?,
                thread_id: row.get(12)?,
                account_id: row.get(13)?,
                message_id: row.get(14)?,
                imap_uid: row.get(15)?,
            });
        }

        Ok(emails)
    }

    /// Get all emails from a specific folder and account
    pub async fn get_emails_by_folder_and_account(&self, folder: &str, account_id: Option<i64>) -> Result<Vec<DbEmail>> {
        let query = if account_id.is_some() {
            "SELECT id, from_address, to_addresses, cc_addresses, bcc_addresses,
                    subject, body, preview, date, status, is_flagged, folder, thread_id, account_id, message_id, imap_uid
             FROM emails
             WHERE folder = ?1 AND account_id = ?2 AND status != 'deleted'
             ORDER BY date DESC"
        } else {
            "SELECT id, from_address, to_addresses, cc_addresses, bcc_addresses,
                    subject, body, preview, date, status, is_flagged, folder, thread_id, account_id, message_id, imap_uid
             FROM emails
             WHERE folder = ?1 AND account_id IS NULL AND status != 'deleted'
             ORDER BY date DESC"
        };

        let mut rows = if let Some(acc_id) = account_id {
            self.conn
                .query(query, libsql::params![folder, acc_id])
                .await
                .context("Failed to query emails by folder and account")?
        } else {
            self.conn
                .query(query, libsql::params![folder])
                .await
                .context("Failed to query emails by folder")?
        };

        let mut emails = Vec::new();
        while let Some(row) = rows.next().await? {
            emails.push(DbEmail {
                id: row.get(0)?,
                from_address: row.get(1)?,
                to_addresses: row.get(2)?,
                cc_addresses: row.get(3)?,
                bcc_addresses: row.get(4)?,
                subject: row.get(5)?,
                body: row.get(6)?,
                preview: row.get(7)?,
                date: row.get(8)?,
                status: EmailStatus::from_str(&row.get::<String>(9)?),
                is_flagged: row.get::<i64>(10)? != 0,
                folder: row.get(11)?,
                thread_id: row.get(12)?,
                account_id: row.get(13)?,
                message_id: row.get(14)?,
                imap_uid: row.get(15)?,
            });
        }

        Ok(emails)
    }

    /// Insert or update an account
    pub async fn save_account(&self, account: &DbAccount) -> Result<i64> {
        if account.id == 0 {
            // Insert new account
            self.conn
                .execute(
                    "INSERT INTO accounts (name, email, provider, is_default, color, display_order)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    libsql::params![
                        account.name.as_str(),
                        account.email.as_str(),
                        account.provider.as_str(),
                        account.is_default as i64,
                        account.color.as_deref(),
                        account.display_order,
                    ],
                )
                .await
                .context("Failed to insert account")?;
            Ok(self.conn.last_insert_rowid())
        } else {
            // Update existing account
            self.conn
                .execute(
                    "UPDATE accounts SET name = ?1, email = ?2, provider = ?3, is_default = ?4, 
                     color = ?5, display_order = ?6, updated_at = CURRENT_TIMESTAMP WHERE id = ?7",
                    libsql::params![
                        account.name.as_str(),
                        account.email.as_str(),
                        account.provider.as_str(),
                        account.is_default as i64,
                        account.color.as_deref(),
                        account.display_order,
                        account.id,
                    ],
                )
                .await
                .context("Failed to update account")?;
            Ok(account.id)
        }
    }

    /// Get all accounts
    pub async fn get_accounts(&self) -> Result<Vec<DbAccount>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, name, email, provider, is_default, color, display_order
                 FROM accounts
                 ORDER BY display_order",
                (),
            )
            .await
            .context("Failed to query accounts")?;

        let mut accounts = Vec::new();
        while let Some(row) = rows.next().await? {
            accounts.push(DbAccount {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                provider: row.get(3)?,
                is_default: row.get::<i64>(4)? != 0,
                color: row.get(5)?,
                display_order: row.get(6)?,
            });
        }

        Ok(accounts)
    }

    /// Get account by ID
    pub async fn get_account_by_id(&self, id: i64) -> Result<Option<DbAccount>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, name, email, provider, is_default, color, display_order
                 FROM accounts
                 WHERE id = ?1",
                libsql::params![id],
            )
            .await
            .context("Failed to query account")?;

        if let Some(row) = rows.next().await? {
            Ok(Some(DbAccount {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                provider: row.get(3)?,
                is_default: row.get::<i64>(4)? != 0,
                color: row.get(5)?,
                display_order: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete an account
    pub async fn delete_account(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM accounts WHERE id = ?1", libsql::params![id])
            .await
            .context("Failed to delete account")?;
        Ok(())
    }

    /// Clear all emails from the inbox folder (for development/testing)
    pub async fn clear_inbox(&self) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM emails WHERE folder = 'inbox'",
                (),
            )
            .await
            .context("Failed to clear inbox")?;

        Ok(())
    }

    /// Check if an email with the given message_id already exists
    /// Returns true if the email exists (to prevent duplicates during sync)
    pub async fn email_exists_by_message_id(&self, message_id: &str) -> Result<bool> {
        let mut rows = self
            .conn
            .query(
                "SELECT COUNT(*) FROM emails WHERE message_id = ?1",
                libsql::params![message_id],
            )
            .await
            .context("Failed to check email existence")?;

        if let Some(row) = rows.next().await? {
            let count: i64 = row.get(0)?;
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    async fn create_test_db() -> Result<EmailDatabase> {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = PathBuf::from(format!("/tmp/test_tume_{}_{}.db", std::process::id(), id));
        // Clean up if exists
        let _ = std::fs::remove_file(&path);
        EmailDatabase::new(Some(path)).await
    }

    #[tokio::test]
    async fn test_database_initialization() {
        let db = create_test_db().await.unwrap();
        let folders = db.get_folders().await.unwrap();
        assert_eq!(folders.len(), 5);
        assert_eq!(folders[0].name, "inbox");
    }

    #[tokio::test]
    async fn test_insert_and_get_email() {
        let db = create_test_db().await.unwrap();

        let email = DbEmail {
            id: 0,
            from_address: "test@example.com".to_string(),
            to_addresses: "recipient@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: "Test Subject".to_string(),
            body: "Test body content".to_string(),
            preview: "Test body content".to_string(),
            date: "2026-01-12 12:00".to_string(),
            status: EmailStatus::Unread,
            is_flagged: false,
            folder: "inbox".to_string(),
            thread_id: None,
            account_id: None,
            message_id: Some("<test123@example.com>".to_string()),
        };

        let id = db.insert_email(&email).await.unwrap();
        assert!(id > 0);

        let retrieved = db.get_email_by_id(id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.from_address, "test@example.com");
        assert_eq!(retrieved.subject, "Test Subject");
        assert_eq!(retrieved.message_id, Some("<test123@example.com>".to_string()));
    }

    #[tokio::test]
    async fn test_get_emails_by_folder() {
        let db = create_test_db().await.unwrap();

        let email1 = DbEmail {
            id: 0,
            from_address: "test1@example.com".to_string(),
            to_addresses: "recipient@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: "Test 1".to_string(),
            body: "Body 1".to_string(),
            preview: "Body 1".to_string(),
            date: "2026-01-12 12:00".to_string(),
            status: EmailStatus::Unread,
            is_flagged: false,
            folder: "inbox".to_string(),
            thread_id: None,
            account_id: None,
            message_id: Some("<test1@example.com>".to_string()),
        };

        let email2 = DbEmail {
            id: 0,
            from_address: "test2@example.com".to_string(),
            to_addresses: "recipient@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: "Test 2".to_string(),
            body: "Body 2".to_string(),
            preview: "Body 2".to_string(),
            date: "2026-01-12 13:00".to_string(),
            status: EmailStatus::Read,
            is_flagged: false,
            folder: "sent".to_string(),
            thread_id: None,
            account_id: None,
            message_id: Some("<test2@example.com>".to_string()),
        };

        db.insert_email(&email1).await.unwrap();
        db.insert_email(&email2).await.unwrap();

        let inbox_emails = db.get_emails_by_folder("inbox").await.unwrap();
        assert_eq!(inbox_emails.len(), 1);
        assert_eq!(inbox_emails[0].subject, "Test 1");

        let sent_emails = db.get_emails_by_folder("sent").await.unwrap();
        assert_eq!(sent_emails.len(), 1);
        assert_eq!(sent_emails[0].subject, "Test 2");
    }

    #[tokio::test]
    async fn test_update_email_status() {
        let db = create_test_db().await.unwrap();

        let email = DbEmail {
            id: 0,
            from_address: "test@example.com".to_string(),
            to_addresses: "recipient@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: "Test".to_string(),
            body: "Body".to_string(),
            preview: "Body".to_string(),
            date: "2026-01-12 12:00".to_string(),
            status: EmailStatus::Unread,
            is_flagged: false,
            folder: "inbox".to_string(),
            thread_id: None,
            account_id: None,
            message_id: None,
        };

        let id = db.insert_email(&email).await.unwrap();
        db.update_email_status(id, EmailStatus::Read).await.unwrap();

        let retrieved = db.get_email_by_id(id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, EmailStatus::Read);
    }

    #[tokio::test]
    async fn test_archive_email() {
        let db = create_test_db().await.unwrap();

        let email = DbEmail {
            id: 0,
            from_address: "test@example.com".to_string(),
            to_addresses: "recipient@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: "Test".to_string(),
            body: "Body".to_string(),
            preview: "Body".to_string(),
            date: "2026-01-12 12:00".to_string(),
            status: EmailStatus::Unread,
            is_flagged: false,
            folder: "inbox".to_string(),
            thread_id: None,
            account_id: None,
            message_id: None,
        };

        let id = db.insert_email(&email).await.unwrap();
        db.archive_email(id).await.unwrap();

        let retrieved = db.get_email_by_id(id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, EmailStatus::Archived);
        assert_eq!(retrieved.folder, "archive");
    }

    #[tokio::test]
    async fn test_save_and_get_drafts() {
        let db = create_test_db().await.unwrap();

        let draft = DbDraft {
            id: 0,
            recipients: "test@example.com".to_string(),
            subject: "Draft subject".to_string(),
            body: "Draft body".to_string(),
            created_at: String::new(),
            updated_at: String::new(),
            account_id: None,
        };

        let id = db.save_draft(&draft).await.unwrap();
        assert!(id > 0);

        let drafts = db.get_drafts().await.unwrap();
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].subject, "Draft subject");
    }

    #[tokio::test]
    async fn test_search_emails() {
        let db = create_test_db().await.unwrap();

        let email1 = DbEmail {
            id: 0,
            from_address: "alice@example.com".to_string(),
            to_addresses: "recipient@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: "Meeting notes".to_string(),
            body: "Important meeting discussion".to_string(),
            preview: "Important meeting discussion".to_string(),
            date: "2026-01-12 12:00".to_string(),
            status: EmailStatus::Unread,
            is_flagged: false,
            folder: "inbox".to_string(),
            thread_id: None,
            account_id: None,
            message_id: None,
        };

        let email2 = DbEmail {
            id: 0,
            from_address: "bob@example.com".to_string(),
            to_addresses: "recipient@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: "Project update".to_string(),
            body: "The project is progressing well".to_string(),
            preview: "The project is progressing well".to_string(),
            date: "2026-01-12 13:00".to_string(),
            status: EmailStatus::Read,
            is_flagged: false,
            folder: "inbox".to_string(),
            thread_id: None,
            account_id: None,
            message_id: None,
        };

        db.insert_email(&email1).await.unwrap();
        db.insert_email(&email2).await.unwrap();

        let results = db.search_emails("meeting").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].subject, "Meeting notes");

        let results = db.search_emails("project").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].subject, "Project update");

        let results = db.search_emails("alice").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_clear_inbox() {
        let db = create_test_db().await.unwrap();

        let email = DbEmail {
            id: 0,
            from_address: "test@example.com".to_string(),
            to_addresses: "me@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: "Test".to_string(),
            body: "Body".to_string(),
            preview: "Body".to_string(),
            date: "2026-01-12 12:00".to_string(),
            status: EmailStatus::Unread,
            is_flagged: false,
            folder: "inbox".to_string(),
            thread_id: None,
            account_id: None,
            message_id: None,
        };

        db.insert_email(&email).await.unwrap();
        let emails = db.get_emails_by_folder("inbox").await.unwrap();
        assert_eq!(emails.len(), 1);

        // Clear inbox
        db.clear_inbox().await.unwrap();
        let emails = db.get_emails_by_folder("inbox").await.unwrap();
        assert_eq!(emails.len(), 0);
    }

    #[tokio::test]
    async fn test_email_deduplication_by_message_id() {
        let db = create_test_db().await.unwrap();

        // Create an email with a message_id
        let email1 = DbEmail {
            id: 0,
            from_address: "test@example.com".to_string(),
            to_addresses: "me@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: "Test Email".to_string(),
            body: "Body".to_string(),
            preview: "Body".to_string(),
            date: "2026-01-12 12:00".to_string(),
            status: EmailStatus::Unread,
            is_flagged: false,
            folder: "inbox".to_string(),
            thread_id: None,
            account_id: None,
            message_id: Some("<unique-123@example.com>".to_string()),
        };

        // Insert the email
        db.insert_email(&email1).await.unwrap();
        
        // Check that it exists
        let exists = db.email_exists_by_message_id("<unique-123@example.com>").await.unwrap();
        assert!(exists);
        
        // Check that another message_id doesn't exist
        let not_exists = db.email_exists_by_message_id("<different-456@example.com>").await.unwrap();
        assert!(!not_exists);
        
        // Verify only one email in inbox
        let emails = db.get_emails_by_folder("inbox").await.unwrap();
        assert_eq!(emails.len(), 1);
    }

    #[tokio::test]
    async fn test_sync_deduplication_workflow() {
        let db = create_test_db().await.unwrap();

        // Simulate first sync - insert 3 emails
        let email1 = DbEmail {
            id: 0,
            from_address: "sender1@example.com".to_string(),
            to_addresses: "me@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: "Email 1".to_string(),
            body: "Body 1".to_string(),
            preview: "Body 1".to_string(),
            date: "2026-01-12 12:00".to_string(),
            status: EmailStatus::Unread,
            is_flagged: false,
            folder: "inbox".to_string(),
            thread_id: None,
            account_id: None,
            message_id: Some("<msg1@server.com>".to_string()),
        };

        let email2 = DbEmail {
            id: 0,
            from_address: "sender2@example.com".to_string(),
            to_addresses: "me@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: "Email 2".to_string(),
            body: "Body 2".to_string(),
            preview: "Body 2".to_string(),
            date: "2026-01-12 13:00".to_string(),
            status: EmailStatus::Unread,
            is_flagged: false,
            folder: "inbox".to_string(),
            thread_id: None,
            account_id: None,
            message_id: Some("<msg2@server.com>".to_string()),
        };

        let email3 = DbEmail {
            id: 0,
            from_address: "sender3@example.com".to_string(),
            to_addresses: "me@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: "Email 3".to_string(),
            body: "Body 3".to_string(),
            preview: "Body 3".to_string(),
            date: "2026-01-12 14:00".to_string(),
            status: EmailStatus::Unread,
            is_flagged: false,
            folder: "inbox".to_string(),
            thread_id: None,
            account_id: None,
            message_id: Some("<msg3@server.com>".to_string()),
        };

        db.insert_email(&email1).await.unwrap();
        db.insert_email(&email2).await.unwrap();
        db.insert_email(&email3).await.unwrap();

        let emails = db.get_emails_by_folder("inbox").await.unwrap();
        assert_eq!(emails.len(), 3);

        // Simulate second sync - try to insert the same emails plus 2 new ones
        // These should be skipped
        let exists1 = db.email_exists_by_message_id("<msg1@server.com>").await.unwrap();
        let exists2 = db.email_exists_by_message_id("<msg2@server.com>").await.unwrap();
        let exists3 = db.email_exists_by_message_id("<msg3@server.com>").await.unwrap();
        assert!(exists1);
        assert!(exists2);
        assert!(exists3);

        // New emails that don't exist yet
        let exists4 = db.email_exists_by_message_id("<msg4@server.com>").await.unwrap();
        let exists5 = db.email_exists_by_message_id("<msg5@server.com>").await.unwrap();
        assert!(!exists4);
        assert!(!exists5);

        // Insert only the new emails
        let email4 = DbEmail {
            id: 0,
            from_address: "sender4@example.com".to_string(),
            to_addresses: "me@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: "Email 4".to_string(),
            body: "Body 4".to_string(),
            preview: "Body 4".to_string(),
            date: "2026-01-12 15:00".to_string(),
            status: EmailStatus::Unread,
            is_flagged: false,
            folder: "inbox".to_string(),
            thread_id: None,
            account_id: None,
            message_id: Some("<msg4@server.com>".to_string()),
        };

        let email5 = DbEmail {
            id: 0,
            from_address: "sender5@example.com".to_string(),
            to_addresses: "me@example.com".to_string(),
            cc_addresses: None,
            bcc_addresses: None,
            subject: "Email 5".to_string(),
            body: "Body 5".to_string(),
            preview: "Body 5".to_string(),
            date: "2026-01-12 16:00".to_string(),
            status: EmailStatus::Unread,
            is_flagged: false,
            folder: "inbox".to_string(),
            thread_id: None,
            account_id: None,
            message_id: Some("<msg5@server.com>".to_string()),
        };

        db.insert_email(&email4).await.unwrap();
        db.insert_email(&email5).await.unwrap();

        // Verify we now have 5 total emails (not 8 which would be if duplicates were allowed)
        let emails = db.get_emails_by_folder("inbox").await.unwrap();
        assert_eq!(emails.len(), 5);
        
        // Verify all message IDs are unique
        let message_ids: Vec<_> = emails.iter()
            .filter_map(|e| e.message_id.as_ref())
            .collect();
        assert_eq!(message_ids.len(), 5);
    }
}

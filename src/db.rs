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
}

/// Database representation of a folder/label
#[derive(Debug, Clone)]
pub struct DbFolder {
    pub id: i64,
    pub name: String,
    pub display_order: i64,
}

impl EmailDatabase {
    /// Create a new database connection at the specified path
    pub async fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let path = db_path.unwrap_or_else(|| {
            let mut path = dirs::home_dir().expect("Could not find home directory");
            path.push(".tume");
            std::fs::create_dir_all(&path).expect("Could not create .tume directory");
            path.push("emails.db");
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

        // Initialize default folders if they don't exist
        self.initialize_default_folders().await?;

        Ok(())
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
                "INSERT INTO emails (from_address, to_addresses, cc_addresses, bcc_addresses, 
                                     subject, body, preview, date, status, is_flagged, folder, thread_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
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
                ],
            )
            .await
            .context("Failed to insert email")?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get all emails from a specific folder
    pub async fn get_emails_by_folder(&self, folder: &str) -> Result<Vec<DbEmail>> {
        let mut rows = self.conn
            .query(
                "SELECT id, from_address, to_addresses, cc_addresses, bcc_addresses,
                        subject, body, preview, date, status, is_flagged, folder, thread_id
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
            });
        }

        Ok(emails)
    }

    /// Get a single email by ID
    pub async fn get_email_by_id(&self, id: i64) -> Result<Option<DbEmail>> {
        let mut rows = self.conn
            .query(
                "SELECT id, from_address, to_addresses, cc_addresses, bcc_addresses,
                        subject, body, preview, date, status, is_flagged, folder, thread_id
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
        let mut rows = self.conn
            .query("SELECT is_flagged FROM emails WHERE id = ?1", libsql::params![id])
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
                    "INSERT INTO drafts (recipients, subject, body) VALUES (?1, ?2, ?3)",
                    libsql::params![draft.recipients.as_str(), draft.subject.as_str(), draft.body.as_str()],
                )
                .await
                .context("Failed to insert draft")?;
            Ok(self.conn.last_insert_rowid())
        } else {
            // Update existing draft
            self.conn
                .execute(
                    "UPDATE drafts SET recipients = ?1, subject = ?2, body = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4",
                    libsql::params![draft.recipients.as_str(), draft.subject.as_str(), draft.body.as_str(), draft.id],
                )
                .await
                .context("Failed to update draft")?;
            Ok(draft.id)
        }
    }

    /// Get all drafts
    pub async fn get_drafts(&self) -> Result<Vec<DbDraft>> {
        let mut rows = self.conn
            .query(
                "SELECT id, recipients, subject, body, created_at, updated_at
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
        let mut rows = self.conn
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
        let mut rows = self.conn
            .query(
                "SELECT id, from_address, to_addresses, cc_addresses, bcc_addresses,
                        subject, body, preview, date, status, is_flagged, folder, thread_id
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
            });
        }

        Ok(emails)
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
        };

        let id = db.insert_email(&email).await.unwrap();
        assert!(id > 0);

        let retrieved = db.get_email_by_id(id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.from_address, "test@example.com");
        assert_eq!(retrieved.subject, "Test Subject");
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
}

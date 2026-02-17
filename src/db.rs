use libsql::{Builder, Connection, EncryptionConfig, Cipher::Aes256Cbc};
use std::path::Path;

use crate::email::Email;

/// Simple DB module that opens a local libSQL/SQLite file and exposes a few helpers.
pub struct Db {
    conn: Connection,
}

impl Db {
    /// Open (or create) a local file database at `path`.
    pub async fn open_local(path: impl AsRef<Path>) -> Result<Self, libsql::Error> {
        // libsql wants something path-like; we pass a string.
        let path_str = path.as_ref().to_string_lossy().to_string();
        let encryption_key = "this is my key".as_bytes();
        let encryption_config = EncryptionConfig::new(Aes256Cbc, encryption_key.into());
        let db = Builder::new_local(path_str).encryption_config(encryption_config).build().await?;
        let conn = db.connect()?;
        Ok(Self { conn })
    }

    /// Run basic schema setup (idempotent).
    pub async fn migrate(&self) -> Result<(), libsql::Error> {
        self.conn
            .execute(
                r#"
                create table if not exists emails (
                    id integer primary key,
                    from text not null,
                    subject text not null,
                    body text not null,
                    datetime_received text not null,
                    datetime_read text not null,
                    unread integer not null
                );
                "#,
                (),
            )
            .await?;
        Ok(())
    }

    pub async fn insert_email(&self, email: &Email) -> Result<(), libsql::Error> {
        let unread_i64 = if email.unread { 1 } else { 0 };
        self.conn
            .execute("insert into emails (from_addr, subject, body, datetime_received,  unread) values (?1, ?2, ?3, ?4, ?6)", (email.from.clone(), email.subject.clone(), email.datetime_received.to_rfc3339(), email.unread))
            .await?;
        Ok(())
    }

    pub async fn list_emails(&self) -> Result<Vec<(i64, String, String, bool)>, libsql::Error> {
        let mut rows = self
            .conn
            .query("select id, from_addr, subject, unread from emails order by id desc", ())
            .await?;
        let mut out = Vec::new();
        while let Some(row) = rows.next().await? {
            let id: i64 = row.get(0)?;
            let from_addr: String = row.get(1)?;
            let subject: String = row.get(2)?;
            let unread_i64: i64 = row.get(3)?;
            let unread = unread_i64 != 0;
            out.push((id, from_addr, subject, unread));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    
    use std::os::unix::thread;

    use chrono::Local;
    use futures::executor::block_on;
    use lipsum::lipsum;
    use rand::prelude::*;
    use crate::{db::Db, email::Email};

    #[test]
    fn create_emails() {
        // run async open using a tiny executor (no tokio)
        let db = block_on(Db::open_local("tume.db")).unwrap();

        block_on(db.migrate()).unwrap();

        // if you have migrations/schema, keep this; otherwise remove it
        let _ = block_on(db.migrate());


        for _ in 0..500 {
            let body = lipsum(500);
            let subject = lipsum::lipsum_words_with_rng(thread_rng(), 25);
            let from = format!("{}@test.com", lipsum::lipsum_words_with_rng(thread_rng(), 1));
            let email = Email {
                from,
                subject,
                unread: true,
                datetime_received: Local::now(),
                datetime_read: None,
                body,
            };

            // insert_email is async in the libsql wrapper shown earlier
            block_on(db.insert_email(&email)).unwrap();
        }
        assert_eq!(1, 1);
    }
}

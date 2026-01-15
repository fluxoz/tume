# Email Integration Implementation Guide

This document provides a roadmap for implementing actual IMAP/SMTP email integration in TUME.

## Current Status

### ✅ Completed
- **Inbox Rules Engine**: Fully functional rule-based email filtering
  - Pattern matching (from, subject, body)
  - Logical operators (AND, OR)
  - Actions (move, flag, mark as read, delete, archive)
  - Batch processing support
- **Database Schema**: Complete schema for emails, folders, accounts, attachments, and rules
- **Credentials Management**: Secure storage with system keyring + encrypted file fallback
- **Multi-Account Support**: Configuration and UI for multiple accounts
- **Local Operations**: Full CRUD operations on local database

### ⏳ Pending Implementation
- **IMAP Integration**: Email fetching, folder sync, message operations
- **SMTP Integration**: Email sending with authentication
- **OAuth2 Support**: For Gmail, Outlook, etc.
- **Attachment Handling**: Download, upload, and storage
- **Real-time Sync**: Push notifications or periodic polling

## Implementation Roadmap

### Phase 1: Dependencies

Add required crates to `Cargo.toml`:

```toml
[dependencies]
# IMAP support
async-imap = "0.9"
imap-proto = "0.16"

# SMTP support  
lettre = { version = "0.11", features = ["tokio1-rustls-tls", "smtp-transport"] }

# TLS/SSL
tokio-rustls = "0.25"
rustls-native-certs = "0.7"

# Email parsing
mail-parser = "0.9"

# Async compat layer
tokio-util = { version = "0.7", features = ["compat"] }
futures = "0.3"
```

### Phase 2: IMAP Client Implementation

Replace the stub `ImapClient` in `src/email_sync.rs`:

```rust
use async_imap::Client as AsyncImapClient;
use tokio_util::compat::TokioAsyncReadCompatExt;

pub struct ImapClient {
    credentials: Credentials,
}

impl ImapClient {
    async fn connect(&self) -> Result<ImapSession> {
        // 1. Establish TCP connection
        let addr = format!("{}:{}", self.credentials.imap_server, self.credentials.imap_port);
        let tcp = TcpStream::connect(&addr).await?;
        
        // 2. Wrap with TLS using tokio-rustls
        let tls = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_native_roots()
            .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(tls));
        let tls_stream = connector.connect(
            ServerName::try_from(self.credentials.imap_server.as_str())?,
            tcp
        ).await?;
        
        // 3. Create IMAP client with compat layer for futures traits
        let compat_stream = tls_stream.compat();
        let client = AsyncImapClient::new(compat_stream);
        
        // 4. Login
        let session = client
            .login(&self.credentials.imap_username, &self.credentials.imap_password)
            .await
            .map_err(|e| anyhow!("Login failed: {}", e.0))?;
            
        Ok(session)
    }
    
    pub async fn fetch_emails(&self, folder: &str, limit: Option<usize>) -> Result<Vec<DbEmail>> {
        let mut session = self.connect().await?;
        
        // Select mailbox
        session.select(folder).await?;
        
        // Search for messages
        let uids = session.search("ALL").await?;
        
        // Limit results if requested
        let uids: Vec<u32> = if let Some(limit) = limit {
            uids.into_iter().rev().take(limit).collect()
        } else {
            uids
        };
        
        let mut emails = Vec::new();
        
        // Fetch each message
        for uid in uids {
            let fetches = session
                .fetch(uid.to_string(), "(FLAGS RFC822)")
                .await?;
                
            for fetch in fetches.iter() {
                if let Some(body) = fetch.body() {
                    let email = self.parse_email(body, fetch.flags(), folder)?;
                    emails.push(email);
                }
            }
        }
        
        session.logout().await?;
        Ok(emails)
    }
    
    fn parse_email(&self, raw: &[u8], flags: &[Flag], folder: &str) -> Result<DbEmail> {
        let parsed = mail_parser::MessageParser::default()
            .parse(raw)
            .ok_or_else(|| anyhow!("Failed to parse email"))?;
            
        // Extract fields
        let from = parsed.from()
            .and_then(|addrs| addrs.first())
            .and_then(|addr| addr.address())
            .unwrap_or("unknown")
            .to_string();
            
        let subject = parsed.subject().unwrap_or("(No Subject)").to_string();
        
        let body = parsed.body_text(0)
            .or_else(|| parsed.body_html(0))
            .unwrap_or("")
            .to_string();
            
        // Determine status from flags
        let status = if flags.iter().any(|f| matches!(f, Flag::Seen)) {
            DbEmailStatus::Read
        } else {
            DbEmailStatus::Unread
        };
        
        let is_flagged = flags.iter().any(|f| matches!(f, Flag::Flagged));
        
        Ok(DbEmail {
            id: 0,
            from_address: from,
            subject,
            body: body.clone(),
            preview: body.chars().take(150).collect(),
            // ... other fields
        })
    }
}
```

### Phase 3: SMTP Client Implementation

Replace the stub `SmtpClient`:

```rust
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message,
    transport::smtp::authentication::Credentials as SmtpCreds,
};

pub struct SmtpClient {
    credentials: Credentials,
}

impl SmtpClient {
    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        // Build email message
        let email = Message::builder()
            .from(self.credentials.smtp_username.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .body(body.to_string())?;
            
        // Configure SMTP transport
        let creds = SmtpCreds::new(
            self.credentials.smtp_username.clone(),
            self.credentials.smtp_password.clone(),
        );
        
        let mailer = AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(
            &self.credentials.smtp_server
        )?
        .credentials(creds)
        .port(self.credentials.smtp_port)
        .build();
        
        // Send email
        mailer.send(email).await?;
        Ok(())
    }
}
```

### Phase 4: Folder Management

Add folder operations to `ImapClient`:

```rust
impl ImapClient {
    pub async fn list_folders(&self) -> Result<Vec<String>> {
        let mut session = self.connect().await?;
        let mailboxes = session.list(Some(""), Some("*")).await?;
        
        let folders = mailboxes
            .iter()
            .map(|mb| mb.name().to_string())
            .collect();
            
        session.logout().await?;
        Ok(folders)
    }
    
    pub async fn create_folder(&self, name: &str) -> Result<()> {
        let mut session = self.connect().await?;
        session.create(name).await?;
        session.logout().await?;
        Ok(())
    }
    
    pub async fn delete_folder(&self, name: &str) -> Result<()> {
        let mut session = self.connect().await?;
        session.delete(name).await?;
        session.logout().await?;
        Ok(())
    }
}
```

### Phase 5: Message Operations

Add message manipulation methods:

```rust
impl ImapClient {
    pub async fn mark_seen(&self, folder: &str, uid: u32, seen: bool) -> Result<()> {
        let mut session = self.connect().await?;
        session.select(folder).await?;
        
        let flag = if seen { "+FLAGS" } else { "-FLAGS" };
        session.store(uid.to_string(), format!("{} (\\Seen)", flag)).await?;
        
        session.logout().await?;
        Ok(())
    }
    
    pub async fn flag_message(&self, folder: &str, uid: u32, flagged: bool) -> Result<()> {
        let mut session = self.connect().await?;
        session.select(folder).await?;
        
        let flag = if flagged { "+FLAGS" } else { "-FLAGS" };
        session.store(uid.to_string(), format!("{} (\\Flagged)", flag)).await?;
        
        session.logout().await?;
        Ok(())
    }
    
    pub async fn move_message(&self, from: &str, uid: u32, to: &str) -> Result<()> {
        let mut session = self.connect().await?;
        session.select(from).await?;
        
        // Copy to destination
        session.copy(uid.to_string(), to).await?;
        
        // Mark original as deleted
        session.store(uid.to_string(), "+FLAGS (\\Deleted)").await?;
        
        // Expunge to actually delete
        session.expunge().await?;
        
        session.logout().await?;
        Ok(())
    }
}
```

### Phase 6: OAuth2 Support

For Gmail and Outlook, implement OAuth2:

```rust
// Add oauth2 dependency
// oauth2 = "4.4"

use oauth2::{
    AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
    basic::BasicClient, AuthorizationCode, TokenResponse,
};

pub struct OAuth2Provider {
    client_id: String,
    client_secret: String,
    auth_url: String,
    token_url: String,
}

impl OAuth2Provider {
    pub fn gmail() -> Self {
        Self {
            client_id: std::env::var("GMAIL_CLIENT_ID").unwrap(),
            client_secret: std::env::var("GMAIL_CLIENT_SECRET").unwrap(),
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
        }
    }
    
    pub async fn get_access_token(&self, auth_code: &str) -> Result<String> {
        let client = BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            AuthUrl::new(self.auth_url.clone())?,
            Some(TokenUrl::new(self.token_url.clone())?),
        );
        
        let token = client
            .exchange_code(AuthorizationCode::new(auth_code.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await?;
            
        Ok(token.access_token().secret().clone())
    }
}
```

### Phase 7: Attachment Handling

Add attachment support:

```rust
impl ImapClient {
    pub async fn fetch_attachment(&self, folder: &str, uid: u32, part_id: &str) -> Result<Vec<u8>> {
        let mut session = self.connect().await?;
        session.select(folder).await?;
        
        let fetch = session
            .fetch(uid.to_string(), format!("BODY[{}]", part_id))
            .await?;
            
        // Extract attachment data from fetch result
        let data = fetch
            .iter()
            .next()
            .and_then(|f| f.body())
            .ok_or_else(|| anyhow!("Attachment not found"))?;
            
        session.logout().await?;
        Ok(data.to_vec())
    }
}

impl SmtpClient {
    pub async fn send_with_attachments(
        &self,
        to: &str,
        subject: &str,
        body: &str,
        attachments: Vec<(String, Vec<u8>)>,
    ) -> Result<()> {
        use lettre::message::{header, MultiPart, SinglePart};
        
        let mut multipart = MultiPart::mixed()
            .singlepart(SinglePart::plain(body.to_string()));
            
        for (filename, data) in attachments {
            let attachment = SinglePart::builder()
                .header(header::ContentType::parse("application/octet-stream")?)
                .header(header::ContentDisposition::attachment(&filename))
                .body(data);
            multipart = multipart.singlepart(attachment);
        }
        
        let email = Message::builder()
            .from(self.credentials.smtp_username.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .multipart(multipart)?;
            
        // Send as before...
    }
}
```

## Testing Strategy

### Unit Tests
- Test each IMAP/SMTP method independently
- Mock IMAP/SMTP servers for testing
- Test error handling and edge cases

### Integration Tests
- Test against real email providers (using test accounts)
- Test OAuth2 flow end-to-end
- Test attachment upload/download
- Test folder operations

### Manual Testing Checklist
- [ ] Connect to Gmail via IMAP
- [ ] Connect to Outlook via IMAP
- [ ] Fetch emails from inbox
- [ ] Send email via SMTP
- [ ] Move email between folders
- [ ] Mark email as read/unread
- [ ] Flag/unflag emails
- [ ] Download attachments
- [ ] Send email with attachments
- [ ] Apply inbox rules automatically
- [ ] Search emails on server
- [ ] Handle connection failures gracefully

## Security Considerations

1. **TLS/SSL**: Always use TLS for IMAP (port 993) and SMTP (port 587)
2. **Credentials**: Never log passwords or access tokens
3. **OAuth2**: Prefer OAuth2 over password authentication when available
4. **Token Storage**: Store OAuth2 tokens securely using keyring
5. **Connection Pooling**: Reuse connections but implement timeouts
6. **Rate Limiting**: Respect server rate limits to avoid being blocked
7. **Input Validation**: Sanitize all user input before sending to server

## Performance Optimization

1. **Connection Pooling**: Maintain persistent IMAP connections
2. **Batch Operations**: Fetch multiple emails in a single request
3. **Partial Fetch**: Fetch headers first, bodies on demand
4. **Background Sync**: Use background tasks for periodic email sync
5. **Caching**: Cache folder list and email metadata locally
6. **Incremental Sync**: Only fetch new emails since last sync

## Error Handling

Implement robust error handling for:
- Network failures
- Authentication errors
- Server timeouts
- Invalid credentials
- Mailbox not found
- Message not found
- Rate limiting
- Server maintenance

## Future Enhancements

- **Push Notifications**: IMAP IDLE for real-time updates
- **Offline Mode**: Queue operations when offline
- **Email Threading**: Group related emails
- **Rich Text Editing**: HTML email composition
- **Address Book**: Contact management
- **Calendar Integration**: Meeting invites
- **Encryption**: PGP/S/MIME support
- **Unified Inbox**: Multiple accounts in one view

## Resources

- [async-imap Documentation](https://docs.rs/async-imap)
- [lettre Documentation](https://docs.rs/lettre)
- [IMAP RFC 3501](https://tools.ietf.org/html/rfc3501)
- [SMTP RFC 5321](https://tools.ietf.org/html/rfc5321)
- [OAuth 2.0 RFC 6749](https://tools.ietf.org/html/rfc6749)
- [Gmail API OAuth Guide](https://developers.google.com/gmail/imap/xoauth2-protocol)
- [Outlook OAuth Guide](https://learn.microsoft.com/en-us/exchange/client-developer/legacy-protocols/how-to-authenticate-an-imap-pop-smtp-application-by-using-oauth)

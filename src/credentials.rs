use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result, anyhow};
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{PasswordHash, PasswordVerifier, SaltString};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Service name for keyring storage
const SERVICE_NAME: &str = "tume-email-client";

/// User identifier for keyring storage
const USERNAME: &str = "default";

/// Represents email server credentials
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct Credentials {
    pub imap_server: String,
    pub imap_port: u16,
    pub imap_username: String,
    pub imap_password: String,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
}

/// Backend type for credentials storage
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageBackend {
    /// System keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)
    SystemKeyring,
    /// Encrypted file with master password
    EncryptedFile,
}

impl StorageBackend {
    pub fn as_str(&self) -> &str {
        match self {
            StorageBackend::SystemKeyring => "System Keyring",
            StorageBackend::EncryptedFile => "Encrypted File",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            StorageBackend::SystemKeyring => {
                "Credentials stored in your system's secure keyring (Keychain/Credential Manager/Secret Service)"
            }
            StorageBackend::EncryptedFile => {
                "Credentials stored in an encrypted file at ~/.local/share/tume/credentials.enc, protected by your master password"
            }
        }
    }
}

/// Credentials manager with hybrid storage support
pub struct CredentialsManager {
    backend: StorageBackend,
    file_path: PathBuf,
}

/// Encrypted credentials file structure
#[derive(Serialize, Deserialize)]
struct EncryptedData {
    /// Salt for key derivation (base64)
    salt: String,
    /// Nonce for AES-GCM (base64)
    nonce: String,
    /// Encrypted credentials (base64)
    ciphertext: String,
    /// Password verification hash (PHC string format)
    password_hash: String,
}

impl CredentialsManager {
    /// Create a new credentials manager with automatic backend detection
    pub fn new() -> Self {
        let backend = Self::detect_available_backend();
        let file_path = Self::default_file_path();
        
        let mut debug_log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/tume_debug.log")
            .ok();
        
        if let Some(ref mut log) = debug_log {
            use std::io::Write;
            let _ = writeln!(log, "\n=== CredentialsManager::new() ===");
            let _ = writeln!(log, "Detected backend: {:?}", backend);
            let _ = writeln!(log, "File path: {:?}", file_path);
        }
        
        Self { backend, file_path }
    }

    /// Create a credentials manager with a specific backend
    pub fn with_backend(backend: StorageBackend) -> Self {
        let file_path = Self::default_file_path();
        Self { backend, file_path }
    }

    /// Get the currently active backend
    pub fn backend(&self) -> StorageBackend {
        self.backend
    }

    /// Detect which storage backend is available
    fn detect_available_backend() -> StorageBackend {
        // Try to check if system keyring is available
        if Self::is_keyring_available() {
            StorageBackend::SystemKeyring
        } else {
            StorageBackend::EncryptedFile
        }
    }

    /// Check if system keyring is available
    fn is_keyring_available() -> bool {
        // Try a test operation to see if keyring is available
        match keyring::Entry::new(SERVICE_NAME, "test-availability") {
            Ok(entry) => {
                // Try to set and delete a test value
                if entry.set_password("test").is_ok() {
                    let _ = entry.delete_credential();
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    /// Get the default file path for encrypted credentials
    fn default_file_path() -> PathBuf {
        let mut path = dirs::home_dir().expect("Could not find home directory");
        path.push(".local");
        path.push("share");
        path.push("tume");
        path.push("credentials.enc");
        path
    }

    /// Check if credentials exist in the current backend
    pub fn credentials_exist(&self) -> bool {
        let mut debug_log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/tume_debug.log")
            .ok();
        
        let result = match self.backend {
            StorageBackend::SystemKeyring => self.keyring_credentials_exist(),
            StorageBackend::EncryptedFile => {
                let exists = self.file_path.exists();
                if let Some(ref mut log) = debug_log {
                    use std::io::Write;
                    let _ = writeln!(log, "Checking encrypted file credentials: {:?} exists = {}", self.file_path, exists);
                }
                exists
            },
        };
        
        if let Some(ref mut log) = debug_log {
            use std::io::Write;
            let _ = writeln!(log, "credentials_exist({:?}) = {}", self.backend, result);
        }
        
        result
    }

    /// Check if credentials exist in keyring
    fn keyring_credentials_exist(&self) -> bool {
        match keyring::Entry::new(SERVICE_NAME, USERNAME) {
            Ok(entry) => entry.get_password().is_ok(),
            Err(_) => false,
        }
    }

    /// Save credentials using the current backend
    pub fn save_credentials(&self, credentials: &Credentials, master_password: Option<&str>) -> Result<()> {
        let mut debug_log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/tume_debug.log")
            .ok();
        
        if let Some(ref mut log) = debug_log {
            use std::io::Write;
            let _ = writeln!(log, "\n=== save_credentials() called ===");
            let _ = writeln!(log, "Backend: {:?}", self.backend);
            let _ = writeln!(log, "File path: {:?}", self.file_path);
            let _ = writeln!(log, "Master password provided: {}", master_password.is_some());
        }
        
        let result = match self.backend {
            StorageBackend::SystemKeyring => self.save_to_keyring(credentials),
            StorageBackend::EncryptedFile => {
                let password = master_password
                    .ok_or_else(|| anyhow!("Master password required for encrypted file storage"))?;
                self.save_to_encrypted_file(credentials, password)
            }
        };
        
        if let Some(ref mut log) = debug_log {
            use std::io::Write;
            match &result {
                Ok(_) => {
                    let _ = writeln!(log, "Credentials saved successfully");
                    let _ = writeln!(log, "File exists after save: {}", self.file_path.exists());
                },
                Err(e) => {
                    let _ = writeln!(log, "Failed to save credentials: {}", e);
                }
            }
        }
        
        result
    }

    /// Load credentials using the current backend
    pub fn load_credentials(&self, master_password: Option<&str>) -> Result<Credentials> {
        match self.backend {
            StorageBackend::SystemKeyring => self.load_from_keyring(),
            StorageBackend::EncryptedFile => {
                let password = master_password
                    .ok_or_else(|| anyhow!("Master password required for encrypted file storage"))?;
                self.load_from_encrypted_file(password)
            }
        }
    }

    /// Delete credentials from the current backend
    pub fn delete_credentials(&self) -> Result<()> {
        match self.backend {
            StorageBackend::SystemKeyring => self.delete_from_keyring(),
            StorageBackend::EncryptedFile => self.delete_encrypted_file(),
        }
    }

    /// Verify master password (for encrypted file backend)
    pub fn verify_master_password(&self, password: &str) -> Result<bool> {
        if self.backend != StorageBackend::EncryptedFile {
            return Err(anyhow!("Password verification only available for encrypted file backend"));
        }

        if !self.file_path.exists() {
            return Ok(false);
        }

        let encrypted_data: EncryptedData = {
            let json = fs::read_to_string(&self.file_path)
                .context("Failed to read encrypted credentials file")?;
            serde_json::from_str(&json).context("Failed to parse encrypted credentials file")?
        };

        // Verify password against stored hash
        let parsed_hash = PasswordHash::new(&encrypted_data.password_hash)
            .map_err(|e| anyhow!("Failed to parse password hash: {}", e))?;
        
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Migrate credentials from one backend to another
    pub fn migrate_to(&self, target_backend: StorageBackend, 
                      current_master_password: Option<&str>,
                      new_master_password: Option<&str>) -> Result<()> {
        // Load credentials from current backend
        let credentials = self.load_credentials(current_master_password)?;
        
        // Create a new manager with target backend
        let target_manager = Self::with_backend(target_backend);
        
        // Save to target backend
        target_manager.save_credentials(&credentials, new_master_password)?;
        
        // Delete from current backend
        self.delete_credentials()?;
        
        Ok(())
    }

    // ============ System Keyring Operations ============

    fn save_to_keyring(&self, credentials: &Credentials) -> Result<()> {
        let entry = keyring::Entry::new(SERVICE_NAME, USERNAME)
            .context("Failed to create keyring entry")?;
        
        // Serialize credentials to JSON
        let json = serde_json::to_string(credentials)
            .context("Failed to serialize credentials")?;
        
        entry.set_password(&json)
            .context("Failed to save credentials to keyring")?;
        
        Ok(())
    }

    fn load_from_keyring(&self) -> Result<Credentials> {
        let entry = keyring::Entry::new(SERVICE_NAME, USERNAME)
            .context("Failed to create keyring entry")?;
        
        let json = entry.get_password()
            .context("Failed to retrieve credentials from keyring. Please configure credentials first.")?;
        
        let credentials: Credentials = serde_json::from_str(&json)
            .context("Failed to parse credentials from keyring")?;
        
        Ok(credentials)
    }

    fn delete_from_keyring(&self) -> Result<()> {
        let entry = keyring::Entry::new(SERVICE_NAME, USERNAME)
            .context("Failed to create keyring entry")?;
        
        entry.delete_credential()
            .context("Failed to delete credentials from keyring")?;
        
        Ok(())
    }

    // ============ Encrypted File Operations ============

    fn save_to_encrypted_file(&self, credentials: &Credentials, master_password: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create credentials directory")?;
        }

        // Generate salt for key derivation
        let salt = SaltString::generate(&mut OsRng);
        
        // Hash password for verification
        let password_hash = Argon2::default()
            .hash_password(master_password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash master password: {}", e))?
            .to_string();

        // Derive encryption key from password
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(master_password.as_bytes(), salt.as_str().as_bytes(), &mut key)
            .map_err(|e| anyhow!("Failed to derive encryption key: {}", e))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Serialize credentials
        let json = serde_json::to_string(credentials)
            .context("Failed to serialize credentials")?;

        // Encrypt credentials
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;
        let ciphertext = cipher
            .encrypt(nonce, json.as_bytes())
            .map_err(|_| anyhow!("Failed to encrypt credentials"))?;

        // Zeroize sensitive data
        key.zeroize();

        // Create encrypted data structure
        let encrypted_data = EncryptedData {
            salt: base64::encode(salt.as_str().as_bytes()),
            nonce: base64::encode(&nonce_bytes),
            ciphertext: base64::encode(&ciphertext),
            password_hash,
        };

        // Write to file
        let json = serde_json::to_string(&encrypted_data)
            .context("Failed to serialize encrypted data")?;
        fs::write(&self.file_path, json)
            .context("Failed to write encrypted credentials file")?;

        Ok(())
    }

    fn load_from_encrypted_file(&self, master_password: &str) -> Result<Credentials> {
        if !self.file_path.exists() {
            return Err(anyhow!("Credentials file not found. Please configure credentials first."));
        }

        // Read encrypted data
        let encrypted_data: EncryptedData = {
            let json = fs::read_to_string(&self.file_path)
                .context("Failed to read encrypted credentials file")?;
            serde_json::from_str(&json)
                .context("Failed to parse encrypted credentials file")?
        };

        // Verify password
        let parsed_hash = PasswordHash::new(&encrypted_data.password_hash)
            .map_err(|e| anyhow!("Failed to parse password hash: {}", e))?;
        Argon2::default()
            .verify_password(master_password.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow!("Incorrect master password"))?;

        // Decode base64 data
        let salt_bytes = base64::decode(&encrypted_data.salt)
            .context("Failed to decode salt")?;
        let nonce_bytes = base64::decode(&encrypted_data.nonce)
            .context("Failed to decode nonce")?;
        let ciphertext = base64::decode(&encrypted_data.ciphertext)
            .context("Failed to decode ciphertext")?;

        // Derive decryption key
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(master_password.as_bytes(), &salt_bytes, &mut key)
            .map_err(|e| anyhow!("Failed to derive decryption key: {}", e))?;

        // Decrypt credentials
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| anyhow!("Failed to decrypt credentials"))?;

        // Zeroize key
        key.zeroize();

        // Parse credentials
        let json = String::from_utf8(plaintext)
            .context("Failed to parse decrypted data as UTF-8")?;
        let credentials: Credentials = serde_json::from_str(&json)
            .context("Failed to parse credentials JSON")?;

        Ok(credentials)
    }

    fn delete_encrypted_file(&self) -> Result<()> {
        if self.file_path.exists() {
            fs::remove_file(&self.file_path)
                .context("Failed to delete encrypted credentials file")?;
        }
        Ok(())
    }
}

// Add base64 encoding/decoding helpers
mod base64 {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;

    pub fn encode(data: &[u8]) -> String {
        STANDARD.encode(data)
    }

    pub fn decode(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
        STANDARD.decode(data)
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
            imap_password: "imap_secret".to_string(),
            smtp_server: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_username: "user@example.com".to_string(),
            smtp_password: "smtp_secret".to_string(),
        }
    }

    #[test]
    fn test_encrypted_file_save_and_load() {
        // Create a temporary file path
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_tume_creds_{}.enc", std::process::id()));
        
        // Clean up if exists
        let _ = std::fs::remove_file(&file_path);

        let mut manager = CredentialsManager::with_backend(StorageBackend::EncryptedFile);
        manager.file_path = file_path.clone();

        let credentials = create_test_credentials();
        let master_password = "test-master-password-123";

        // Save credentials
        manager.save_credentials(&credentials, Some(master_password))
            .expect("Failed to save credentials");

        // Verify file exists
        assert!(manager.credentials_exist());

        // Load credentials
        let loaded = manager.load_credentials(Some(master_password))
            .expect("Failed to load credentials");

        assert_eq!(loaded.imap_server, credentials.imap_server);
        assert_eq!(loaded.imap_password, credentials.imap_password);
        assert_eq!(loaded.smtp_server, credentials.smtp_server);

        // Test wrong password
        let wrong_result = manager.load_credentials(Some("wrong-password"));
        assert!(wrong_result.is_err());

        // Clean up
        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_password_verification() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_tume_verify_{}.enc", std::process::id()));
        let _ = std::fs::remove_file(&file_path);

        let mut manager = CredentialsManager::with_backend(StorageBackend::EncryptedFile);
        manager.file_path = file_path.clone();

        let credentials = create_test_credentials();
        let master_password = "correct-password";

        manager.save_credentials(&credentials, Some(master_password))
            .expect("Failed to save credentials");

        // Verify correct password
        assert!(manager.verify_master_password(master_password).unwrap());

        // Verify wrong password
        assert!(!manager.verify_master_password("wrong-password").unwrap());

        // Clean up
        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_delete_credentials() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_tume_delete_{}.enc", std::process::id()));
        let _ = std::fs::remove_file(&file_path);

        let mut manager = CredentialsManager::with_backend(StorageBackend::EncryptedFile);
        manager.file_path = file_path.clone();

        let credentials = create_test_credentials();
        manager.save_credentials(&credentials, Some("password"))
            .expect("Failed to save credentials");

        assert!(manager.credentials_exist());

        manager.delete_credentials()
            .expect("Failed to delete credentials");

        assert!(!manager.credentials_exist());
    }

    #[test]
    fn test_backend_detection() {
        let manager = CredentialsManager::new();
        // Should select either keyring or encrypted file
        assert!(
            manager.backend() == StorageBackend::SystemKeyring 
            || manager.backend() == StorageBackend::EncryptedFile
        );
    }
}

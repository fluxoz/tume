/// Email provider presets for common services
/// This module provides pre-configured IMAP and SMTP settings for popular email providers

use serde::{Deserialize, Serialize};

/// Email provider preset with IMAP and SMTP configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmailProvider {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub imap_server: &'static str,
    pub imap_port: u16,
    pub imap_security: SecurityType,
    pub smtp_server: &'static str,
    pub smtp_port: u16,
    pub smtp_security: SecurityType,
    pub username_hint: &'static str,
}

/// Security/encryption type for connections
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SecurityType {
    /// SSL/TLS (implicit encryption)
    Tls,
    /// STARTTLS (explicit encryption)
    StartTls,
}

impl EmailProvider {
    /// Get all available provider presets
    pub fn all() -> Vec<EmailProvider> {
        vec![
            Self::gmail(),
            Self::outlook(),
            Self::yahoo(),
            Self::protonmail(),
            Self::icloud(),
            Self::fastmail(),
            Self::aol(),
            Self::zoho(),
            Self::gmx(),
            Self::mailcom(),
            Self::yandex(),
            Self::custom(),
        ]
    }

    /// Get a provider by its ID
    pub fn by_id(id: &str) -> Option<EmailProvider> {
        Self::all().into_iter().find(|p| p.id == id)
    }

    /// Gmail configuration
    pub fn gmail() -> Self {
        EmailProvider {
            id: "gmail",
            name: "Gmail",
            description: "Google Gmail - Requires app-specific password if 2FA is enabled",
            imap_server: "imap.gmail.com",
            imap_port: 993,
            imap_security: SecurityType::Tls,
            smtp_server: "smtp.gmail.com",
            smtp_port: 587,
            smtp_security: SecurityType::StartTls,
            username_hint: "your.email@gmail.com",
        }
    }

    /// Microsoft Outlook/Office 365 configuration
    pub fn outlook() -> Self {
        EmailProvider {
            id: "outlook",
            name: "Outlook / Office 365",
            description: "Microsoft Outlook.com, Hotmail, Live, and Office 365 accounts",
            imap_server: "outlook.office365.com",
            imap_port: 993,
            imap_security: SecurityType::Tls,
            smtp_server: "smtp.office365.com",
            smtp_port: 587,
            smtp_security: SecurityType::StartTls,
            username_hint: "your.email@outlook.com",
        }
    }

    /// Yahoo Mail configuration
    pub fn yahoo() -> Self {
        EmailProvider {
            id: "yahoo",
            name: "Yahoo Mail",
            description: "Yahoo Mail - Requires app-specific password",
            imap_server: "imap.mail.yahoo.com",
            imap_port: 993,
            imap_security: SecurityType::Tls,
            smtp_server: "smtp.mail.yahoo.com",
            smtp_port: 587,
            smtp_security: SecurityType::StartTls,
            username_hint: "your.email@yahoo.com",
        }
    }

    /// ProtonMail Bridge configuration
    pub fn protonmail() -> Self {
        EmailProvider {
            id: "protonmail",
            name: "ProtonMail Bridge",
            description: "ProtonMail - Requires ProtonMail Bridge running locally",
            imap_server: "127.0.0.1",
            imap_port: 1143,
            imap_security: SecurityType::StartTls,
            smtp_server: "127.0.0.1",
            smtp_port: 1025,
            smtp_security: SecurityType::StartTls,
            username_hint: "your.email@proton.me",
        }
    }

    /// iCloud Mail configuration
    pub fn icloud() -> Self {
        EmailProvider {
            id: "icloud",
            name: "iCloud Mail",
            description: "Apple iCloud Mail - Requires app-specific password",
            imap_server: "imap.mail.me.com",
            imap_port: 993,
            imap_security: SecurityType::Tls,
            smtp_server: "smtp.mail.me.com",
            smtp_port: 587,
            smtp_security: SecurityType::StartTls,
            username_hint: "your.email@icloud.com",
        }
    }

    /// Fastmail configuration
    pub fn fastmail() -> Self {
        EmailProvider {
            id: "fastmail",
            name: "Fastmail",
            description: "Fastmail - Privacy-focused email service",
            imap_server: "imap.fastmail.com",
            imap_port: 993,
            imap_security: SecurityType::Tls,
            smtp_server: "smtp.fastmail.com",
            smtp_port: 587,
            smtp_security: SecurityType::StartTls,
            username_hint: "your.email@fastmail.com",
        }
    }

    /// AOL Mail configuration
    pub fn aol() -> Self {
        EmailProvider {
            id: "aol",
            name: "AOL Mail",
            description: "AOL Mail - Requires app-specific password",
            imap_server: "imap.aol.com",
            imap_port: 993,
            imap_security: SecurityType::Tls,
            smtp_server: "smtp.aol.com",
            smtp_port: 587,
            smtp_security: SecurityType::StartTls,
            username_hint: "your.email@aol.com",
        }
    }

    /// Zoho Mail configuration
    pub fn zoho() -> Self {
        EmailProvider {
            id: "zoho",
            name: "Zoho Mail",
            description: "Zoho Mail - Business and personal email",
            imap_server: "imap.zoho.com",
            imap_port: 993,
            imap_security: SecurityType::Tls,
            smtp_server: "smtp.zoho.com",
            smtp_port: 587,
            smtp_security: SecurityType::StartTls,
            username_hint: "your.email@zoho.com",
        }
    }

    /// GMX Mail configuration
    pub fn gmx() -> Self {
        EmailProvider {
            id: "gmx",
            name: "GMX Mail",
            description: "GMX Mail - Free email service",
            imap_server: "imap.gmx.com",
            imap_port: 993,
            imap_security: SecurityType::Tls,
            smtp_server: "smtp.gmx.com",
            smtp_port: 587,
            smtp_security: SecurityType::StartTls,
            username_hint: "your.email@gmx.com",
        }
    }

    /// Mail.com configuration
    pub fn mailcom() -> Self {
        EmailProvider {
            id: "mailcom",
            name: "Mail.com",
            description: "Mail.com - Free email with many domain options",
            imap_server: "imap.mail.com",
            imap_port: 993,
            imap_security: SecurityType::Tls,
            smtp_server: "smtp.mail.com",
            smtp_port: 587,
            smtp_security: SecurityType::StartTls,
            username_hint: "your.email@mail.com",
        }
    }

    /// Yandex Mail configuration
    pub fn yandex() -> Self {
        EmailProvider {
            id: "yandex",
            name: "Yandex Mail",
            description: "Yandex Mail - Russian email service",
            imap_server: "imap.yandex.com",
            imap_port: 993,
            imap_security: SecurityType::Tls,
            smtp_server: "smtp.yandex.com",
            smtp_port: 587,
            smtp_security: SecurityType::StartTls,
            username_hint: "your.email@yandex.com",
        }
    }

    /// Custom/Manual configuration
    pub fn custom() -> Self {
        EmailProvider {
            id: "custom",
            name: "Custom (Other Provider)",
            description: "Manually configure IMAP and SMTP settings",
            imap_server: "",
            imap_port: 993,
            imap_security: SecurityType::Tls,
            smtp_server: "",
            smtp_port: 587,
            smtp_security: SecurityType::StartTls,
            username_hint: "your.email@domain.com",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_providers_are_unique() {
        let providers = EmailProvider::all();
        let mut ids: Vec<&str> = providers.iter().map(|p| p.id).collect();
        ids.sort();
        ids.dedup();
        
        // Should have same length after dedup (no duplicates)
        assert_eq!(ids.len(), providers.len());
    }

    #[test]
    fn test_provider_by_id() {
        assert!(EmailProvider::by_id("gmail").is_some());
        assert!(EmailProvider::by_id("outlook").is_some());
        assert!(EmailProvider::by_id("custom").is_some());
        assert!(EmailProvider::by_id("nonexistent").is_none());
    }

    #[test]
    fn test_gmail_config() {
        let gmail = EmailProvider::gmail();
        assert_eq!(gmail.id, "gmail");
        assert_eq!(gmail.imap_server, "imap.gmail.com");
        assert_eq!(gmail.imap_port, 993);
        assert_eq!(gmail.smtp_server, "smtp.gmail.com");
        assert_eq!(gmail.smtp_port, 587);
    }

    #[test]
    fn test_protonmail_uses_local_bridge() {
        let proton = EmailProvider::protonmail();
        assert_eq!(proton.imap_server, "127.0.0.1");
        assert_eq!(proton.smtp_server, "127.0.0.1");
    }

    #[test]
    fn test_custom_provider_empty_servers() {
        let custom = EmailProvider::custom();
        assert_eq!(custom.imap_server, "");
        assert_eq!(custom.smtp_server, "");
    }
}

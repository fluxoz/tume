use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Represents a single email account/mailbox
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub name: String,
    pub email: String,
    pub provider: String,
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub display_order: Option<i64>,
}

/// Keybindings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    #[serde(default = "default_switch_account_keys")]
    pub switch_account: Vec<String>,
    #[serde(default = "default_next_account_key")]
    pub next_account: String,
    #[serde(default = "default_prev_account_key")]
    pub prev_account: String,
    #[serde(default = "default_mailbox_picker_key")]
    pub mailbox_picker: String,
    #[serde(default = "default_add_account_key")]
    pub add_account: String,
}

fn default_switch_account_keys() -> Vec<String> {
    vec!["1".to_string(), "2".to_string(), "3".to_string(), 
         "4".to_string(), "5".to_string(), "6".to_string(),
         "7".to_string(), "8".to_string(), "9".to_string()]
}

fn default_next_account_key() -> String {
    "]".to_string()
}

fn default_prev_account_key() -> String {
    "[".to_string()
}

fn default_mailbox_picker_key() -> String {
    "M".to_string()
}

fn default_add_account_key() -> String {
    "A".to_string()
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            switch_account: default_switch_account_keys(),
            next_account: default_next_account_key(),
            prev_account: default_prev_account_key(),
            mailbox_picker: default_mailbox_picker_key(),
            add_account: default_add_account_key(),
        }
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub accounts: HashMap<String, Account>,
    #[serde(default)]
    pub keybindings: Keybindings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            accounts: HashMap::new(),
            keybindings: Keybindings::default(),
        }
    }
}

impl Config {
    /// Load configuration from file or return default config
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        
        let config: Config = toml::from_str(&contents)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        fs::write(&config_path, contents)
            .context("Failed to write config file")?;
        
        Ok(())
    }

    /// Get config file path (~/.config/tume/config.toml)
    pub fn config_path() -> Result<PathBuf> {
        let mut path = dirs::home_dir()
            .context("Could not find home directory")?;
        path.push(".config");
        path.push("tume");
        path.push("config.toml");
        Ok(path)
    }

    /// Get accounts as a sorted vector
    pub fn get_accounts_sorted(&self) -> Vec<Account> {
        let mut accounts: Vec<Account> = self.accounts.values().cloned().collect();
        accounts.sort_by_key(|a| a.display_order.unwrap_or(999));
        accounts
    }

    /// Get default account if one is set
    pub fn get_default_account(&self) -> Option<Account> {
        self.accounts.values().find(|a| a.default).cloned()
    }

    /// Add or update an account
    pub fn set_account(&mut self, key: String, account: Account) {
        self.accounts.insert(key, account);
    }

    /// Remove an account
    pub fn remove_account(&mut self, key: &str) -> Option<Account> {
        self.accounts.remove(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.accounts.len(), 0);
        assert_eq!(config.keybindings.next_account, "]");
        assert_eq!(config.keybindings.prev_account, "[");
    }

    #[test]
    fn test_config_serialization() {
        let mut config = Config::default();
        config.set_account(
            "work".to_string(),
            Account {
                name: "Work Email".to_string(),
                email: "work@example.com".to_string(),
                provider: "gmail".to_string(),
                default: true,
                color: Some("blue".to_string()),
                display_order: Some(1),
            },
        );

        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("work@example.com"));
        assert!(toml_str.contains("Work Email"));

        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.accounts.len(), 1);
        assert!(parsed.accounts.contains_key("work"));
    }

    #[test]
    fn test_get_accounts_sorted() {
        let mut config = Config::default();
        config.set_account(
            "personal".to_string(),
            Account {
                name: "Personal".to_string(),
                email: "me@example.com".to_string(),
                provider: "gmail".to_string(),
                default: false,
                color: None,
                display_order: Some(2),
            },
        );
        config.set_account(
            "work".to_string(),
            Account {
                name: "Work".to_string(),
                email: "work@example.com".to_string(),
                provider: "outlook".to_string(),
                default: true,
                color: None,
                display_order: Some(1),
            },
        );

        let sorted = config.get_accounts_sorted();
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].email, "work@example.com");
        assert_eq!(sorted[1].email, "me@example.com");
    }

    #[test]
    fn test_get_default_account() {
        let mut config = Config::default();
        config.set_account(
            "work".to_string(),
            Account {
                name: "Work".to_string(),
                email: "work@example.com".to_string(),
                provider: "gmail".to_string(),
                default: true,
                color: None,
                display_order: Some(1),
            },
        );

        let default = config.get_default_account();
        assert!(default.is_some());
        assert_eq!(default.unwrap().email, "work@example.com");
    }
}

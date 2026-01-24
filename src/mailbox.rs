use url::Url;
use crate::email::Email;
use crate::folder::Folder;

#[derive(Clone, Debug)]
pub enum AuthType {
    Password {
        username: String,
        password: String,
    },
    OAuth2 {
        access_token: String,
        refresh_token: String,
        client_id: String,
        client_secret: Option<Url>,
        scopes: Vec<String>,
    },
    ApiKey {
        key: String,
    }
}

#[derive(Clone, Debug)]
pub enum Protocol {
    Imap,
    Jmap,
}

#[derive(Clone, Debug)]
struct ServerConfig {
    pub hostname: String,
    pub port: u16,
    pub protocol: Protocol,
    pub use_tls: bool,
    pub starttls: bool,
    pub accept_invalid_certs: bool,
}

#[derive(Clone, Debug)]
pub struct MailBox {
    pub name: String,
    pub inbox: Vec<Email>,
    pub folders: Vec<Folder>,
    pub auth: AuthType,
    pub imap: Option<ServerConfig>,
    pub smtp: Option<ServerConfig>,
    pub provider_kind: Option<ProviderKind>,
    pub autodiscover: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderKind {
    Gmail,
    Microsoft365,
    OutlookCom,
    ICloud,
    Yahoo,
    Fastmail,
    Proton,
    Zoho,
    Custom,
}

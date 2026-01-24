use blob::Blob;

#[derive(Clone, Debug)]
pub struct Email {
    pub recipients: Vec<String>,
    pub from: String,
    pub bcc: Option<Vec<String>>,
    pub subject: String,
    pub body: String,
    pub attachments: Option<Vec<Blob>>,
}

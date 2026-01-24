use crate::email::Email;

#[derive(Clone, Debug)]
pub struct Folder {
    pub emails: Vec<Email>,
}


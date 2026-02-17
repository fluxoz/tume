use std::fmt::Display;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use chrono::{DateTime, Local};

#[derive(Clone, Debug)]
pub struct Email {
    pub from: String,
    pub subject: String,
    pub unread: bool,
    pub datetime_received: DateTime<Local>,
    pub datetime_read: Option<DateTime<Local>>,
    pub body: String,
}

impl Email {
    pub fn new_test<S>(input: &S) -> Self 
        where 
            S: Display,
            S: AsRef<str>,
    {
        Email {
            from: "test@testing.com".to_owned(),
            subject: input.to_string(),
            unread: true,
            datetime_received: Local::now(),
            datetime_read: None,
            body: "This is a test email for formatting and UI development.".to_owned(),
        }
    }
    pub fn from_slice<S>(items: &[S]) -> Vec<Email> 
        where 
            S: Display,
            S: AsRef<str>,
    {

        let mut email_vec = vec![];
        for i in items {
            email_vec.push(Email::new_test(i));

        }
        email_vec
    }
}



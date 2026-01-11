use std::fmt;

#[derive(Debug, Clone)]
pub struct Email {
    pub from: String,
    pub subject: String,
    pub preview: String,
    pub body: String,
    pub date: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    InboxList,
    EmailDetail,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Delete,
    Archive,
    Reply,
    Compose,
    Forward,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Delete => write!(f, "Delete (d)"),
            Action::Archive => write!(f, "Archive (a)"),
            Action::Reply => write!(f, "Reply (r)"),
            Action::Compose => write!(f, "Compose (c)"),
            Action::Forward => write!(f, "Forward (f)"),
        }
    }
}

pub struct App {
    pub emails: Vec<Email>,
    pub current_view: View,
    pub selected_index: usize,
    pub should_quit: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            emails: Self::mock_emails(),
            current_view: View::InboxList,
            selected_index: 0,
            should_quit: false,
            status_message: None,
        }
    }

    fn mock_emails() -> Vec<Email> {
        vec![
            Email {
                from: "alice@example.com".to_string(),
                subject: "Project Update: Q1 Planning".to_string(),
                preview: "Hi team, I wanted to share some updates on our Q1 planning...".to_string(),
                body: "Hi team,\n\nI wanted to share some updates on our Q1 planning. We've made significant progress on the roadmap and I'd like to schedule a meeting to discuss next steps.\n\nLooking forward to your feedback.\n\nBest regards,\nAlice".to_string(),
                date: "2026-01-10 14:30".to_string(),
            },
            Email {
                from: "bob@example.com".to_string(),
                subject: "Meeting notes from yesterday".to_string(),
                preview: "Here are the notes from our meeting yesterday...".to_string(),
                body: "Here are the notes from our meeting yesterday:\n\n1. Discussed new feature requirements\n2. Reviewed timeline for implementation\n3. Assigned tasks to team members\n\nPlease review and let me know if I missed anything.\n\nBob".to_string(),
                date: "2026-01-10 09:15".to_string(),
            },
            Email {
                from: "notifications@github.com".to_string(),
                subject: "[fluxoz/tume] New issue opened: Create TUI stub".to_string(),
                preview: "A new issue has been opened in your repository...".to_string(),
                body: "A new issue has been opened in your repository fluxoz/tume:\n\nTitle: Create a TUI stub for this project\n\nThis project is meant to be a TUI email client...".to_string(),
                date: "2026-01-09 22:45".to_string(),
            },
            Email {
                from: "charlie@example.com".to_string(),
                subject: "Re: Budget approval request".to_string(),
                preview: "Thanks for submitting the budget request...".to_string(),
                body: "Thanks for submitting the budget request. I've reviewed the numbers and everything looks good. Approved!\n\nCharlie".to_string(),
                date: "2026-01-09 16:20".to_string(),
            },
            Email {
                from: "newsletter@techblog.com".to_string(),
                subject: "Weekly Tech Digest: Rust 1.92 Released".to_string(),
                preview: "This week in tech: Rust 1.92 brings exciting new features...".to_string(),
                body: "This week in tech:\n\n- Rust 1.92 Released with improved compile times\n- New TUI libraries gaining popularity\n- Terminal applications making a comeback\n\nRead more at techblog.com".to_string(),
                date: "2026-01-09 08:00".to_string(),
            },
        ]
    }

    pub fn next_email(&mut self) {
        if !self.emails.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.emails.len() - 1);
        }
    }

    pub fn previous_email(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn open_email(&mut self) {
        if !self.emails.is_empty() && self.current_view == View::InboxList {
            self.current_view = View::EmailDetail;
        }
    }

    pub fn close_email(&mut self) {
        if self.current_view == View::EmailDetail {
            self.current_view = View::InboxList;
        }
    }

    pub fn perform_action(&mut self, action: Action) {
        if self.emails.is_empty() {
            return;
        }

        let email = &self.emails[self.selected_index];
        match action {
            Action::Delete => {
                self.status_message = Some(format!("Deleted email: {}", email.subject));
            }
            Action::Archive => {
                self.status_message = Some(format!("Archived email: {}", email.subject));
            }
            Action::Reply => {
                self.status_message = Some(format!("Replying to: {}", email.from));
            }
            Action::Compose => {
                self.status_message = Some("Composing new email...".to_string());
            }
            Action::Forward => {
                self.status_message = Some(format!("Forwarding email: {}", email.subject));
            }
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn get_selected_email(&self) -> Option<&Email> {
        self.emails.get(self.selected_index)
    }
}

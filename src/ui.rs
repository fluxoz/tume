use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use pulldown_cmark::{Parser, Event as MarkdownEvent, Tag};

use crate::app::{App, View, ComposeMode, ComposeField};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_header(f, chunks[0]);
    
    match app.current_view {
        View::InboxList => render_inbox(f, chunks[1], app),
        View::EmailDetail => render_email_detail(f, chunks[1], app),
        View::Compose => render_compose(f, chunks[1], app),
    }
    
    render_footer(f, chunks[2], app);
}

fn render_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new("TUME - Terminal Email Client")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

fn render_inbox(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .emails
        .iter()
        .enumerate()
        .map(|(i, email)| {
            let style = if i == app.selected_index {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = vec![
                Line::from(vec![
                    Span::styled("From: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&email.from),
                ]),
                Line::from(vec![
                    Span::styled("Subject: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&email.subject),
                ]),
                Line::from(vec![
                    Span::styled("Date: ", Style::default().add_modifier(Modifier::ITALIC)),
                    Span::raw(&email.date),
                ]),
                Line::from(Span::raw(&email.preview)),
                Line::from(""),
            ];

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Inbox (j/k to navigate, Enter to read, q to quit)"),
    );

    f.render_widget(list, area);
}

fn render_email_detail(f: &mut Frame, area: Rect, app: &App) {
    if let Some(email) = app.get_selected_email() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(area);

        // Email metadata
        let metadata = vec![
            Line::from(vec![
                Span::styled("From: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&email.from),
            ]),
            Line::from(vec![
                Span::styled("Subject: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&email.subject),
            ]),
            Line::from(vec![
                Span::styled("Date: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&email.date),
            ]),
        ];

        let metadata_widget = Paragraph::new(metadata)
            .block(Block::default().borders(Borders::ALL).title("Email Details"));
        f.render_widget(metadata_widget, chunks[0]);

        // Email body
        let body = Paragraph::new(email.body.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Message (h to go back)"),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(body, chunks[1]);
    }
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let help_text = match app.current_view {
        View::InboxList => {
            "j/k: Navigate | Enter/l: Read | d: Delete | a: Archive | r: Reply | c: Compose | f: Forward | q: Quit"
        }
        View::EmailDetail => {
            "h/Esc: Back | d: Delete | a: Archive | r: Reply | f: Forward | q: Quit"
        }
        View::Compose => {
            if let Some(ref compose) = app.compose_state {
                match compose.mode {
                    ComposeMode::Normal => "i: Insert | j/k: Navigate fields | p: Preview | Esc/q: Exit",
                    ComposeMode::Insert => "Esc: Normal mode | Type to edit field",
                }
            } else {
                ""
            }
        }
    };

    let text = if let Some(ref msg) = app.status_message {
        vec![
            Line::from(Span::styled(msg, Style::default().fg(Color::Yellow))),
            Line::from(Span::raw(help_text)),
        ]
    } else {
        vec![Line::from(Span::raw(help_text))]
    };

    let footer = Paragraph::new(text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, area);
}

fn render_compose(f: &mut Frame, area: Rect, app: &App) {
    if let Some(ref compose) = app.compose_state {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        // Recipients field
        let recipients_style = if compose.current_field == ComposeField::Recipients {
            if compose.mode == ComposeMode::Insert {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default()
        };
        
        let recipients_text = if compose.recipients.is_empty() && compose.current_field != ComposeField::Recipients {
            Span::styled("<empty>", Style::default().fg(Color::DarkGray))
        } else {
            Span::styled(&compose.recipients, Style::default())
        };
        
        let recipients_widget = Paragraph::new(Line::from(vec![
            Span::styled("To: ", recipients_style),
            recipients_text,
        ]))
        .block(Block::default().borders(Borders::ALL).title(
            if compose.current_field == ComposeField::Recipients && compose.mode == ComposeMode::Insert {
                "Recipients [INSERT]"
            } else if compose.current_field == ComposeField::Recipients {
                "Recipients [NORMAL]"
            } else {
                "Recipients"
            }
        ));
        f.render_widget(recipients_widget, chunks[0]);

        // Subject field
        let subject_style = if compose.current_field == ComposeField::Subject {
            if compose.mode == ComposeMode::Insert {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default()
        };
        
        let subject_text = if compose.subject.is_empty() && compose.current_field != ComposeField::Subject {
            Span::styled("<empty>", Style::default().fg(Color::DarkGray))
        } else {
            Span::styled(&compose.subject, Style::default())
        };
        
        let subject_widget = Paragraph::new(Line::from(vec![
            Span::styled("Subject: ", subject_style),
            subject_text,
        ]))
        .block(Block::default().borders(Borders::ALL).title(
            if compose.current_field == ComposeField::Subject && compose.mode == ComposeMode::Insert {
                "Subject [INSERT]"
            } else if compose.current_field == ComposeField::Subject {
                "Subject [NORMAL]"
            } else {
                "Subject"
            }
        ));
        f.render_widget(subject_widget, chunks[1]);

        // Body field with optional preview
        let body_title = if compose.show_preview {
            if compose.current_field == ComposeField::Body && compose.mode == ComposeMode::Insert {
                "Body [INSERT] - Preview"
            } else if compose.current_field == ComposeField::Body {
                "Body [NORMAL] - Preview"
            } else {
                "Body - Preview"
            }
        } else {
            if compose.current_field == ComposeField::Body && compose.mode == ComposeMode::Insert {
                "Body [INSERT]"
            } else if compose.current_field == ComposeField::Body {
                "Body [NORMAL]"
            } else {
                "Body"
            }
        };
        
        let body_text = if compose.show_preview {
            markdown_to_text(&compose.body)
        } else if compose.body.is_empty() && compose.current_field != ComposeField::Body {
            "<empty>".to_string()
        } else {
            compose.body.clone()
        };
        
        let body_widget = Paragraph::new(body_text)
            .block(Block::default().borders(Borders::ALL).title(body_title))
            .wrap(Wrap { trim: false });
        f.render_widget(body_widget, chunks[2]);
    }
}

fn markdown_to_text(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut result = String::new();
    
    for event in parser {
        match event {
            MarkdownEvent::Start(Tag::Heading(..)) => {
                result.push_str("## ");
            }
            MarkdownEvent::End(Tag::Heading(..)) => {
                result.push('\n');
            }
            MarkdownEvent::Start(Tag::Emphasis) => {
                result.push('_');
            }
            MarkdownEvent::End(Tag::Emphasis) => {
                result.push('_');
            }
            MarkdownEvent::Start(Tag::Strong) => {
                result.push_str("**");
            }
            MarkdownEvent::End(Tag::Strong) => {
                result.push_str("**");
            }
            MarkdownEvent::Start(Tag::CodeBlock(_)) => {
                result.push_str("\n```\n");
            }
            MarkdownEvent::End(Tag::CodeBlock(_)) => {
                result.push_str("\n```\n");
            }
            MarkdownEvent::Code(text) => {
                result.push('`');
                result.push_str(&text);
                result.push('`');
            }
            MarkdownEvent::Text(text) => {
                result.push_str(&text);
            }
            MarkdownEvent::SoftBreak => {
                result.push(' ');
            }
            MarkdownEvent::HardBreak => {
                result.push('\n');
            }
            MarkdownEvent::Start(Tag::Paragraph) => {}
            MarkdownEvent::End(Tag::Paragraph) => {
                result.push_str("\n\n");
            }
            MarkdownEvent::Start(Tag::List(_)) => {
                result.push('\n');
            }
            MarkdownEvent::End(Tag::List(_)) => {
                result.push('\n');
            }
            MarkdownEvent::Start(Tag::Item) => {
                result.push_str("• ");
            }
            MarkdownEvent::End(Tag::Item) => {
                result.push('\n');
            }
            _ => {}
        }
    }
    
    result
}

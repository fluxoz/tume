use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, View};

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

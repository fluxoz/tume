use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use tui_markdown::from_str;

use crate::app::{App, View, ComposeMode, ComposeField};

// Helper function to convert ratatui_core::Color to ratatui::Color
fn convert_color(core_color: ratatui_core::style::Color) -> Color {
    use ratatui_core::style::Color as CoreColor;
    match core_color {
        CoreColor::Reset => Color::Reset,
        CoreColor::Black => Color::Black,
        CoreColor::Red => Color::Red,
        CoreColor::Green => Color::Green,
        CoreColor::Yellow => Color::Yellow,
        CoreColor::Blue => Color::Blue,
        CoreColor::Magenta => Color::Magenta,
        CoreColor::Cyan => Color::Cyan,
        CoreColor::Gray => Color::Gray,
        CoreColor::DarkGray => Color::DarkGray,
        CoreColor::LightRed => Color::LightRed,
        CoreColor::LightGreen => Color::LightGreen,
        CoreColor::LightYellow => Color::LightYellow,
        CoreColor::LightBlue => Color::LightBlue,
        CoreColor::LightMagenta => Color::LightMagenta,
        CoreColor::LightCyan => Color::LightCyan,
        CoreColor::White => Color::White,
        CoreColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        CoreColor::Indexed(i) => Color::Indexed(i),
    }
}

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
                    ComposeMode::Normal => "i: Insert | j/k: Navigate fields | d: Clear field | p: Preview | Esc/q: Exit",
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
        
        // Set cursor position for Recipients field
        if compose.current_field == ComposeField::Recipients && compose.mode == ComposeMode::Insert {
            let cursor_x = chunks[0].x + 1 + 4 + compose.cursor_position as u16; // border + "To: " + cursor position
            let cursor_y = chunks[0].y + 1; // border
            f.set_cursor_position((cursor_x.min(chunks[0].right().saturating_sub(2)), cursor_y));
        }

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
        
        // Set cursor position for Subject field
        if compose.current_field == ComposeField::Subject && compose.mode == ComposeMode::Insert {
            let cursor_x = chunks[1].x + 1 + 9 + compose.cursor_position as u16; // border + "Subject: " + cursor position
            let cursor_y = chunks[1].y + 1; // border
            f.set_cursor_position((cursor_x.min(chunks[1].right().saturating_sub(2)), cursor_y));
        }

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
        
        if compose.show_preview && !compose.body.is_empty() {
            // Render markdown using tui-markdown
            let markdown_core_text = from_str(&compose.body);
            // Convert ratatui_core::Text to ratatui::Text by rebuilding from lines
            let mut lines: Vec<Line> = Vec::new();
            for core_line in markdown_core_text.lines {
                let mut spans: Vec<Span> = Vec::new();
                for core_span in core_line.spans {
                    // Convert style from ratatui_core to ratatui
                    let mut style = Style::default();
                    if let Some(fg) = core_span.style.fg {
                        style = style.fg(convert_color(fg));
                    }
                    if let Some(bg) = core_span.style.bg {
                        style = style.bg(convert_color(bg));
                    }
                    // Convert modifiers
                    if core_span.style.add_modifier.contains(ratatui_core::style::Modifier::BOLD) {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if core_span.style.add_modifier.contains(ratatui_core::style::Modifier::ITALIC) {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                    if core_span.style.add_modifier.contains(ratatui_core::style::Modifier::UNDERLINED) {
                        style = style.add_modifier(Modifier::UNDERLINED);
                    }
                    
                    spans.push(Span::styled(
                        core_span.content.to_string(),
                        style,
                    ));
                }
                lines.push(Line::from(spans));
            }
            let markdown_text = Text::from(lines);
            
            let body_widget = Paragraph::new(markdown_text)
                .block(Block::default().borders(Borders::ALL).title(body_title))
                .wrap(Wrap { trim: false });
            f.render_widget(body_widget, chunks[2]);
        } else {
            // Render plain text
            let body_text = if compose.body.is_empty() && compose.current_field != ComposeField::Body {
                "<empty>".to_string()
            } else {
                compose.body.clone()
            };
            
            let body_widget = Paragraph::new(body_text)
                .block(Block::default().borders(Borders::ALL).title(body_title))
                .wrap(Wrap { trim: false });
            f.render_widget(body_widget, chunks[2]);
            
            // Set cursor position for Body field (only in non-preview mode)
            if compose.current_field == ComposeField::Body && compose.mode == ComposeMode::Insert {
                // Calculate cursor position in body text
                let text_before_cursor = &compose.body[..compose.cursor_position.min(compose.body.len())];
                let mut lines = text_before_cursor.lines();
                let line_count = lines.clone().count();
                let col_in_line = lines.last().map(|l| l.len()).unwrap_or(0);
                
                let cursor_x = chunks[2].x + 1 + col_in_line as u16; // border + column position
                let cursor_y = chunks[2].y + 1 + (line_count.saturating_sub(1)) as u16; // border + line number
                f.set_cursor_position((
                    cursor_x.min(chunks[2].right().saturating_sub(2)),
                    cursor_y.min(chunks[2].bottom().saturating_sub(2))
                ));
            }
        }
    }
}

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use tui_markdown::from_str;

use crate::app::{App, ComposeField, ComposeMode, View};

// Layout constants
const MIN_WIDTH_FOR_VERTICAL_SPLIT: u16 = 120;

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

// Helper function to build email metadata display
fn build_email_metadata<'a>(from: &'a str, subject: &'a str, date: &'a str) -> Vec<Line<'a>> {
    vec![
        Line::from(vec![
            Span::styled("From: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(from),
        ]),
        Line::from(vec![
            Span::styled("Subject: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(subject),
        ]),
        Line::from(vec![
            Span::styled("Date: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(date),
        ]),
    ]
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

    render_header(f, chunks[0], app);

    match app.current_view {
        View::InboxList => render_inbox(f, chunks[1], app),
        View::EmailDetail => render_email_detail(f, chunks[1], app),
        View::Compose => render_compose(f, chunks[1], app),
    }

    render_footer(f, chunks[2], app);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    // Build header text with account info if available
    let header_text = if let Some(account_name) = app.get_current_account_name() {
        format!("TUME - Terminal Email Client [{}]", account_name)
    } else {
        "TUME - Terminal Email Client".to_string()
    };

    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

fn render_inbox(f: &mut Frame, area: Rect, app: &App) {
    // If preview panel is enabled, split the view
    if app.show_preview_panel {
        // Decide on split direction based on terminal dimensions
        // Use vertical split if width > threshold, otherwise horizontal
        let use_vertical_split = area.width > MIN_WIDTH_FOR_VERTICAL_SPLIT;

        let chunks = if use_vertical_split {
            // Vertical split: list on left, preview on right
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area)
        } else {
            // Horizontal split: list on top, preview on bottom
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area)
        };

        // Render the inbox list in the first chunk
        render_inbox_list(f, chunks[0], app);

        // Render the preview in the second chunk
        render_inbox_preview(f, chunks[1], app);
    } else {
        // No preview panel, use full area for list
        render_inbox_list(f, area, app);
    }
}

fn render_inbox_list(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .emails
        .iter()
        .enumerate()
        .map(|(i, email)| {
            // Determine style based on visual selection and cursor position
            let style = if i == app.selected_index && app.is_email_selected(i) {
                // Cursor position within visual selection - use a distinct color
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else if app.is_email_selected(i) {
                // In visual mode and selected (but not cursor)
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if i == app.selected_index {
                // Cursor position (not selected in visual mode)
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Calculate column widths for proper alignment
            // From: 30 chars, Subject: remaining space - 20 for date, Date: 20 chars
            let from_width = 30;
            let date_width = 20;

            // Helper function to safely truncate strings at character boundaries
            let truncate_str = |s: &str, max_len: usize| -> String {
                if s.len() <= max_len {
                    return s.to_string();
                }
                // Find the last character boundary before max_len
                let mut end = max_len.saturating_sub(3).max(1);
                while end > 0 && !s.is_char_boundary(end) {
                    end -= 1;
                }
                format!("{}...", &s[..end])
            };

            // Truncate from field if too long
            let from_display = if email.from.len() > from_width {
                truncate_str(&email.from, from_width)
            } else {
                format!("{:<width$}", &email.from, width = from_width)
            };

            // Truncate date field if too long
            let date_display = if email.date.len() > date_width {
                truncate_str(&email.date, date_width)
            } else {
                format!("{:<width$}", &email.date, width = date_width)
            };

            // Calculate subject width (remaining space)
            let available_width = area.width.saturating_sub(4) as usize; // subtract borders
            let subject_width = available_width
                .saturating_sub(from_width + date_width + 4) // subtract column separators
                .max(10); // ensure minimum readable width

            let subject_display = if email.subject.len() > subject_width {
                truncate_str(&email.subject, subject_width)
            } else {
                format!("{:<width$}", &email.subject, width = subject_width)
            };

            let content = Line::from(format!(
                "{}  {}  {}",
                from_display, subject_display, date_display
            ));

            ListItem::new(content).style(style)
        })
        .collect();

    let title = if app.visual_mode {
        format!("Inbox - VISUAL LINE ({} selected)", app.visual_selections.len())
    } else {
        "Inbox (j/k to navigate, Enter to read, V for visual mode, q to quit)".to_string()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title),
    );

    f.render_widget(list, area);
}

fn render_inbox_preview(f: &mut Frame, area: Rect, app: &App) {
    if let Some(email) = app.get_selected_email() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(area);

        // Email metadata
        let metadata = build_email_metadata(&email.from, &email.subject, &email.date);

        let metadata_widget =
            Paragraph::new(metadata).block(Block::default().borders(Borders::ALL).title("Preview"));
        f.render_widget(metadata_widget, chunks[0]);

        // Email body
        let body = Paragraph::new(email.body.as_str())
            .block(Block::default().borders(Borders::ALL).title("Message"))
            .wrap(Wrap { trim: false });
        f.render_widget(body, chunks[1]);
    } else {
        let placeholder = Paragraph::new("No email selected")
            .block(Block::default().borders(Borders::ALL).title("Preview"))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(placeholder, area);
    }
}

fn render_email_detail(f: &mut Frame, area: Rect, app: &App) {
    if let Some(email) = app.get_selected_email() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(area);

        // Email metadata
        let metadata = build_email_metadata(&email.from, &email.subject, &email.date);

        let metadata_widget = Paragraph::new(metadata).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Email Details"),
        );
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
            if app.visual_mode {
                "j/k: Extend selection | d: Delete selected | a: Archive selected | Esc: Exit visual mode"
            } else {
                "j/k: Navigate | Enter/l: Read | V: Visual mode | p: Preview | d: Delete | a: Archive | c: Compose | q: Quit"
            }
        }
        View::EmailDetail => {
            "h/Esc: Back | d: Delete | a: Archive | r: Reply | f: Forward | q: Quit"
        }
        View::Compose => {
            if let Some(ref compose) = app.compose_state {
                match compose.mode {
                    ComposeMode::Normal => {
                        "i: Insert | j/k: Navigate | d: Clear | p: Preview | w: Save draft | Esc/q: Exit"
                    }
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
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default()
        };

        let recipients_text =
            if compose.recipients.is_empty() && compose.current_field != ComposeField::Recipients {
                Span::styled("<empty>", Style::default().fg(Color::DarkGray))
            } else {
                Span::styled(&compose.recipients, Style::default())
            };

        let recipients_widget = Paragraph::new(Line::from(vec![
            Span::styled("To: ", recipients_style),
            recipients_text,
        ]))
        .block(Block::default().borders(Borders::ALL).title(
            if compose.current_field == ComposeField::Recipients
                && compose.mode == ComposeMode::Insert
            {
                "Recipients [INSERT]"
            } else if compose.current_field == ComposeField::Recipients {
                "Recipients [NORMAL]"
            } else {
                "Recipients"
            },
        ));
        f.render_widget(recipients_widget, chunks[0]);

        // Set cursor position for Recipients field
        if compose.current_field == ComposeField::Recipients && compose.mode == ComposeMode::Insert
        {
            let cursor_x = chunks[0].x + 1 + 4 + compose.cursor_position as u16; // border + "To: " + cursor position
            let cursor_y = chunks[0].y + 1; // border
            f.set_cursor_position((cursor_x.min(chunks[0].right().saturating_sub(2)), cursor_y));
        }

        // Subject field
        let subject_style = if compose.current_field == ComposeField::Subject {
            if compose.mode == ComposeMode::Insert {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default()
        };

        let subject_text =
            if compose.subject.is_empty() && compose.current_field != ComposeField::Subject {
                Span::styled("<empty>", Style::default().fg(Color::DarkGray))
            } else {
                Span::styled(&compose.subject, Style::default())
            };

        let subject_widget = Paragraph::new(Line::from(vec![
            Span::styled("Subject: ", subject_style),
            subject_text,
        ]))
        .block(Block::default().borders(Borders::ALL).title(
            if compose.current_field == ComposeField::Subject && compose.mode == ComposeMode::Insert
            {
                "Subject [INSERT]"
            } else if compose.current_field == ComposeField::Subject {
                "Subject [NORMAL]"
            } else {
                "Subject"
            },
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
                    if core_span
                        .style
                        .add_modifier
                        .contains(ratatui_core::style::Modifier::BOLD)
                    {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if core_span
                        .style
                        .add_modifier
                        .contains(ratatui_core::style::Modifier::ITALIC)
                    {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                    if core_span
                        .style
                        .add_modifier
                        .contains(ratatui_core::style::Modifier::UNDERLINED)
                    {
                        style = style.add_modifier(Modifier::UNDERLINED);
                    }

                    spans.push(Span::styled(core_span.content.to_string(), style));
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
            let body_text =
                if compose.body.is_empty() && compose.current_field != ComposeField::Body {
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
                let text_before_cursor =
                    &compose.body[..compose.cursor_position.min(compose.body.len())];

                // Count newlines to get line number
                let line_count = text_before_cursor.matches('\n').count();

                // Get column position by finding characters after last newline
                let col_in_line = if let Some(last_newline_pos) = text_before_cursor.rfind('\n') {
                    text_before_cursor[last_newline_pos + 1..].len()
                } else {
                    text_before_cursor.len()
                };

                let cursor_x = chunks[2].x + 1 + col_in_line as u16; // border + column position
                let cursor_y = chunks[2].y + 1 + line_count as u16; // border + line number
                f.set_cursor_position((
                    cursor_x.min(chunks[2].right().saturating_sub(2)),
                    cursor_y.min(chunks[2].bottom().saturating_sub(2)),
                ));
            }
        }
    }
}

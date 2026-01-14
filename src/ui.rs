use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use tui_markdown::from_str;

use crate::app::{App, ComposeField, ComposeMode, View, CredentialField, CredentialsMode};
use crate::credentials::StorageBackend;
use crate::theme::Theme;

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
        View::CredentialsSetup => render_credentials_setup(f, chunks[1], app),
        View::CredentialsUnlock => render_credentials_unlock(f, chunks[1], app),
        View::CredentialsManagement => render_credentials_management(f, chunks[1], app),
    }

    render_footer(f, chunks[2], app);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    
    // Build header text with account info if available
    // Don't show account name during credentials setup/unlock
    let header_text = if app.current_view == View::CredentialsSetup 
        || app.current_view == View::CredentialsUnlock 
        || app.current_view == View::CredentialsManagement {
        "TUME - Terminal Email Client".to_string()
    } else if let Some(account_name) = app.get_current_account_name() {
        format!("TUME - Terminal Email Client [{}]", account_name)
    } else {
        "TUME - Terminal Email Client".to_string()
    };

    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(theme.title.to_color())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border.to_color())));
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
    let theme = &app.theme;
    
    let items: Vec<ListItem> = app
        .emails
        .iter()
        .enumerate()
        .map(|(i, email)| {
            // Determine style based on visual selection and cursor position
            let style = if i == app.selected_index && app.is_email_selected(i) {
                // Cursor position within visual selection - use a distinct color
                Style::default()
                    .bg(theme.cursor.to_color())
                    .fg(theme.text_bold.to_color())
                    .add_modifier(Modifier::BOLD)
            } else if app.is_email_selected(i) {
                // In visual mode and selected (but not cursor)
                Style::default()
                    .bg(theme.visual_selection.to_color())
                    .fg(theme.text_normal.to_color())
                    .add_modifier(Modifier::BOLD)
            } else if i == app.selected_index {
                // Cursor position (not selected in visual mode)
                Style::default()
                    .bg(theme.selection.to_color())
                    .fg(theme.text_bold.to_color())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text_normal.to_color())
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
            .border_style(Style::default().fg(theme.border.to_color()))
            .title(title),
    );

    f.render_widget(list, area);
}

fn render_inbox_preview(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    
    if let Some(email) = app.get_selected_email() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(area);

        // Email metadata
        let metadata = build_email_metadata(&email.from, &email.subject, &email.date);

        let metadata_widget =
            Paragraph::new(metadata).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border.to_color()))
                    .title("Preview"));
        f.render_widget(metadata_widget, chunks[0]);

        // Email body
        let body = Paragraph::new(email.body.as_str())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border.to_color()))
                .title("Message"))
            .wrap(Wrap { trim: false });
        f.render_widget(body, chunks[1]);
    } else {
        let placeholder = Paragraph::new("No email selected")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border.to_color()))
                .title("Preview"))
            .style(Style::default().fg(theme.text_dim.to_color()));
        f.render_widget(placeholder, area);
    }
}

fn render_email_detail(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    
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
                .border_style(Style::default().fg(theme.border.to_color()))
                .title("Email Details"),
        );
        f.render_widget(metadata_widget, chunks[0]);

        // Email body
        let body = Paragraph::new(email.body.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border.to_color()))
                    .title("Message (h to go back)"),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(body, chunks[1]);
    }
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    
    let help_text = match app.current_view {
        View::InboxList => {
            if app.visual_mode {
                "j/k: Extend selection | d: Delete selected | a: Archive selected | Esc: Exit visual mode"
            } else {
                "j/k: Navigate | Enter/l: Read | V: Visual mode | p: Preview | s: Sync | d: Delete | a: Archive | c: Compose | m: Creds | q: Quit"
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
        View::CredentialsSetup => {
            if let Some(setup) = &app.credentials_setup_state {
                if setup.provider_selection_mode {
                    "j/k: Navigate | Enter/l: Select provider | Esc/q: Cancel"
                } else {
                    match setup.mode {
                        crate::app::CredentialsMode::Normal => {
                            "i: Insert | j/k: Navigate fields | h: Back to providers | P: Toggle passwords | Enter: Save | Esc/q: Cancel"
                        }
                        crate::app::CredentialsMode::Insert => {
                            "Esc: Normal mode | Type to edit field | Left/Right: Move cursor"
                        }
                    }
                }
            } else {
                "i: Insert | j/k: Navigate fields | P: Toggle passwords | Enter: Save | Esc: Cancel"
            }
        }
        View::CredentialsUnlock => {
            "Type master password | Enter: Unlock | Esc: Quit"
        }
        View::CredentialsManagement => {
            "r: Reset credentials | Esc: Back to inbox"
        }
    };

    // Build status bar with mode indicator and current theme
    let mode_text = match app.current_view {
        View::InboxList if app.visual_mode => " VISUAL LINE ",
        View::Compose => {
            if let Some(ref compose) = app.compose_state {
                match compose.mode {
                    ComposeMode::Normal => " NORMAL ",
                    ComposeMode::Insert => " INSERT ",
                }
            } else {
                ""
            }
        }
        View::CredentialsSetup => {
            if let Some(ref setup) = app.credentials_setup_state {
                match setup.mode {
                    CredentialsMode::Normal => " NORMAL ",
                    CredentialsMode::Insert => " INSERT ",
                }
            } else {
                ""
            }
        }
        _ => "",
    };
    
    // Email count indicator
    let email_count = if app.current_view == View::InboxList {
        format!(" {} emails ", app.emails.len())
    } else {
        String::new()
    };
    
    // Theme name in footer
    let theme_indicator = format!(" {} ", app.theme.name);

    let text = if let Some(ref msg) = app.status_message {
        vec![
            Line::from(vec![
                Span::styled(mode_text, 
                    Style::default()
                        .bg(theme.status_bar_mode.to_color())
                        .fg(theme.status_bar.to_color())  // Use status_bar color for better contrast
                        .add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(msg, Style::default().fg(theme.warning.to_color())),
                Span::raw("  "),
                Span::styled(email_count, Style::default().fg(theme.text_dim.to_color())),
                Span::styled(theme_indicator, Style::default().fg(theme.text_dim.to_color())),
            ]),
            Line::from(Span::styled(help_text, Style::default().fg(theme.text_dim.to_color()))),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled(mode_text, 
                    Style::default()
                        .bg(theme.status_bar_mode.to_color())
                        .fg(theme.status_bar.to_color())  // Use status_bar color for better contrast
                        .add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(email_count, Style::default().fg(theme.text_dim.to_color())),
                Span::styled(theme_indicator, Style::default().fg(theme.text_dim.to_color())),
            ]),
            Line::from(Span::styled(help_text, Style::default().fg(theme.text_dim.to_color()))),
        ]
    };

    let footer = Paragraph::new(text)
        .style(Style::default().bg(theme.status_bar.to_color()))
        .alignment(Alignment::Left)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border.to_color())));
    f.render_widget(footer, area);
}

fn render_compose(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    
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
                    .fg(theme.insert_mode.to_color())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(theme.active_field.to_color())
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default().fg(theme.compose_field_label.to_color())
        };

        let recipients_text =
            if compose.recipients.is_empty() && compose.current_field != ComposeField::Recipients {
                Span::styled("<empty>", Style::default().fg(theme.compose_field_empty.to_color()))
            } else {
                Span::styled(&compose.recipients, Style::default().fg(theme.compose_field_value.to_color()))
            };

        let recipients_widget = Paragraph::new(Line::from(vec![
            Span::styled("To: ", recipients_style),
            recipients_text,
        ]))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(
                if compose.current_field == ComposeField::Recipients {
                    theme.border_focused.to_color()
                } else {
                    theme.border.to_color()
                }
            ))
            .title(
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
                    .fg(theme.insert_mode.to_color())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(theme.active_field.to_color())
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default().fg(theme.compose_field_label.to_color())
        };

        let subject_text =
            if compose.subject.is_empty() && compose.current_field != ComposeField::Subject {
                Span::styled("<empty>", Style::default().fg(theme.compose_field_empty.to_color()))
            } else {
                Span::styled(&compose.subject, Style::default().fg(theme.compose_field_value.to_color()))
            };

        let subject_widget = Paragraph::new(Line::from(vec![
            Span::styled("Subject: ", subject_style),
            subject_text,
        ]))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(
                if compose.current_field == ComposeField::Subject {
                    theme.border_focused.to_color()
                } else {
                    theme.border.to_color()
                }
            ))
            .title(
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
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(
                        if compose.current_field == ComposeField::Body {
                            theme.border_focused.to_color()
                        } else {
                            theme.border.to_color()
                        }
                    ))
                    .title(body_title))
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
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(
                        if compose.current_field == ComposeField::Body {
                            theme.border_focused.to_color()
                        } else {
                            theme.border.to_color()
                        }
                    ))
                    .title(body_title))
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

fn render_credentials_setup(f: &mut Frame, area: Rect, app: &App) {
    let setup = match &app.credentials_setup_state {
        Some(s) => s,
        None => return,
    };

    // Check if we're in provider selection mode
    if setup.provider_selection_mode {
        render_provider_selection(f, area, app);
    } else {
        render_credentials_fields(f, area, app);
    }
}

fn render_provider_selection(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let setup = match &app.credentials_setup_state {
        Some(s) => s,
        None => return,
    };

    let backend = app.credentials_manager
        .as_ref()
        .map(|m| m.backend())
        .unwrap_or(StorageBackend::EncryptedFile);

    // Title
    let title = " Email Provider Setup ";
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border.to_color()))
        .title(title)
        .style(Style::default().fg(theme.title.to_color()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split into sections: instructions and provider list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Instructions
            Constraint::Min(10),    // Provider list
        ])
        .split(inner);

    // Render instructions
    let instructions = vec![
        Line::from(Span::styled(
            "Select your email provider",
            Style::default().add_modifier(Modifier::BOLD).fg(theme.text_highlight.to_color()),
        )),
        Line::from(""),
        Line::from(format!("Credentials will be stored using: {}", backend.as_str())),
    ];
    let instructions_para = Paragraph::new(instructions).wrap(Wrap { trim: false });
    f.render_widget(instructions_para, chunks[0]);

    // Render provider list
    let providers = crate::providers::EmailProvider::all();
    let items: Vec<ListItem> = providers
        .iter()
        .enumerate()
        .map(|(i, provider)| {
            let style = if i == setup.provider_list_index {
                Style::default()
                    .fg(theme.active_field.to_color())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text_normal.to_color())
            };

            let marker = if i == setup.provider_list_index {
                "▸ "
            } else {
                "  "
            };

            let content = vec![
                Line::from(vec![
                    Span::styled(marker, style),
                    Span::styled(provider.name, style),
                ]),
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(provider.description, Style::default().fg(theme.text_dim.to_color())),
                ]),
            ];

            ListItem::new(content).style(Style::default())
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border.to_color()))
            .title("Available Providers"),
    );
    f.render_widget(list, chunks[1]);
}

fn render_credentials_fields(f: &mut Frame, area: Rect, app: &App) {
    let setup = match &app.credentials_setup_state {
        Some(s) => s,
        None => return,
    };

    let backend = app.credentials_manager
        .as_ref()
        .map(|m| m.backend())
        .unwrap_or(crate::credentials::StorageBackend::EncryptedFile);

    // Get selected provider name for title
    let provider_name = setup.selected_provider
        .as_ref()
        .and_then(|id| crate::providers::EmailProvider::by_id(id))
        .map(|p| p.name)
        .unwrap_or("Custom");

    // Title with mode indicator
    let mode_str = match setup.mode {
        crate::app::CredentialsMode::Normal => "NORMAL",
        crate::app::CredentialsMode::Insert => "INSERT",
    };
    let title = format!(" {} - Credentials Setup [{}] ", provider_name, mode_str);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split into sections: instructions, form, and backend info
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // Instructions
            Constraint::Min(10),    // Form fields
            Constraint::Length(5),  // Backend info
        ])
        .split(inner);

    // Render instructions
    let instructions = vec![
        Line::from(Span::styled(
            "Configure your email server credentials",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Storage: {}", backend.description())),
    ];
    let instructions_para = Paragraph::new(instructions).wrap(Wrap { trim: false });
    f.render_widget(instructions_para, chunks[0]);

    // Build field items
    let mut field_lines = Vec::new();
    
    // Pre-compute masked passwords to avoid temporary value issues
    let imap_pwd_masked = "*".repeat(setup.imap_password.len());
    let smtp_pwd_masked = "*".repeat(setup.smtp_password.len());
    let master_pwd_masked = "*".repeat(setup.master_password.len());
    let master_pwd_confirm_masked = "*".repeat(setup.master_password_confirm.len());
    
    // IMAP fields
    field_lines.push(Line::from(Span::styled("IMAP Configuration", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
    field_lines.push(build_field_line("IMAP Server:", &setup.imap_server, setup.current_field == CredentialField::ImapServer, setup.mode));
    field_lines.push(build_field_line("IMAP Port:", &setup.imap_port, setup.current_field == CredentialField::ImapPort, setup.mode));
    field_lines.push(build_field_line("IMAP Username:", &setup.imap_username, setup.current_field == CredentialField::ImapUsername, setup.mode));
    field_lines.push(build_field_line("IMAP Password:", 
        if setup.show_passwords { &setup.imap_password } else { &imap_pwd_masked },
        setup.current_field == CredentialField::ImapPassword, setup.mode));
    field_lines.push(Line::from(""));
    
    // SMTP fields
    field_lines.push(Line::from(Span::styled("SMTP Configuration", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
    field_lines.push(build_field_line("SMTP Server:", &setup.smtp_server, setup.current_field == CredentialField::SmtpServer, setup.mode));
    field_lines.push(build_field_line("SMTP Port:", &setup.smtp_port, setup.current_field == CredentialField::SmtpPort, setup.mode));
    field_lines.push(build_field_line("SMTP Username:", &setup.smtp_username, setup.current_field == CredentialField::SmtpUsername, setup.mode));
    field_lines.push(build_field_line("SMTP Password:", 
        if setup.show_passwords { &setup.smtp_password } else { &smtp_pwd_masked },
        setup.current_field == CredentialField::SmtpPassword, setup.mode));
    
    // Master password fields (only for encrypted file backend)
    if backend == StorageBackend::EncryptedFile {
        field_lines.push(Line::from(""));
        field_lines.push(Line::from(Span::styled("Master Password (for encrypted file)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
        field_lines.push(build_field_line("Master Password:", 
            if setup.show_passwords { &setup.master_password } else { &master_pwd_masked },
            setup.current_field == CredentialField::MasterPassword, setup.mode));
        field_lines.push(build_field_line("Confirm Password:", 
            if setup.show_passwords { &setup.master_password_confirm } else { &master_pwd_confirm_masked },
            setup.current_field == CredentialField::MasterPasswordConfirm, setup.mode));
    }

    let fields_para = Paragraph::new(field_lines).wrap(Wrap { trim: false });
    f.render_widget(fields_para, chunks[1]);

    // Render mode-specific tips
    let backend_info = if setup.mode == crate::app::CredentialsMode::Normal {
        vec![
            Line::from(""),
            Line::from(Span::styled("Tip:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from("  Press 'i' to enter Insert mode to edit fields"),
            Line::from("  Press 'P' to toggle password visibility"),
            Line::from("  Press 'h' on first field to go back to provider selection"),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled("Tip:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from("  Press 'Esc' to return to Normal mode"),
            Line::from("  Type freely - all characters including 'j' and 'k' work"),
            Line::from("  Use Left/Right arrows to move cursor within field"),
        ]
    };
    let info_para = Paragraph::new(backend_info).wrap(Wrap { trim: false });
    f.render_widget(info_para, chunks[2]);
}

fn build_field_line<'a>(label: &'a str, value: &'a str, is_active: bool, mode: crate::app::CredentialsMode) -> Line<'a> {
    let style = if is_active {
        if mode == crate::app::CredentialsMode::Insert {
            // Insert mode - yellow and bold
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            // Normal mode - green and bold
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        }
    } else {
        Style::default()
    };
    
    let marker = if is_active { "▸ " } else { "  " };
    Line::from(vec![
        Span::styled(marker, style),
        Span::styled(format!("{:<20}", label), Style::default().add_modifier(Modifier::DIM)),
        Span::styled(value, style),
    ])
}

fn render_credentials_unlock(f: &mut Frame, area: Rect, app: &App) {
    let unlock = match &app.credentials_unlock_state {
        Some(u) => u,
        None => return,
    };

    // Center the unlock dialog
    let dialog_width = 60.min(area.width.saturating_sub(4));
    let dialog_height = 12.min(area.height.saturating_sub(4));
    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: area.x + dialog_x,
        y: area.y + dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear background
    let clear_widget = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(clear_widget, area);

    // Render dialog
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Unlock Credentials ")
        .style(Style::default().fg(Color::Cyan));

    let inner = block.inner(dialog_area);
    f.render_widget(block, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Instructions
            Constraint::Length(1),  // Spacing
            Constraint::Length(3),  // Password field
            Constraint::Length(1),  // Spacing
            Constraint::Min(1),     // Error message or tip
        ])
        .split(inner);

    // Instructions
    let instructions = vec![
        Line::from("Your credentials are stored in an encrypted file."),
        Line::from("Enter your master password to unlock them."),
    ];
    let instructions_para = Paragraph::new(instructions)
        .style(Style::default().fg(Color::White));
    f.render_widget(instructions_para, chunks[0]);

    // Password field
    let password_display = "*".repeat(unlock.master_password.len());
    let password_field = Paragraph::new(vec![
        Line::from(Span::styled("Master Password:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(
            password_display,
            Style::default().fg(Color::Green),
        )),
    ])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(password_field, chunks[2]);

    // Error message or tip
    let message = if let Some(ref err) = unlock.error_message {
        vec![Line::from(Span::styled(
            err,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "Press Enter to unlock, Esc to quit",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    };
    let message_para = Paragraph::new(message);
    f.render_widget(message_para, chunks[4]);

    // Set cursor position in password field
    let password_field_inner = Block::default().borders(Borders::ALL).inner(chunks[2]);
    let cursor_x = password_field_inner.x.saturating_add(unlock.cursor_position.min(u16::MAX as usize) as u16);
    f.set_cursor_position((
        cursor_x.min(password_field_inner.right().saturating_sub(1)),
        password_field_inner.y + 1,
    ));
}

fn render_credentials_management(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Credentials Management ")
        .style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(5),  // Backend info
            Constraint::Min(1),     // Actions
        ])
        .split(inner);

    // Title
    let title = vec![
        Line::from(Span::styled(
            "Manage Your Credentials",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    let title_para = Paragraph::new(title);
    f.render_widget(title_para, chunks[0]);

    // Backend info
    let (backend, description) = app.get_backend_info()
        .unwrap_or((StorageBackend::EncryptedFile, "Unknown".to_string()));
    
    let backend_info = vec![
        Line::from(vec![
            Span::styled("Current Backend: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(backend.as_str(), Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(description),
    ];
    let info_para = Paragraph::new(backend_info).wrap(Wrap { trim: false });
    f.render_widget(info_para, chunks[1]);

    // Actions
    let actions = vec![
        Line::from(""),
        Line::from(Span::styled("Available Actions:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("  r - Reset credentials (delete current and set up new ones)"),
        Line::from("  Esc - Return to inbox"),
        Line::from(""),
        Line::from(Span::styled(
            "⚠ Warning: Resetting credentials will require you to reconfigure your email settings",
            Style::default().fg(Color::Yellow),
        )),
    ];
    let actions_para = Paragraph::new(actions);
    f.render_widget(actions_para, chunks[2]);
}


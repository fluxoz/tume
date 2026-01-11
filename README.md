# TUME - Terminal Email Client

A Terminal User Interface (TUI) email client built with Rust, featuring vim-style keybindings for efficient email management.

## Features

### Current Implementation

- **Inbox View**: Browse a list of emails with sender, subject, date, and preview
- **Email Detail View**: Read full email content
- **Compose View**: Full-featured email composition with modal editing
- **Vim Keybindings**: Navigate efficiently using familiar vim commands
- **Modal Editing**: Normal and Insert modes for composing emails
- **Markdown Support**: Compose emails with markdown and preview rendering
- **Email Actions**: Delete, archive, reply, forward emails (placeholder implementations)

## Installation

### Build from Source

```bash
cargo build --release
```

### Run

```bash
cargo run
```

## Usage

### Inbox View (Main Screen)

The inbox displays a list of emails with the following information:
- **From**: Email sender
- **Subject**: Email subject line
- **Date**: Date and time received
- **Preview**: First line of the email body

#### Keybindings (Inbox View)

| Key | Action |
|-----|--------|
| `j` or `↓` | Move down to next email |
| `k` or `↑` | Move up to previous email |
| `Enter` or `l` | Open selected email |
| `d` | Delete email (placeholder) |
| `a` | Archive email (placeholder) |
| `r` | Reply to email (placeholder) |
| `c` | Compose new email |
| `f` | Forward email (placeholder) |
| `q` or `Esc` | Quit application |

### Email Detail View

When you open an email, you'll see:
- Full email headers (From, Subject, Date)
- Complete email body

#### Keybindings (Detail View)

| Key | Action |
|-----|--------|
| `h` or `Esc` | Go back to inbox |
| `d` | Delete email (placeholder) |
| `a` | Archive email (placeholder) |
| `r` | Reply to email (placeholder) |
| `f` | Forward email (placeholder) |
| `q` | Quit application |

### Compose View

The compose view allows you to write new emails with vim-style modal editing.

#### Modal Editing

- **Normal Mode**: Navigate between fields and access commands (default)
- **Insert Mode**: Edit field content

#### Email Fields

1. **Recipients**: Email addresses of recipients
2. **Subject**: Email subject line
3. **Body**: Email message body (supports markdown)

#### Keybindings (Compose View - Normal Mode)

| Key | Action |
|-----|--------|
| `i` | Enter Insert mode for current field |
| `j` or `↓` | Move to next field |
| `k` or `↑` | Move to previous field |
| `d` | Clear the current field |
| `p` | Toggle markdown preview for body |
| `Esc` or `q` | Exit compose mode |

#### Keybindings (Compose View - Insert Mode)

| Key | Action |
|-----|--------|
| `Esc` | Exit Insert mode and advance to next field |
| `Backspace` | Delete character |
| `Enter` | Insert newline (body field only) |
| Any character | Insert character into current field |

#### Workflow Example

1. Press `c` from inbox to start composing
2. Recipients field is selected (Normal mode)
3. Press `i` to enter Insert mode
4. Type email addresses (e.g., "user@example.com")
5. Press `Esc` to exit Insert mode and advance to Subject
6. Press `i` to edit Subject
7. Type subject line
8. Press `Esc` to advance to Body
9. Press `i` to edit Body
10. Type your message (supports markdown: **bold**, _italic_, ## headings, - lists)
11. Press `Esc` to exit Insert mode
12. Press `p` to preview markdown rendering
13. Press `Esc` or `q` to exit compose

#### Markdown Support

The body field supports markdown with rich terminal preview rendering (powered by `tui-markdown`):
- **Headings**: `## Heading` - rendered with styling
- **Bold**: `**bold text**` - rendered in bold
- **Italic**: `_italic text_` - rendered in italics  
- **Code**: `` `inline code` `` - rendered with code styling
- **Code blocks**: ` ``` code block ``` ` - rendered in code block format
- **Lists**: `- list item` - rendered with bullets
- And more markdown features with proper terminal styling

Press `p` in Normal mode to toggle between raw markdown and rendered preview.

## Architecture

The application is structured into several modules:

- **`main.rs`**: Entry point, terminal setup, and main event loop
- **`app.rs`**: Application state management and business logic
- **`ui.rs`**: UI rendering using Ratatui
- **`events.rs`**: Keyboard event handling and input processing

## Dependencies

- **ratatui**: Terminal UI library for creating rich text user interfaces
- **crossterm**: Cross-platform terminal manipulation
- **anyhow**: Error handling
- **tui-markdown**: Markdown parsing and terminal rendering
- **ratatui-core**: Core types for markdown rendering
- **anyhow**: Error handling
- **pulldown-cmark**: Markdown parsing and rendering

## Development Status

### Completed Features
- ✅ Inbox view with email listing
- ✅ Email detail view
- ✅ Compose view with modal editing
- ✅ Markdown preview in compose
- ✅ Vim-style keybindings throughout
- ✅ GPG and Yubikey hooks (stubs for future encryption/signing)

### Placeholder Features
- Email deletion (shows status message)
- Email archiving (shows status message)
- Reply functionality (shows status message)
- Forward email (shows status message)
- Actual sending of composed emails

### Future Development
- Actual email protocol integration (IMAP/SMTP)
- Send composed emails via SMTP
- Reply and forward with pre-filled fields
- GPG encryption implementation
- Yubikey signing implementation
- Configuration system
- Multiple account support
- Email threading
- Search functionality
- Filtering and sorting

## License

This project is part of the TUME email client initiative.

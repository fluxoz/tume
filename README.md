# TUME - Terminal Email Client

A Terminal User Interface (TUI) email client built with Rust, featuring vim-style keybindings for efficient email management.

## Features

### Current Implementation

- **Inbox View**: Browse a list of emails with sender, subject, date, and preview
- **Email Detail View**: Read full email content
- **Vim Keybindings**: Navigate efficiently using familiar vim commands
- **Email Actions**: Delete, archive, reply, compose, and forward emails (placeholder implementations)

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
| `c` | Compose new email (placeholder) |
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

## Development Status

This is a stub implementation demonstrating the core TUI functionality. The following features are currently placeholders:
- Email deletion (shows status message)
- Email archiving (shows status message)
- Reply functionality (shows status message)
- Compose new email (shows status message)
- Forward email (shows status message)

Future development will include:
- Actual email protocol integration (IMAP/SMTP)
- Email composition interface
- Configuration system
- Multiple account support
- Email threading
- Search functionality
- Filtering and sorting

## License

This project is part of the TUME email client initiative.

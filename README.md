# TUME - Terminal Email Client

A Terminal User Interface (TUI) email client built with Rust, featuring vim-style keybindings for efficient email management.

## Features

### Current Implementation

- **Multi-Account Support**: Manage multiple email accounts with easy switching (1-9, Tab, [/])
- **Configuration**: TOML-based configuration at `~/.config/tume/config.toml`
- **Inbox View**: Browse a list of emails with sender, subject, date, and preview
- **Email Detail View**: Read full email content
- **Compose View**: Full-featured email composition with modal editing
- **Vim Keybindings**: Navigate efficiently using familiar vim commands
- **Visual Line Mode**: Select multiple emails with Shift+V for batch operations (delete, archive)
- **Modal Editing**: Normal and Insert modes for composing emails
- **Markdown Support**: Compose emails with markdown and preview rendering
- **Draft Management**: Auto-save drafts on exit, restore on re-entry, with explicit save option
- **Local Database**: Email and draft storage using Turso/libSQL at `~/.local/share/tume/mail.db`
- **Email Actions**: Delete, archive, reply, forward emails (reply and forward are placeholders)
- **Hybrid Credentials Storage**: Secure storage using system keyring or encrypted file with master password

## Credentials Management

TUME features a robust hybrid credentials storage system that prioritizes security while ensuring compatibility across all platforms.

### Storage Backends

#### System Keyring (Default)

On supported platforms, TUME automatically uses your system's secure credential storage:

- **macOS**: Keychain
- **Windows**: Credential Manager
- **Linux**: Secret Service (requires `libsecret` or similar)

No master password is required - credentials are protected by your system's security.

#### Encrypted File (Fallback)

If the system keyring is unavailable (headless Linux, containers, or unsupported environments), TUME automatically falls back to an encrypted file:

- **Location**: `~/.local/share/tume/credentials.enc`
- **Encryption**: AES-256-GCM with Argon2 key derivation
- **Protection**: Secured by a user-provided master password

### First-Time Setup

When you first run TUME, you'll be guided through credential setup:

1. **Select Your Email Provider**:
   - Choose from popular pre-configured providers:
     - **Gmail** - Google Gmail (requires app-specific password if 2FA enabled)
     - **Outlook / Office 365** - Microsoft email services
     - **Yahoo Mail** - Yahoo email (requires app-specific password)
     - **ProtonMail Bridge** - ProtonMail via local bridge
     - **iCloud Mail** - Apple iCloud (requires app-specific password)
     - **Fastmail** - Privacy-focused email service
     - **AOL Mail** - AOL email (requires app-specific password)
     - **Zoho Mail** - Business and personal email
     - **GMX Mail** - Free email service
     - **Mail.com** - Free email with many domain options
     - **Yandex Mail** - Russian email service
     - **Custom (Other Provider)** - Manually configure any other IMAP/SMTP provider
   
   Server settings (IMAP/SMTP addresses and ports) are automatically pre-filled based on your provider selection!

2. **Configure Email Server Settings**:
   - IMAP server address and port (pre-filled for known providers)
   - IMAP username and password
   - SMTP server address and port (pre-filled for known providers)
   - SMTP username and password

3. **Set Master Password** (if using encrypted file backend):
   - Choose a strong password (minimum 8 characters)
   - Confirm the password
   - This password will be required each time you start TUME

4. **Save Credentials**:
   - Press `Enter` to save your credentials
   - The backend selection is automatic based on availability

### Provider Selection View

When first setting up credentials, you'll see a provider selection screen:

| Key | Action |
|-----|--------|
| `j`, `↓` | Navigate to next provider |
| `k`, `↑` | Navigate to previous provider |
| `Enter`, `l`, `→` | Select provider and proceed to credential entry |
| `Esc`, `q` | Cancel setup (quit if first-time) |

### Credentials Field Entry View

After selecting a provider, you'll enter credentials using **vim-style Normal/Insert modes**:

**Normal Mode** (green highlight):
| Key | Action |
|-----|--------|
| `i` | Enter Insert mode to edit current field |
| `j`, `↓` | Navigate to next field |
| `k`, `↑` | Navigate to previous field |
| `Tab` | Navigate to next field |
| `Shift+Tab` | Navigate to previous field |
| `h`, `←` | Go back to provider selection (when on first field) |
| `P` | Toggle password visibility |
| `Enter` | Save credentials |
| `Esc`, `q` | Cancel setup |

**Insert Mode** (yellow highlight):
| Key | Action |
|-----|--------|
| Type | Enter text in current field (including `j` and `k` characters) |
| `Backspace` | Delete character |
| `Left` / `Right` | Move cursor |
| `Esc` | Exit Insert mode and return to Normal mode |

This prevents the issue where navigation keys like `j`/`k` would trigger field navigation while trying to type those characters.

### Unlocking Credentials

If using the encrypted file backend, you'll need to unlock your credentials at startup:

| Key | Action |
|-----|--------|
| Type | Enter master password |
| `Backspace` | Delete character |
| `Left` / `Right` | Move cursor |
| `Enter` | Unlock and continue |
| `Esc` | Quit application |

### Managing Credentials

Access the credentials management menu from the inbox by pressing `m`:

- **View Backend Info**: See which storage backend is active and why
- **Reset Credentials**: Delete existing credentials and set up new ones
- **Backend Details**: Learn about your current storage method

### Security Best Practices

1. **Master Password**: If using encrypted file storage, choose a strong, unique password
2. **System Keyring**: Keep your system account secure - anyone with access can read keyring credentials
3. **Encrypted File**: Your encrypted credentials file is only as secure as your master password
4. **Network Security**: Always use TLS/SSL for IMAP and SMTP connections (default ports 993/587)

### Troubleshooting

#### Linux: System Keyring Not Available

If TUME falls back to encrypted file on Linux desktop:

```bash
# Install libsecret on Debian/Ubuntu
sudo apt-get install libsecret-1-0 libsecret-1-dev

# Install on Fedora
sudo dnf install libsecret libsecret-devel

# Install on Arch
sudo pacman -S libsecret
```

After installation, restart TUME to use the system keyring.

#### Forgot Master Password

If you forget your master password for the encrypted file backend:

1. The credentials file cannot be decrypted
2. You'll need to delete `~/.local/share/tume/credentials.enc`
3. Restart TUME and set up credentials again

#### Migration Between Backends

Currently, manual migration is not supported. To switch backends:

1. Note your current credentials
2. Reset credentials in the management menu
3. TUME will detect the available backend
4. Re-enter your credentials

## Installation

### Build from Source

```bash
cargo build --release
```

### Run

```bash
cargo run
```

## Configuration

TUME supports multi-account management through a TOML configuration file located at `~/.config/tume/config.toml`.

### Example Configuration

```toml
# TUME Email Client Configuration

# Accounts configuration
[accounts.work]
name = "Work Gmail"
email = "work@company.com"
provider = "gmail"
default = true
color = "blue"
display_order = 1

[accounts.personal]
name = "Personal"
email = "me@gmail.com"
provider = "gmail"
color = "green"
display_order = 2

# Keybindings configuration
[keybindings]
switch_account = ["1", "2", "3", "4", "5", "6", "7", "8", "9"]
next_account = "]"
prev_account = "["
mailbox_picker = "M"
add_account = "A"
```

### Configuration Options

#### Account Settings

- `name`: Display name for the account
- `email`: Email address
- `provider`: Provider type (e.g., "gmail", "outlook", "imap")
- `default`: Set to `true` for the default account (optional)
- `color`: Color for account indicators (optional)
- `display_order`: Order in which accounts appear (lower numbers first)

#### Keybindings

All keybindings are customizable. The default keybindings shown above can be modified to suit your preferences.

### Multi-Account Support

TUME supports managing multiple email accounts simultaneously:

- **Account Switching**: Press `1-9` to switch to accounts 1-9, or use `[` and `]` to cycle through accounts
- **Visual Indicator**: Current account name is shown in the header
- **Per-Account Emails**: Emails are filtered by the currently selected account
- **Unified Inbox**: All accounts can be viewed together (upcoming feature)

## Usage

### Running the Application

```bash
# Normal mode
cargo run

# Development mode (reseeds inbox with test emails on startup)
cargo run -- --dev
```

The `--dev` flag is useful for development and testing. It clears and reseeds the inbox with mock emails every time the application starts, ensuring a consistent test environment.

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
| `V` (Shift+V) | Enter visual line mode for batch operations |
| `p` | Toggle preview panel |
| `1-9` | Switch to account 1-9 |
| `[` | Switch to previous account |
| `]` | Switch to next account |
| `Tab` | Cycle to next account |
| `d` | Delete email |
| `a` | Archive email |
| `r` | Reply to email (placeholder) |
| `c` | Compose new email |
| `f` | Forward email (placeholder) |
| `m` | Manage credentials |
| `q` | Quit application |

#### Visual Line Mode

Visual line mode allows you to select multiple emails and perform batch operations, similar to vim's visual mode.

**Entering Visual Mode:**
- Press `Shift+V` (capital V) while in the inbox view
- The current email will be highlighted in blue
- Status bar shows "-- VISUAL LINE --"
- Title bar shows the count of selected emails

**Visual Mode Keybindings:**

| Key | Action |
|-----|--------|
| `j` or `↓` | Extend selection down |
| `k` or `↑` | Extend selection up |
| `d` | Delete all selected emails |
| `a` | Archive all selected emails |
| `Esc`, `v`, or `V` | Exit visual mode |

**Workflow Example:**
1. Press `Shift+V` to enter visual mode
2. Press `j` twice to select 3 emails (current + 2 below)
3. Press `d` to delete all selected emails
4. Visual mode exits automatically and shows "Deleted 3 emails"

**Notes:**
- Selection is always contiguous (from anchor point to cursor position)
- Only works in inbox list view (not in detail or compose views)
- Batch operations automatically exit visual mode
- Selected emails are highlighted in blue

### Email Detail View

When you open an email, you'll see:
- Full email headers (From, Subject, Date)
- Complete email body

#### Keybindings (Detail View)

| Key | Action |
|-----|--------|
| `h` or `Esc` | Go back to inbox |
| `d` | Delete email |
| `a` | Archive email |
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
| `w` | Save draft to database |
| `Esc` or `q` | Exit compose mode (draft is auto-saved) |

#### Keybindings (Compose View - Insert Mode)

| Key | Action |
|-----|--------|
| `Esc` | Exit Insert mode (auto-advances to next field during initial setup, then stays on Body) |
| `Backspace` | Delete character |
| `Enter` | Insert newline (body field only) |
| `Left` / `Right` | Move cursor left/right |
| Any character | Insert character into current field |

#### Workflow Example

**Initial Setup (one-time traversal):**
1. Press `c` from inbox to start composing
2. Recipients field is selected (Normal mode)
3. Press `i` to enter Insert mode
4. Type email addresses (e.g., "user@example.com")
5. Press `Esc` - auto-advances to Subject field
6. Press `i` to edit Subject
7. Type subject line
8. Press `Esc` - auto-advances to Body field
9. Press `i` to edit Body
10. Type your message (supports markdown: **bold**, _italic_, ## headings, - lists)
11. Press `Esc` - stays on Body field

**After reaching Body:**
- Press `i` to continue editing Body
- Press `Esc` to return to Normal mode (stays on Body)
- Use `j`/`k` to manually navigate to other fields if needed
- Press `p` to preview markdown rendering
- Press `Esc` or `q` to exit compose

#### Markdown Support

The body field supports markdown with rich terminal preview rendering (powered by `tui-markdown`):
- **Headings**: `## Heading` - rendered with styling
- **Bold**: `**bold text**` (must have closing `**`) - rendered in bold
- **Italic**: `_italic text_` (must have closing `_`) - rendered in italics  
- **Code**: `` `inline code` `` - rendered with code styling
- **Code blocks**: ` ``` code block ``` ` - rendered in code block format
- **Lists**: `- list item` - rendered with bullets
- And more markdown features with proper terminal styling

**Important**: Markdown syntax must be complete for rendering. For example, `**text**` (with both opening and closing) will render as bold, but `** text` or `**text` without closing tags will display as plain text.

Press `p` in Normal mode to toggle between raw markdown and rendered preview.

**Example:**
```
Raw markdown:
## Meeting Notes
**Important:** Review by Friday
- First item
- Second item

Preview will show:
Styled heading "Meeting Notes"
"Important:" in bold, followed by " Review by Friday"
• First item
• Second item
```

### Draft Management

TUME automatically manages email drafts to prevent accidental data loss:

#### Draft Behavior

- **Auto-save on Exit**: When you press `Esc` or `q` to exit compose mode, your draft is automatically saved to the database at `~/.local/share/tume/mail.db`
- **Draft Persistence**: The draft remains in memory during the session. Pressing `c` again will restore your draft exactly as you left it
- **Explicit Save**: Press `w` in Normal mode to manually save the draft at any time (you'll see a "Draft saved" status message)
- **Auto-save on Quit**: If you quit the application (press `q` from inbox or detail view) while composing an email, the draft is automatically saved before exit
- **Draft Loading**: When you restart TUME and have a saved draft, it will be automatically loaded when you press `c` to compose a new email

#### Draft Workflow Example

1. Press `c` to start composing an email
2. Enter some content in the fields
3. Press `Esc` to exit (draft is auto-saved)
4. Press `c` again - your draft is restored!
5. Press `w` in Normal mode to save explicitly
6. Continue editing or quit - draft is preserved

**Note**: Currently, TUME keeps only the most recent draft. Sending an email or creating a new draft will replace the previous one.

## Architecture

The application is structured into several modules:

- **`main.rs`**: Entry point, terminal setup, and main event loop
- **`app.rs`**: Application state management and business logic
- **`config.rs`**: Configuration loading and management
- **`db.rs`**: Database operations and schema
- **`ui.rs`**: UI rendering using Ratatui
- **`events.rs`**: Keyboard event handling and input processing

## Dependencies

- **ratatui**: Terminal UI library for creating rich text user interfaces
- **crossterm**: Cross-platform terminal manipulation
- **anyhow**: Error handling
- **serde**: Serialization/deserialization for configuration
- **toml**: TOML configuration file parsing
- **tui-markdown**: Markdown parsing and terminal rendering
- **ratatui-core**: Core types for markdown rendering
- **libsql**: Turso/libSQL for local database storage
- **tokio**: Async runtime for database operations
- **dirs**: Cross-platform directory paths
- **keyring**: System keyring integration for secure credential storage
- **aes-gcm**: AES-256-GCM encryption for file-based credentials
- **argon2**: Password hashing and key derivation
- **serde** / **serde_json**: Data serialization
- **zeroize**: Secure memory zeroing for sensitive data
- **base64**: Binary data encoding

## Development Status

### Completed Features
- ✅ Inbox view with email listing
- ✅ Email detail view
- ✅ Compose view with modal editing
- ✅ Markdown preview in compose
- ✅ Vim-style keybindings throughout
- ✅ Visual line mode for batch operations (Shift+V)
- ✅ Batch delete and archive operations
- ✅ Single delete and archive operations
- ✅ Draft management (auto-save, restore, explicit save with 'w')
- ✅ Local database storage for emails and drafts
- ✅ Hybrid credentials storage (system keyring + encrypted file fallback)
- ✅ Secure credential management UI
- ✅ First-time setup wizard
- ✅ Multi-account support with configuration
- ✅ Account switching (1-9, [, ], Tab)
- ✅ GPG and Yubikey hooks (stubs for future encryption/signing)

### Placeholder Features
- Reply functionality (shows status message)
- Forward email (shows status message)
- Actual sending of composed emails

### Future Development
- Account picker UI (M key)
- Account onboarding wizard (A key)
- Unified inbox view across all accounts
- Actual email protocol integration (IMAP/SMTP)
- Send composed emails via SMTP
- Reply and forward with pre-filled fields
- GPG encryption implementation
- Yubikey signing implementation
- Email threading
- Search functionality
- Filtering and sorting

## License

This project is part of the TUME email client initiative.

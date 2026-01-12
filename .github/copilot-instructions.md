# GitHub Copilot Instructions for TUME

You are an expert Rust developer working on TUME, a Terminal User Interface (TUI) email client with vim-style keybindings.

## Tech Stack

- **Language**: Rust (edition 2024)
- **UI Framework**: Ratatui (v0.29) - Terminal UI library
- **Terminal Handling**: Crossterm (v0.28)
- **Markdown**: tui-markdown (v0.3) for email composition and rendering
- **Error Handling**: anyhow (v1.0)

## Project Structure

```
tume/
├── src/
│   ├── main.rs      # Entry point, terminal setup, main event loop
│   ├── app.rs       # Application state management and business logic
│   ├── ui.rs        # UI rendering using Ratatui
│   └── events.rs    # Keyboard event handling and input processing
├── Cargo.toml       # Project dependencies and metadata
└── README.md        # Comprehensive user documentation
```

## Build and Test Commands

- **Build the project**: `cargo build`
- **Build for release**: `cargo build --release`
- **Run the application**: `cargo run`
- **Run tests**: `cargo test`
- **Check code without building**: `cargo check`
- **Format code**: `cargo fmt`
- **Lint code**: `cargo clippy`

## Coding Conventions

### Rust Style
- Follow standard Rust conventions (use `cargo fmt` and `cargo clippy`)
- Use descriptive variable names
- Prefer explicit error handling with `Result<T>` and `?` operator
- Use `anyhow::Result` for error propagation
- Keep functions focused and single-purpose

### Code Organization
- Each module should have a clear, single responsibility:
  - `main.rs`: Terminal lifecycle management only
  - `app.rs`: All application state and business logic
  - `ui.rs`: All rendering code
  - `events.rs`: All event handling code
- Avoid mixing UI rendering with business logic
- Keep state management centralized in the `App` struct

### Naming Conventions
- Use snake_case for functions and variables
- Use PascalCase for types and structs
- Prefix boolean fields with `is_` or `should_` (e.g., `should_quit`, `is_composing`)
- Use descriptive enum variant names

### Vim Keybindings
When adding new features with keybindings, maintain consistency with vim conventions:
- `j`/`k` for down/up navigation
- `h`/`l` for left/right or back/forward
- `i` for insert mode
- `Esc` to exit modes
- `d` for delete
- `a` for archive/add
- `c` for compose/create
- `q` to quit

## Application Architecture

### Modal System
The app uses vim-style modal editing with three main views:
1. **Inbox View**: Browse email list
2. **Detail View**: Read individual emails
3. **Compose View**: Write emails with Normal/Insert modes

### State Management
- All state is in the `App` struct in `app.rs`
- State changes should be atomic and handled by methods on `App`
- UI should be purely a function of the current state

### Event Loop
The main event loop follows this pattern:
1. Draw UI based on current state
2. Handle keyboard events
3. Update state based on events
4. Repeat until `should_quit` is true

## Testing Guidelines

- Write unit tests for business logic in `app.rs`
- Test state transitions and edge cases
- Use descriptive test names that explain what is being tested
- Mock external dependencies when possible
- Tests should be deterministic and not depend on external state

## Security Considerations

- Never commit secrets or API keys to the repository
- Handle user input safely (all input is already handled via terminal)
- Future SMTP/IMAP integration should use secure connections (TLS)
- GPG and Yubikey features are currently placeholders - implement with proper key management

## Current Development Status

### Implemented Features
- Inbox view with email listing
- Email detail view
- Compose view with modal editing and markdown preview
- Vim-style keybindings throughout
- GPG and Yubikey hooks (stubs)

### Placeholder Features (Show Status Messages Only)
- Email deletion
- Email archiving
- Reply functionality
- Forward email
- Actual sending of emails

### Future Work Areas
- IMAP/SMTP protocol integration
- Actual email sending via SMTP
- Reply/forward with pre-filled fields
- GPG encryption implementation
- Yubikey signing implementation
- Configuration system
- Multiple account support
- Email threading and search

## Do NOT Modify

- **Never change** the existing vim keybinding conventions without explicit request
- **Never remove** working features to simplify code
- **Do not modify** `Cargo.toml` dependencies unless specifically needed for a feature
- **Do not change** the modal editing behavior in compose view
- **Avoid breaking changes** to the public API of the `App` struct

## Task Scoping Best Practices

When working on this repository:
1. Make minimal, focused changes that address the specific issue
2. Maintain backward compatibility unless explicitly told otherwise
3. Test any changes to event handling or state management carefully
4. Update README.md if you add new keybindings or features
5. Follow the existing code structure and patterns
6. Run `cargo fmt` and `cargo clippy` before committing changes

## Example Code Patterns

### Adding a New Action
```rust
// In app.rs - add method to App impl
pub fn new_action(&mut self) {
    // Update state
    self.status_message = "Action performed".to_string();
}

// In events.rs - handle the key event
KeyCode::Char('x') => {
    app.new_action();
}
```

### Adding a New View State
```rust
// In app.rs - add to ViewState enum
pub enum ViewState {
    Inbox,
    Detail,
    Compose,
    NewView,  // Add new variant
}

// Update UI rendering in ui.rs
match app.view_state {
    ViewState::NewView => draw_new_view(f, app, area),
    // ...
}
```

## Documentation
- Keep README.md up to date with feature changes
- Include keybinding documentation for all new shortcuts
- Document complex algorithms or non-obvious code with comments
- Use doc comments (`///`) for public APIs

## Questions or Clarifications
If you encounter ambiguity or need clarification on the desired behavior, ask the user before making assumptions.

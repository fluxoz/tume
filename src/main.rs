mod app;
mod credentials;
mod config;
mod db;
mod email_sync;
mod events;
mod providers;
mod theme;
mod ui;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let dev_mode = args.iter().any(|arg| arg == "--dev");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state with database
    let mut app = App::with_database(dev_mode).await.unwrap_or_else(|e| {
        eprintln!(
            "Warning: Failed to initialize database: {}. Using in-memory mode.",
            e
        );
        App::new()
    });

    // Main loop
    let res = run_app(&mut terminal, &mut app);

    // Save draft before cleaning up terminal (if needed)
    if app.has_unsaved_draft() {
        if let Err(e) = app.save_draft_before_quit_async().await {
            eprintln!("Warning: Failed to save draft before quit: {}", e);
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;
        events::handle_events(app)?;
        
        // Check for completed sync results
        app.check_sync_result();

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

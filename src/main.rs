/*
* Linux File Utility Program
* Simple TUI for exploring a filesystem
* Benjamin Lovy
* July 18, 2021
* SDEV-345
* Professor Gary Savard
*/

// Ergonomic Result and Error types to simply error handling boilerplate
use anyhow::Result;

// Terminal rendering and IO
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};

// TUI widget library
use tui::{backend::TermionBackend, Terminal};

// IO
use std::io;

// Application state
mod app;
// User input event handling.  Largely from TUI-RS documentation.
mod events;
// List data structure that tracks extra state.  Largely from TUI-RS documentation.
mod stateful_list;
// User interface definition
mod ui;

use app::App;
use events::{Event, Events};

fn main() -> Result<()> {
    // Grab a handle to STDOUT in raw mode (no auto printing or buffering)
    let stdout = io::stdout().into_raw_mode()?;
    // Enable mouse
    let stdout = MouseTerminal::from(stdout);
    // Use a separate overaly over exisitng terminal.  On quit, old terminal is restored.
    let stdout = AlternateScreen::from(stdout);

    // Hook up integration to real terminal
    let backend = TermionBackend::new(stdout);
    // Build the handle to the terminal we can draw a TUI to.
    let mut terminal = Terminal::new(backend)?;

    // Init event stream and app state
    let events = Events::new();
    let mut app = App::new();

    // Render the app.  Runs forever, or until a "quit" event is received.
    // The full widget graph is re-built on every frame.
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // 1. This breaks the loop and exits the program on `q` button press.
        // 2. The `up`/`down` keys change the currently selected item in the App's `items` list.
        // 3. `left` unselects the current item.
        if let Event::Input(input) = events.next()? {
            match input {
                Key::Char('q') => {
                    break;
                }
                Key::Left => {
                    app.dir_list.unselect();
                }
                Key::Down => {
                    app.dir_list.next();
                }
                Key::Up => {
                    app.dir_list.previous();
                }
                _ => {} // Ignore all other key inputs
            }
        }
    }

    Ok(())
}

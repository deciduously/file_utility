//! # Linux File Utility Program
//!
//! `file_utility` is a simple terminal user interface (TUI) for exploring a filesystem.
//!  It implements functionality for navigating a directory, viewing file metadata, changing directories, and changing file permissions.

// Ergonomic Result and Error types to simply error handling boilerplate
use anyhow::{Error, Result};

// Terminal rendering and IO
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};

// TUI widget library
use tui::{backend::TermionBackend, Terminal};

// IO
use std::{
    io::{self, stderr, Write},
    path::PathBuf,
};

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

/// Print an error that occurred as well as any errors that were chained to get there.
fn print_error(err: Error) {
    let _ = writeln!(stderr(), "error: {}", err);
    for cause in err.chain() {
        let _ = writeln!(stderr(), "caused by: {}", cause);
    }
}

// Error-checked entry-point
fn run() -> Result<()> {
    // Grab a handle to STDOUT in raw mode (no auto printing or buffering)
    let stdout = io::stdout().into_raw_mode()?;
    // Enable mouse
    let stdout = MouseTerminal::from(stdout);
    // Use a separate overlay over existing terminal.  On quit, old terminal is restored.
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
        // 4. `right` enters the currently selected directory, or takes no action on files.
        if let Event::Input(input) = events.next()? {
            use app::{AppMode, InputType};
            match app.mode {
                AppMode::Nav => match input {
                    Key::Char('q') => break,
                    Key::Left | Key::Char('a') => app.dir_list.unselect(),
                    Key::Right | Key::Char('d') | Key::Char('\n') => app.enter_selected()?,
                    Key::Down | Key::Char('s') => app.dir_list.next(),
                    Key::Up | Key::Char('w') => app.dir_list.previous(),
                    Key::Char('p') => app.mode = AppMode::Input(InputType::Permission),
                    Key::Char('c') => app.mode = AppMode::Input(InputType::CopyFile),
                    Key::Char('j') => app.mode = AppMode::Input(InputType::ChangeDir),
                    _ => {} // Ignore all other key inputs
                },
                AppMode::Input(input_type) => match input {
                    Key::Char('\n') => {
                        let user_input = app.user_input.drain(..).collect::<String>();
                        match input_type {
                            InputType::CopyFile => {
                                app.copy_selected(PathBuf::from(&user_input).as_path())?
                            }
                            InputType::ChangeDir => {
                                app.change_dir(PathBuf::from(&user_input).as_path())?
                            }
                            InputType::Permission => app.set_permissions(&user_input)?,
                        }
                        app.mode = AppMode::default();
                    }
                    Key::Char(c) => {
                        app.user_input.push(c);
                    }
                    Key::Backspace => {
                        app.user_input.pop();
                    }
                    Key::Esc => {
                        let _ = app.user_input.drain(..);
                        app.mode = AppMode::Nav;
                    }
                    _ => {}
                },
            }
        }
    }

    Ok(())
}

// Executable entrypoint
fn main() {
    // If any error occurs display it before quitting
    if let Err(e) = run() {
        print_error(e);
    }
}

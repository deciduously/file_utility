//! Linux File Utility Program
//! Simple TUI for exploring a filesystem
//! Benjamin Lovy
//! July 18, 2021
//! SDEV-345
//! Professor Gary Savard

//! This module imperatively defines the user interface.  It is computed every frame tick.

use crate::app::App;

// Convert a relative path to absolute.
use std::fs::canonicalize;

// TUI widget library
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

// The usage text isn't dynamic in any way.
const USAGE_TEXT: &str = "Use the up and down arrows to navigate the list.  Use the left arrow to unselect all.  Use `q` to quit the program.";

/// Helper function to build a block
fn create_block<'a>(title: &'a str) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
}

/// Render the whole application.
pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // First, define the layout.  Each chunk is a location where we can render a widget
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(f.size());

    draw_explorer(f, app, chunks[0]);
    draw_usage(f, app, chunks[1]);
}

fn draw_explorer<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    draw_dir_list(f, app, chunks[0]);
    draw_details(f, app, chunks[1]);
}

/// Render the selectable directory listing pane.
fn draw_dir_list<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    // Add text to each result that's a directory
    let items: Vec<ListItem> = app
        .dir_list
        .items
        .iter()
        .map(|i| {
            let mut lines = vec![Spans::from(i.0.to_string())];
            if i.0.is_directory {
                lines.push(Spans::from(Span::styled(
                    "directory",
                    Style::default().add_modifier(Modifier::ITALIC),
                )));
            }
            // Push the full text to the list
            ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    // The block title will show the current directory
    let listing_title = format!(
        "Directory Listing | {:?}",
        canonicalize(&app.current_directory)
            .expect("Could not get absolute path from relative path")
    );

    // Create a List from all items, highlight the selected one
    let items = List::new(items)
        .block(create_block(&listing_title))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // Render the item list
    f.render_stateful_widget(items, area, &mut app.dir_list.state);
}

/// Render the details pane.
fn draw_details<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{

    // The next widget is the details block, which displays information about the highlighted entry.
    let detail_text = if let Some(listing) = app.dir_list.grab_selected() {
        let detail = listing
            .0
            .detail_string()
            .unwrap_or_else(|_| "Could not read metadata".to_string());

        // It also grabs the contents
        let contents = if !listing.0.is_directory {
            listing
                .0
                .contents()
                .unwrap_or_else(|_| "Could not read file contents.".to_string())
        } else {
            "<directory>".to_string()
        };

        format!("{}\nFile Contents:\n{}", detail, contents)
    } else {
        "Nothing selected.".to_string()
    };

    let detail = Paragraph::new(detail_text)
        .style(Style::default())
        .block(create_block("Detail"));
    f.render_widget(detail, area);
}

/// Render the usage panel.
fn draw_usage<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
where
    B: Backend,
{
    // Finally, on the bottom, we want to render usage instructions
    let usage = Paragraph::new(USAGE_TEXT)
        .style(Style::default())
        .block(create_block("Usage"));
    f.render_widget(usage, area);
}

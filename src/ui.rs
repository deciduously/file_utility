//! This module imperatively defines the user interface.  It is computed every frame tick.

use crate::app::App;

// Convert a relative path to absolute.
use std::fs::canonicalize;

// TUI widget library
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

// The usage text isn't dynamic in any way.
const USAGE_TEXT: &str = "\u{1F815}/w: up \u{1F817}/s: down \u{1F816}/d: enter directory \u{1F814}/a: unselect all\nc: copy file j: jump to directory p: change permissions\nq: quit";

/// Helper function to build a block
fn create_block(title: &str) -> Block {
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
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
        .split(f.size());

    draw_dir_list(f, app, chunks[0]);
    draw_left_panel(f, app, chunks[1]);
}

///
fn draw_left_panel<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5)].as_ref())
        .split(area);

    draw_details(f, app, chunks[0]);
    draw_usage(f, app, chunks[1]);
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
            let mut spans = vec![];
            if i.1 == 0 {
                spans.push(Span::from("."));
            } else if i.1 == 1 {
                spans.push(Span::from(".."));
            } else {
                spans.push(Span::from(i.0.to_string()));
                if i.0.is_directory {
                    spans.push(Span::styled(
                        " - directory",
                        Style::default().add_modifier(Modifier::ITALIC),
                    ));
                }
            }
            // Push the full text to the list
            ListItem::new(Spans::from(spans))
                .style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    // The block title will show the current directory
    let absolute = canonicalize(&app.current_directory)
        .expect("Could not get absolute path from relative path");
    let listing_title = &absolute.to_str().unwrap_or("\"???\"")[..absolute.to_str().unwrap().len()];

    // Create a List from all items, highlight the selected one
    let items = List::new(items)
        .block(create_block(listing_title))
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
        let contents = if let Some(s) = listing.0.contents().unwrap_or(None) {
            format!("File Contents:\n{}", s)
        } else {
            "".to_string()
        };

        format!("{}\n\n{}", detail, contents)
    } else {
        "Nothing selected.".to_string()
    };

    let detail = Paragraph::new(detail_text)
        .style(Style::default())
        .block(create_block("Detail"));
    f.render_widget(detail, area);
}

/// Render the usage panel.
fn draw_usage<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    use crate::app::AppMode;
    match &app.mode {
        AppMode::Nav => {
            // Finally, on the bottom, we want to render usage instructions
            let usage = Paragraph::new(Text::from(USAGE_TEXT))
                .style(Style::default())
                .block(create_block("Usage"));
            f.render_widget(usage, area);
        }
        AppMode::Input(input_type) => {
            let input = Paragraph::new(app.user_input.as_ref())
                .style(Style::default().fg(Color::Yellow))
                .block(create_block(input_type.message()));
            f.render_widget(input, area);
            f.set_cursor(area.x + app.user_input.width() as u16 + 1, area.y + 1);
        }
    }
}

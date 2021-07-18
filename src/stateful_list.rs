//! Linux File Utility Program
//! Simple TUI for exploring a filesystem
//! Benjamin Lovy
//! July 18, 2021
//! SDEV-345
//! Professor Gary Savard

//! A StatefulList holds a vector as well as extra state about which item is selected, if any.
//! Adapted from https://github.com/fdehau/tui-rs/blob/master/examples/util/mod.rs

use tui::widgets::ListState;

/// Associates a ListState with a Vec<T> that tracks which item is selected.
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn new() -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn grab_selected(&self) -> Option<&T> {
        if let Some(idx) = self.state.selected() {
            Some(&self.items[idx])
        } else {
            None
        }
    }
}

//! Linux File Utility Program
//! Simple TUI for exploring a filesystem
//! Benjamin Lovy
//! July 18, 2021
//! SDEV-345
//! Professor Gary Savard

//! This module defines the application state

// Ergonomic Result and Error types to simply error handling boilerplate
use anyhow::Result;

use crate::stateful_list::StatefulList;

// Input and output (stdio, stderr, etc), OS integration, type conversions
use std::{
    fmt,
    fs::{self, canonicalize, File},
    io::{prelude::*, BufReader},
    os::unix::fs::PermissionsExt, // Unix-specific st_mode
    path::{Path, PathBuf},
    str::FromStr,
    time::SystemTime,
};

/// Each displayed entry stores some information about itself
#[derive(Debug)]
pub struct FileListing {
    path: PathBuf,
    pub is_directory: bool,
}

impl FileListing {
    // Constructor
    pub fn new(path: PathBuf, is_directory: bool) -> Self {
        Self { path, is_directory }
    }

    /// Get the file contents as a string
    pub fn contents(&self) -> Result<Option<String>> {
        if self.is_directory {
            Ok(None)
        } else {
            // Open the file, store the descriptor
            let f = File::open(&self.path)?;
            // Initialize a buffered reader
            let mut buf_reader = BufReader::new(f);
            // Initialize return string
            let mut result = String::new();
            // Read entire file to result string
            buf_reader.read_to_string(&mut result)?;
            // Return result
            Ok(Some(result))
        }
    }

    /// Returns a multi-line string to render in the detail tab when the file is selected.
    pub fn detail_string(&self) -> Result<String> {
        // Read the metadata
        let m = fs::metadata(&self.path)?;

        let d_or_f = if m.is_dir() {
            "directory"
        } else {
            "regular file"
        };
        let len = m.len(); // in bytes
        let permissions = m.permissions().mode();

        // Helper closure to unwrap times which may not come back.
        let unwrap_time = |r: std::result::Result<SystemTime, std::io::Error>| {
            if let Ok(ts) = r {
                return format!("{:?}", ts);
            } else {
                return "never".to_string();
            };
        };

        let last_modified = unwrap_time(m.modified());
        let last_accessed = unwrap_time(m.accessed());
        // FIXME: this is returning an error for all files, unclear why?
        //let created = unwrap_time(m.created());

        let result = format!("Path {:?} is a {}.\nSize: {} bytes\nLast modified: {}\nLast accessed: {}\nPermissions (as st_mode): {:o}",
        self.path, d_or_f, len, last_accessed, last_modified, permissions
    );

        Ok(result)
        // Rust automatically closes the file at the end of the block here
    }
}

// Pretty-printing - provides to_string() method as well
impl fmt::Display for FileListing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.path.as_path().file_name().unwrap().to_str().unwrap()
        ) // truncate the leading "./" and trailing "
    }
}

/// Return a StatefulList containing all files in the given directory/
fn list_of_dir(path: &Path) -> Result<StatefulList<(FileListing, usize)>> {
    // This only makes sense if the path is a directory
    let result: Vec<(FileListing, usize)> = if path.is_dir() {
        let mut vec = Vec::new();
        let canonicalized = canonicalize(path).unwrap();
        let parent = canonicalized
            .as_path()
            .parent()
            .unwrap_or_else(|| canonicalized.as_path());

        // First, we'll always push an entry for "." and ".."
        vec.push((FileListing::new(canonicalized.clone(), true), 0));
        vec.push((FileListing::new(parent.to_path_buf(), true), 1));

        for (idx, entry) in fs::read_dir(path)?.enumerate() {
            // Unwrap entry
            let entry = entry?;
            // Grab path and check if its a directory
            let path = entry.path();
            let is_dir = path.is_dir();
            // Add the result to the return vector
            vec.push((FileListing::new(path, is_dir), idx + 2))
        }
        vec
    } else {
        vec![]
    };

    Ok(StatefulList::with_items(result))
}

/// The current state of the app.
/// The dir_list tracks information about which entry is selected, via ListState.
/// The events are used to mutate the state.
pub struct App {
    pub current_directory: PathBuf,
    pub dir_list: StatefulList<(FileListing, usize)>,
}

impl App {
    pub fn new() -> Self {
        let default_path = PathBuf::from_str(".").expect("Should read current directory");
        let dir_list =
            list_of_dir(&default_path).expect("Should enumerate current directory listing");
        Self {
            current_directory: default_path,
            dir_list,
        }
    }

    /// Change the active directory
    pub fn change_dir(&mut self, path: &Path) -> Result<()> {
        self.current_directory = path.to_path_buf();
        self.dir_list = list_of_dir(&self.current_directory).unwrap();
        Ok(())
    }

    /// Changes the current directory to whichever is selected, if any.  Takes no action if none.
    pub fn enter_selected(&mut self) -> Result<()> {
        if let Some((listing, _)) = self.dir_list.grab_selected() {
            if listing.is_directory {
                let new_path = listing.path.clone();
                self.change_dir(&new_path)?;
            }
        } else {
            // Do nothing!
        }
        Ok(())
    }
}

//! Linux File Utility Program
//! Simple TUI for exploring a filesystem
//! Benjamin Lovy
//! July 18, 2021
//! SDEV-345
//! Professor Gary Savard

//! This module defines the application state

// Ergonomic Result and Error types to simply error handling boilerplate
use anyhow::Result;

// For parsing/serializing file permissions
use libc::{S_IRGRP, S_IROTH, S_IRUSR, S_IWGRP, S_IWOTH, S_IWUSR, S_IXGRP, S_IXOTH, S_IXUSR};

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

/// Pretty-print a permissions triplet into a human-readable string component
fn triplet(mode: u16, read: u16, write: u16, execute: u16) -> String {
    match (mode & read, mode & write, mode & execute) {
        (0, 0, 0) => "---",
        (_, 0, 0) => "r--",
        (0, _, 0) => "-w-",
        (0, 0, _) => "--x",
        (_, 0, _) => "r-x",
        (_, _, 0) => "rw-",
        (0, _, _) => "-wx",
        (_, _, _) => "rwx",
    }
    .to_string()
}

/// Parse an st_mode bitfield into a human-readable permission string
fn permissions_to_string(mode: u16) -> String {
    let user = triplet(mode, S_IRUSR as u16, S_IWUSR as u16, S_IXUSR as u16);
    let group = triplet(mode, S_IRGRP as u16, S_IWGRP as u16, S_IXGRP as u16);
    let other = triplet(mode, S_IROTH as u16, S_IWOTH as u16, S_IXOTH as u16);
    [user, group, other].join("")
}

/// Get the correct u32 mode value from a rwxrwxrwx permission string
fn string_to_permissions(s: &str) -> Option<u16> {
    if s.len() == 9 {
        Some(s.chars().collect::<Vec<char>>().chunks(3).enumerate().fold(
            0,
            |mut acc, (idx, triple)| match idx {
                0 => {
                    if triple[0] == 'r' {
                        acc |= S_IRUSR as u16;
                    }
                    if triple[1] == 'w' {
                        acc |= S_IWUSR as u16;
                    }
                    if triple[2] == 'x' {
                        acc |= S_IXUSR as u16;
                    }
                    acc
                }
                1 => {
                    if triple[0] == 'r' {
                        acc |= S_IRGRP as u16;
                    }
                    if triple[1] == 'w' {
                        acc |= S_IWGRP as u16;
                    }
                    if triple[2] == 'x' {
                        acc |= S_IXGRP as u16;
                    }
                    acc
                }
                2 => {
                    if triple[0] == 'r' {
                        acc |= S_IROTH as u16;
                    }
                    if triple[1] == 'w' {
                        acc |= S_IWOTH as u16;
                    }
                    if triple[2] == 'x' {
                        acc |= S_IXOTH as u16;
                    }
                    acc
                }
                _ => unreachable!(),
            },
        ))
    } else {
        None
    }
}

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

        let permissions = permissions_to_string(m.permissions().mode() as u16);

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

        let result = format!("Path {:?} is a {}.\nSize: {} bytes\nLast modified: {}\nLast accessed: {}\nPermissions: {}",
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

/// The application has a user input secondary mode
#[derive(Debug, PartialEq)]
pub enum AppMode {
    Default,
    Input(InputType),
}

/// There are several possible input types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputType {
    Permission,
    Copy,
    ChangeDir,
}

impl InputType {
    // Prompt text to display in each input mode
    pub fn message(&self) -> &'static str {
        match self {
            InputType::ChangeDir => "Enter destination directory",
            InputType::Copy => "Enter target",
            InputType::Permission => "Enter permission string from --------- to rwxrwxrwx",
        }
    }
}

impl Default for AppMode {
    fn default() -> Self {
        AppMode::Default
    }
}

/// The current state of the app.
/// The dir_list tracks information about which entry is selected, via ListState.
/// The events are used to mutate the state.
pub struct App {
    pub user_input: String,
    pub app_mode: AppMode,
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
            app_mode: AppMode::default(),
            user_input: String::new(),
        }
    }

    /// Change the active directory
    pub fn change_dir(&mut self, path: &Path) -> Result<()> {
        self.current_directory = path.to_path_buf();
        self.dir_list = list_of_dir(&self.current_directory).unwrap();
        Ok(())
    }

    /// Copy the selected file to the target location
    pub fn copy_selected(&mut self, target: &Path) -> Result<()> {
        if target.is_dir() {
            // FIXME - this should really grab the filename and build a new path
            return Ok(())
        }
        // First, create the destination.
        let mut target = File::create(target)?;

        // Grab the input contents
        let contents = if let Some((selected, _)) = self.dir_list.grab_selected() {
            selected
                .contents()
                .unwrap_or_default()
                .unwrap_or_else(String::new)
        } else {
            String::new()
        };

        // Write contents to file
        write!(target, "{}", contents)?;
        // Re-read directory list
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
        }
        Ok(())
    }

    /// Attempt to change the currently selected file's permission string
    pub fn set_permissions(&mut self, new_perms: &str) -> Result<()> {
        if let Some((listing, _)) = self.dir_list.grab_selected() {
            if !listing.is_directory {
                let f = File::open(&listing.path)?;
                let m = f.metadata()?;
                let mut permissions = m.permissions();
                if let Some(new_mode) = string_to_permissions(new_perms) {
                    permissions.set_mode(new_mode as u32);
                }
                f.set_permissions(permissions)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_roundtrip_permissions() {
        let str = "---rwxr-x";
        assert_eq!(
            str,
            permissions_to_string(string_to_permissions(str).unwrap())
        );
    }
}

# file_utility

A simple TUI file utility.  This program is only designed to run and compile on Linux.

![screenshot](https://github.com/deciduously/file_utility/blob/main/assets/screen_0.png)

## Usage

1. [Download](https://github.com/deciduously/file_utility/releases) or otherwise obtain the compiled executable.
1. Make it executable: `$ chmod +x file_utility`.
1. Place it somewhere in your PATH: e.g. `$ mv file_utility $HOME/bin`.
1. Navigate to the direcctory where you'd like the program to start: e.g. `$ cd ~`.
1. Execute the program: `file_utility`.

The provided executable was compiled with [`musl libc`](https://musl.libc.org/) using the [`x86_64-unknown-linux-musl`](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html) target and thus fully statically linked for maximum compatibility, but I've only tested this on Fedora 34 and Debian 10.

## Development

This program is implemented in Rust, using a handful of dependencies found on [crates.io](https://crates.io/).

### Requirements

* Stable [Rust](https://www.rust-lang.org/tools/install) - the default stable toolchain is fine.  Obtainable via rustup using the instructions at this link.

### Build

Clone or download this repository.  Enter the project directory containing `Cargo.toml` and execute `cargo run` to compile and execute the program.  The resulting executable will be located at `target/x86_64-unknown-linux-musl/`

### Crates

* [anyhow](https://github.com/dtolnay/anyhow) - Ergonomic error handling
* [termion](https://gitlab.redox-os.org/redox-os/termion) - Low-level terminal interface (like ncurses but not)
* [tui-rs](https://github.com/fdehau/tui-rs) - Widget-based terminal user interface library.
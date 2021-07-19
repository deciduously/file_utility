# file_utility

A simple TUI file utility.  This program is only designed to run and compile on 64-bit Linux.

![screenshot](https://github.com/deciduously/file_utility/blob/main/assets/screen_0.png)

## Usage

1. [Download](https://github.com/deciduously/file_utility/releases) or otherwise obtain the compiled executable.
1. Make it executable: `$ chmod +x file_utility`.
1. Place it somewhere in your PATH: e.g. `$ mv file_utility $HOME/bin`.
1. Navigate to the directory where you'd like the program to start: e.g. `$ cd ~`.
1. Execute the program: `file_utility`.

You can use the `Esc` key to abort text-input mode.

If you don't see the Usage panel at the bottom, try resizing your terminal window.  It usually just works but if that seems to fix it if not.

I've only tested this on Fedora 34 and Debian 10, but it is fully statically linked and I would expect it to work on many other 64-bit Linux systems.

## Development

This program is implemented in Rust, using a handful of dependencies found on [crates.io](https://crates.io/).

### Requirements

* **Stable [Rust](https://www.rust-lang.org/tools/install)**:  The default stable toolchain is fine.  Obtainable via `rustup` using the instructions at this link.
* **`x86_64-unknown-linux-musl`** target: Once `rustup` is installed and ready to use, execute `rustup target add x86_64-unknown-linux-musl`.  Alternatively, you could delete or rename the `.cargo/config.toml` which forces this target.  This program should also build for the default Linux target and run fine on your machine.  To ensure this binary is as portable as possible, this repository is configured to always compile for this alternate target.  In default builds, Rust statically links all your dependenices but still dynamically links to the system `libc`, which means incompatible versions may prevent the executable from running.  Using this alternate `libc` alows even this to be statically linked, removing the runtime dependency on a minumum installed version.

### Build

Clone or download this repository.  Enter the project directory containing `Cargo.toml` and execute `cargo run` to compile and execute the program.  The resulting executable will be located at `target/x86_64-unknown-linux-musl/debug/file_utility`.  To compile with release mode, add the `--release` flag.  Use `cargo build` to build the binary without running it.  Use `cargo test` to run tests.

### Crates

* [anyhow](https://github.com/dtolnay/anyhow) - Ergonomic error handling.
* [libc](https://github.com/rust-lang/libc) - FFI bindings to libc.
* [termion](https://gitlab.redox-os.org/redox-os/termion) - Low-level terminal interface (like ncurses but not).
* [tui-rs](https://github.com/fdehau/tui-rs) - Widget-based terminal user interface library.
* [unicode-width](https://unicode-rs.github.io/unicode-width/unicode_width/index.html) - Unicode string width on screen.

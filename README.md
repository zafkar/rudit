# Rudit

Rudit is a lightweight terminal-based text editor written in Rust.

## Features

- Minimal and fast
- Basic text editing functionality
- Keyboard shortcuts for navigation and editing
- Mouse navigation

## Installation

Ensure you have Rust installed.

```sh
cargo install --git https://github.com/zafkar/rudit.git
```

## Usage

```help
A simple rust editor

Usage: rudit.exe [OPTIONS] [FILE] [COMMAND]

Commands:
  config  Print the default config
  help    Print this message or the help of the given subcommand(s)

Arguments:
  [FILE]  The file to edit

Options:
  -c, --config <FILE>  Sets a custom config file
  -h, --help           Print help
  -V, --version        Print version
```

## Controls

The default controls can be found in the config file.

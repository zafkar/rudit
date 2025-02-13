use anyhow::Result;
use buffer::Buffer;
use clap::{arg, command};
use crossterm::{
    event::{self},
    execute,
    style::{self},
    terminal,
};
use editor::Editor;
use std::io::stdout;

mod buffer;
mod config;
mod editor;
mod pos;

fn main() -> Result<()> {
    let matches = command!()
        .args([arg!([FILE] "The file to edit")])
        .get_matches();

    let mut stdout = stdout();
    terminal::enable_raw_mode()?;

    let mut editor = Editor::new();
    if let Some(path) = matches.get_one::<String>("FILE") {
        editor = editor.set_buffer(Buffer::load_from_file(path)?);
    }

    {
        let (width, height) = terminal::size()?;
        editor.set_size(width, height);
    }

    while !editor.is_done() {
        editor.process_event(event::read()?)?;
        editor.display(&mut stdout)?;
    }

    execute!(stdout, style::ResetColor)?;

    Ok(())
}

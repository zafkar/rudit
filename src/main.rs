use anyhow::Result;
use buffer::Buffer;
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
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;

    let mut editor = Editor::new();
    editor = editor.set_buffer(Buffer::load_from_file("src/main.rs")?);
    {
        let (width, height) = terminal::size()?;
        editor.set_size(width, height);
    }

    while !editor.is_done() {
        editor.process_event(event::read()?);
        editor.display(&mut stdout)?;
    }

    execute!(stdout, style::ResetColor)?;

    Ok(())
}

use std::{
    collections::VecDeque,
    io::{stdout, Write},
};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self},
    execute, queue,
    style::{self},
    terminal,
};

fn main() -> Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;

    let (mut width, mut height) = terminal::size()?;

    let mut buffer = VecDeque::new();
    buffer.push_back(String::from("Hellow World !"));

    execute!(
        stdout,
        event::EnableBracketedPaste,
        event::EnableFocusChange,
        event::EnableMouseCapture,
        terminal::Clear(terminal::ClearType::All),
        cursor::EnableBlinking,
        cursor::MoveTo(0, 0),
        style::SetForegroundColor(style::Color::Rgb {
            r: 0x00,
            g: 0xd9,
            b: 0x07
        }),
        style::SetBackgroundColor(style::Color::Rgb {
            r: 0xb1,
            g: 0x62,
            b: 0x86
        }),
    )?;

    loop {
        buffer.push_back(match event::read()? {
            event::Event::FocusGained => String::from("focus"),
            event::Event::FocusLost => String::from("unfocus"),
            event::Event::Key(key_event) => match key_event.code {
                event::KeyCode::Esc => break,
                code => format!("{}", code),
            },
            event::Event::Mouse(mouse_event) => format!(
                "{},{},{},{:?}",
                mouse_event.column, mouse_event.row, mouse_event.modifiers, mouse_event.kind
            ),
            event::Event::Paste(paste) => format!("paste => {paste}"),
            event::Event::Resize(new_width, new_height) => {
                width = new_width;
                height = new_height;
                format!("{width},{height}")
            }
        });

        queue!(stdout, terminal::Clear(terminal::ClearType::All))?;
        let buffer_len = buffer.len();
        for i in 0..height {
            if let Some(line) =
                buffer.get(buffer_len.checked_sub(height as usize).unwrap_or_default() + i as usize)
            {
                queue!(stdout, cursor::MoveTo(0, i), style::Print(line.clone()))?;
            }
        }

        stdout.flush()?;
    }

    execute!(stdout, style::ResetColor)?;

    Ok(())
}

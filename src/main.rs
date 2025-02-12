use anyhow::Result;
use buffer::Buffer;
use crossterm::{
    cursor,
    event::{self, KeyEventKind},
    execute, queue,
    style::{self},
    terminal,
};
use pos::Pos;
use std::io::{stdout, Write};

mod buffer;
mod pos;

fn main() -> Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;

    let (mut width, mut height) = terminal::size()?;

    let mut buffer = Buffer::load_from_file("src/main.rs")?;
    let mut scroll = Pos::default();

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
        match event::read()? {
            event::Event::FocusGained => (),
            event::Event::FocusLost => (),
            event::Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        event::KeyCode::Esc => break,
                        event::KeyCode::Up => {
                            buffer.move_up();
                            let new_pos = buffer.get_viewport_pos(scroll);
                            scroll = cap_scroll(scroll, buffer.get_cursor(), width, height);
                            queue!(stdout, cursor::MoveTo(new_pos.x as u16, new_pos.y as u16))?;
                        }
                        event::KeyCode::Down => {
                            buffer.move_down();
                            let new_pos = buffer.get_viewport_pos(scroll);
                            scroll = cap_scroll(scroll, buffer.get_cursor(), width, height);
                            queue!(stdout, cursor::MoveTo(new_pos.x as u16, new_pos.y as u16))?;
                        }
                        _ => (),
                    }
                }
            }
            event::Event::Mouse(_mouse_event) => (),
            event::Event::Paste(_paste) => (),
            event::Event::Resize(new_width, new_height) => {
                width = new_width;
                height = new_height;
            }
        };

        queue!(
            stdout,
            cursor::SavePosition,
            terminal::Clear(terminal::ClearType::All)
        )?;

        for (index, line) in buffer
            .get_viewport(scroll.x, scroll.y, width as usize, height as usize - 1)
            .iter()
            .enumerate()
        {
            queue!(
                stdout,
                cursor::MoveTo(0, index as u16),
                style::Print(format!("{line}"))
            )?;
        }

        queue!(
            stdout,
            cursor::MoveTo(0, height - 1),
            terminal::Clear(terminal::ClearType::CurrentLine),
            style::SetAttribute(style::Attribute::Dim),
            style::Print(format!("{scroll:?}")),
            style::SetAttribute(style::Attribute::NormalIntensity)
        )?;

        queue!(stdout, cursor::RestorePosition)?;

        stdout.flush()?;
    }

    execute!(stdout, style::ResetColor)?;

    Ok(())
}

fn cap_scroll(current_scroll: Pos, buffer_cursor: Pos, width: u16, height: u16) -> Pos {
    let width = width as usize;
    let height = height as usize;

    let x = if buffer_cursor.x < current_scroll.x {
        buffer_cursor.x
    } else if buffer_cursor.x > (current_scroll.x + width) {
        current_scroll.x + (current_scroll.x + width - buffer_cursor.x)
    } else {
        current_scroll.x
    };
    let y = if buffer_cursor.y < current_scroll.y {
        buffer_cursor.y
    } else if buffer_cursor.y > (current_scroll.y + height) {
        buffer_cursor.y - height
    } else {
        current_scroll.y
    };
    Pos::new(x, y)
}

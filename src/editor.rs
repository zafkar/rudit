use std::io::{Stdout, Write};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyEventKind},
    queue, style, terminal,
};

use crate::{buffer::Buffer, pos::Pos};

#[derive(Debug, Clone)]
pub struct Editor {
    scroll: Pos,
    size: Pos,
    buffer: Buffer,
    state: EditorState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EditorState {
    Init,
    Normal,
    Close,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            scroll: Pos::default(),
            size: Pos::default(),
            buffer: Buffer::default(),
            state: EditorState::Init,
        }
    }

    pub fn is_done(&self) -> bool {
        self.state == EditorState::Close
    }

    pub fn set_buffer(mut self, buffer: Buffer) -> Editor {
        self.buffer = buffer;
        self
    }

    pub fn set_size(&mut self, width: u16, height: u16) {
        self.size.x = width as usize;
        self.size.y = height as usize;
    }

    fn cap_scroll(&mut self) {
        let x = if self.buffer.get_cursor().x < self.scroll.x {
            self.buffer.get_cursor().x
        } else if self.buffer.get_cursor().x > (self.scroll.x + self.size.x) {
            self.scroll.x + (self.scroll.x + self.size.x - self.buffer.get_cursor().x)
        } else {
            self.scroll.x
        };
        let y = if self.buffer.get_cursor().y < self.scroll.y {
            self.buffer.get_cursor().y
        } else if self.buffer.get_cursor().y > (self.scroll.y + self.size.y) {
            self.buffer.get_cursor().y - self.size.y
        } else {
            self.scroll.y
        };
        self.scroll = Pos::new(x, y);
    }

    fn init(&mut self, stdout: &mut Stdout) -> Result<()> {
        queue!(
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

        self.state = EditorState::Normal;
        Ok(())
    }

    pub fn process_event(&mut self, event: Event) {
        match event {
            event::Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        event::KeyCode::Esc => self.state = EditorState::Close,
                        event::KeyCode::Up => {
                            self.buffer.move_up();
                            self.cap_scroll();
                        }
                        event::KeyCode::Down => {
                            self.buffer.move_down();
                            self.cap_scroll();
                        }
                        _ => (),
                    }
                }
            }
            event::Event::Resize(width, height) => {
                self.set_size(width, height);
            }
            _ => (),
        };
    }

    pub fn display(&mut self, stdout: &mut Stdout) -> Result<()> {
        if self.state == EditorState::Init {
            self.init(stdout)?;
        }

        let viewport_pos = self.buffer.get_viewport_pos(self.scroll);
        queue!(
            stdout,
            cursor::MoveTo(viewport_pos.x as u16, viewport_pos.y as u16),
            cursor::SavePosition,
            terminal::Clear(terminal::ClearType::All)
        )?;

        for (index, line) in self
            .buffer
            .get_viewport(self.scroll.x, self.scroll.y, self.size.x, self.size.y - 1)
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
            cursor::MoveTo(0, self.size.y as u16 - 1),
            terminal::Clear(terminal::ClearType::CurrentLine),
            style::SetAttribute(style::Attribute::Dim),
            style::Print(format!("{:?}", self.scroll)),
            style::SetAttribute(style::Attribute::NormalIntensity)
        )?;

        queue!(stdout, cursor::RestorePosition)?;

        stdout.flush()?;
        Ok(())
    }
}

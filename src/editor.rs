use std::io::{Stdout, Write};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyEventKind},
    queue, style, terminal,
};

use crate::{buffer::Buffer, config::Config, pos::Pos};

#[derive(Debug, Clone)]
pub struct Editor {
    scroll: Pos,
    window_size: Pos,
    viewport_size: Pos,
    buffer: Buffer,
    state: EditorState,
    config: Config,
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
            window_size: Pos::default(),
            viewport_size: Pos::default(),
            buffer: Buffer::default(),
            state: EditorState::Init,
            config: Config::default(),
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
        self.window_size.x = width as usize;
        self.window_size.y = height as usize;
        self.viewport_size.x = width as usize;
        self.viewport_size.y = height as usize - 1;
    }

    fn cap_scroll(&mut self) {
        let x = if self.buffer.get_cursor().x < self.scroll.x {
            self.buffer.get_cursor().x
        } else if self.buffer.get_cursor().x > (self.scroll.x + self.viewport_size.x - 1) {
            self.buffer.get_cursor().x - self.viewport_size.x + 1
        } else {
            self.scroll.x
        };
        let y = if self.buffer.get_cursor().y < self.scroll.y {
            self.buffer.get_cursor().y
        } else if self.buffer.get_cursor().y > (self.scroll.y + self.viewport_size.y - 1) {
            self.buffer.get_cursor().y - self.viewport_size.y + 1
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
            style::SetForegroundColor(self.config.fg_color_buffer),
            style::SetBackgroundColor(self.config.bg_color_buffer),
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
                        event::KeyCode::Right => {
                            self.buffer.move_right();
                            self.cap_scroll();
                        }
                        event::KeyCode::Left => {
                            self.buffer.move_left();
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
            .get_viewport(self.scroll, self.viewport_size)
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
            cursor::MoveTo(0, self.window_size.y as u16 - 1),
            style::SetForegroundColor(self.config.fg_color_ui),
            style::SetBackgroundColor(self.config.bg_color_ui),
            terminal::Clear(terminal::ClearType::CurrentLine),
            style::Print(format!(
                "Cursor : {}, Scroll : {}",
                self.buffer.get_cursor(),
                self.scroll
            )),
            style::SetForegroundColor(self.config.fg_color_buffer),
            style::SetBackgroundColor(self.config.bg_color_buffer),
        )?;

        queue!(stdout, cursor::RestorePosition)?;

        stdout.flush()?;
        Ok(())
    }
}

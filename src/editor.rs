use std::{
    io::{Stdout, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyEventKind, KeyModifiers},
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
    filename: Option<PathBuf>,
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
            buffer: Buffer::new(),
            state: EditorState::Init,
            config: Config::default(),
            filename: None,
        }
    }

    pub fn is_done(&self) -> bool {
        self.state == EditorState::Close
    }

    pub fn set_document<P>(&mut self, path: P) -> Result<()>
    where
        P: AsRef<Path> + Clone,
        PathBuf: From<P>,
    {
        self.buffer = match Buffer::load_from_file(path.clone()) {
            Ok(buffer) => buffer,
            Err(_) => Buffer::new(),
        };
        self.filename = Some(path.into());
        Ok(())
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

    pub fn process_event(&mut self, event: Event) -> Result<()> {
        match event {
            event::Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.modifiers {
                        KeyModifiers::NONE => match key_event.code {
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
                            event::KeyCode::PageUp => {
                                self.buffer.move_cursor(
                                    self.buffer.get_cursor().x,
                                    self.buffer
                                        .get_cursor()
                                        .y
                                        .checked_sub(self.viewport_size.y - 1)
                                        .unwrap_or_default(),
                                );
                                self.cap_scroll();
                            }
                            event::KeyCode::PageDown => {
                                self.buffer.move_cursor(
                                    self.buffer.get_cursor().x,
                                    self.buffer.get_cursor().y + self.viewport_size.y - 1,
                                );
                                self.cap_scroll();
                            }
                            event::KeyCode::Enter => {
                                self.buffer.add_line_at_cursor()?;
                                self.cap_scroll();
                            }
                            event::KeyCode::Char(c) => {
                                self.buffer.add_str_at_cursor(format!("{}", c).as_str())?;
                            }
                            keycode => {
                                self.buffer
                                    .add_str_at_cursor(format!("{}", keycode).as_str())?;
                            }
                        },
                        KeyModifiers::CONTROL => match key_event.code {
                            event::KeyCode::Char('s') => {
                                if let Some(path) = self.filename.clone() {
                                    self.buffer.save_to_file(path)?;
                                }
                            }
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
            event::Event::Resize(width, height) => {
                self.set_size(width, height);
            }
            event::Event::Mouse(mouse_event) => match mouse_event.kind {
                event::MouseEventKind::ScrollDown => {
                    self.buffer.move_down();
                    self.cap_scroll();
                }
                event::MouseEventKind::ScrollUp => {
                    self.buffer.move_up();
                    self.cap_scroll();
                }
                event::MouseEventKind::Down(button) => match button {
                    event::MouseButton::Left => {
                        self.buffer.move_cursor(
                            mouse_event.column as usize + self.scroll.x,
                            mouse_event.row as usize + self.scroll.y,
                        );
                        self.cap_scroll();
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        };

        Ok(())
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
            style::SetForegroundColor(self.config.fg_color_buffer),
            style::SetBackgroundColor(self.config.bg_color_buffer),
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
                terminal::Clear(terminal::ClearType::CurrentLine),
                style::Print(format!("{line}"))
            )?;
        }

        // Displaying the UI
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
        )?;

        queue!(stdout, cursor::RestorePosition)?;

        stdout.flush()?;
        Ok(())
    }
}

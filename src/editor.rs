use std::{
    fs,
    io::{Stdout, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyEventKind},
    queue, style, terminal,
};
use serde::{Deserialize, Serialize};
use strum::EnumString;

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
    last_keypress: String,
    need_full_clear: bool,
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
            last_keypress: String::new(),
            need_full_clear: false,
        }
    }

    pub fn is_done(&self) -> bool {
        self.state == EditorState::Close
    }

    pub fn set_config<P>(&mut self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file_content = fs::read_to_string(path)?;
        self.config = toml::from_str(&file_content)?;
        Ok(())
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

    pub fn cleanup(stdout: &mut Stdout) -> Result<()> {
        queue!(
            stdout,
            event::DisableBracketedPaste,
            event::DisableFocusChange,
            event::DisableMouseCapture,
            style::ResetColor,
        )?;

        stdout.flush()?;

        Ok(())
    }

    fn init(&mut self, stdout: &mut Stdout) -> Result<()> {
        if self.config.use_mouse {
            queue!(stdout, event::EnableMouseCapture,)?;
        }

        if self.config.use_paste {
            queue!(stdout, event::EnableBracketedPaste,)?;
        }
        queue!(
            stdout,
            style::SetForegroundColor(self.config.fg_color_ui.into()),
            style::SetBackgroundColor(self.config.bg_color_ui.into()),
            terminal::Clear(terminal::ClearType::All),
            cursor::EnableBlinking,
            cursor::MoveTo(0, 0),
            style::SetForegroundColor(self.config.fg_color_buffer.into()),
            style::SetBackgroundColor(self.config.bg_color_buffer.into()),
        )?;

        self.state = EditorState::Normal;
        Ok(())
    }

    pub fn process_event(&mut self, event: Event) -> Result<()> {
        match event {
            event::Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    self.last_keypress = format!("{}{}", key_event.modifiers, key_event.code);
                    match self.config.keybindings.get(&self.last_keypress) {
                        Some(action) => match action {
                            EditorAction::Quit => self.state = EditorState::Close,
                            EditorAction::MoveUp => {
                                self.buffer.move_up();
                                self.cap_scroll();
                            }
                            EditorAction::MoveDown => {
                                self.buffer.move_down();
                                self.cap_scroll();
                            }
                            EditorAction::MoveRight => {
                                self.buffer.move_right();
                                self.cap_scroll();
                            }
                            EditorAction::MoveLeft => {
                                self.buffer.move_left();
                                self.cap_scroll();
                            }
                            EditorAction::PageUp => {
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
                            EditorAction::PageDown => {
                                self.buffer.move_cursor(
                                    self.buffer.get_cursor().x,
                                    self.buffer.get_cursor().y + self.viewport_size.y - 1,
                                );
                                self.cap_scroll();
                            }
                            EditorAction::SaveDocument => {
                                if let Some(path) = self.filename.clone() {
                                    self.buffer.save_to_file(path)?;
                                }
                            }
                            EditorAction::DeleteCharBack => {
                                self.buffer.delete_n_chars_back_from_cursor(1)?;
                                self.need_full_clear = true;
                                self.cap_scroll();
                            }
                            EditorAction::DeleteCharFront => {
                                self.buffer.delete_n_chars_front_from_cursor(1)?;
                                self.need_full_clear = true;
                                self.cap_scroll();
                            }
                        },
                        None => match key_event.code {
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

        if self.need_full_clear {
            queue!(stdout, terminal::Clear(terminal::ClearType::All))?;
        }

        let viewport_pos = self.buffer.get_viewport_pos(self.scroll);
        queue!(
            stdout,
            cursor::MoveTo(viewport_pos.x as u16, viewport_pos.y as u16),
            cursor::SavePosition,
            style::SetForegroundColor(self.config.fg_color_buffer.into()),
            style::SetBackgroundColor(self.config.bg_color_buffer.into()),
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
            style::SetForegroundColor(self.config.fg_color_ui.into()),
            style::SetBackgroundColor(self.config.bg_color_ui.into()),
            terminal::Clear(terminal::ClearType::CurrentLine),
            style::Print(format!(
                "Cursor : {}, Scroll : {}, Last Key Press : ({})",
                self.buffer.get_cursor(),
                self.scroll,
                self.last_keypress
            )),
        )?;

        queue!(stdout, cursor::RestorePosition)?;

        stdout.flush()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Deserialize, Serialize)]
pub enum EditorAction {
    Quit,
    MoveUp,
    MoveDown,
    MoveRight,
    MoveLeft,
    PageUp,
    PageDown,
    SaveDocument,
    DeleteCharBack,
    DeleteCharFront,
}

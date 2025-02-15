use anyhow::{anyhow, Context, Result};
use crossterm::{
    cursor,
    event::{self, Event, KeyEventKind},
    queue, style, terminal,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Stdout, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{buffer::Buffer, config::Config, pos::Pos};

#[derive(Debug, Clone)]
pub struct Editor {
    window_size: Pos,
    edit_buffer: Buffer,
    state: EditorState,
    config: Config,
    filename: Option<PathBuf>,
    last_keypress: String,
    need_full_clear: bool,
    command_buffer: Buffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EditorState {
    Init,
    EditMode,
    CommandMode,
    Close,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            window_size: Pos::default(),
            edit_buffer: Buffer::new(),
            state: EditorState::Init,
            config: Config::default(),
            filename: None,
            last_keypress: String::new(),
            need_full_clear: false,
            command_buffer: Buffer::new(),
        }
    }

    fn set_state(&mut self, state: EditorState) {
        self.state = state;
        self.update_layout(self.window_size);
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
        self.edit_buffer = match Buffer::load_from_file(path.clone()) {
            Ok(buffer) => buffer,
            Err(_) => Buffer::new(),
        };
        self.filename = Some(path.into());
        Ok(())
    }

    pub fn update_layout(&mut self, window_size: Pos) {
        self.window_size = window_size;

        //Update commandbuffer
        let command_buffer_viewport_size: Pos = match self.state {
            EditorState::EditMode => (window_size.x, 0).into(),
            EditorState::CommandMode => (
                window_size.x,
                (window_size.y.checked_sub(1).unwrap_or_default())
                    .min(self.command_buffer.content_lines_len()),
            )
                .into(),
            _ => self.command_buffer.get_viewport_size(),
        };
        self.command_buffer
            .set_viewport_size(command_buffer_viewport_size);

        self.command_buffer.set_top_left_corner(
            (
                0,
                window_size
                    .y
                    .checked_sub(1 + command_buffer_viewport_size.y)
                    .unwrap_or_default(),
            )
                .into(),
        );

        //Update editbuffer
        self.edit_buffer.set_viewport_size(match self.state {
            EditorState::EditMode => (
                window_size.x,
                window_size.y.checked_sub(1).unwrap_or_default(),
            )
                .into(),
            EditorState::CommandMode => (
                window_size.x,
                (window_size.y)
                    .checked_sub(command_buffer_viewport_size.y + 1)
                    .unwrap_or_default(),
            )
                .into(),
            _ => self.edit_buffer.get_viewport_size(),
        });

        self.need_full_clear = true;
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
            style::SetColors(self.config.color_status_bar.into()),
            terminal::Clear(terminal::ClearType::All),
            cursor::EnableBlinking,
            cursor::MoveTo(0, 0),
            style::SetColors(self.config.color_edit_zone.into()),
        )?;

        self.set_state(EditorState::EditMode);
        Ok(())
    }

    fn execute_command(&mut self, command: &EditorCommand) -> Result<()> {
        match command {
            EditorCommand::SetFilename(filename) => {
                self.filename = if filename.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(filename))
                }
            }
            EditorCommand::SaveAs(filename) => {
                self.filename = Some(PathBuf::from(filename));
                self.edit_buffer
                    .save_to_file(self.filename.clone().unwrap())?;
            }
        }
        Ok(())
    }

    pub fn process_event(&mut self, event: Event) -> Result<()> {
        match event {
            event::Event::Resize(width, height) => {
                self.update_layout((width, height).into());
                Ok(())
            }
            event => match self.state {
                EditorState::EditMode => self.process_event_edit_mode(event),
                EditorState::CommandMode => self.process_event_command_mode(event),
                _ => Ok(()),
            },
        }
    }

    fn process_event_command_mode(&mut self, event: Event) -> Result<()> {
        match event {
            event::Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    self.last_keypress = format!("{}{}", key_event.modifiers, key_event.code);
                    match self
                        .config
                        .command_keybindings
                        .clone()
                        .get(&self.last_keypress)
                    {
                        Some(action) => match action {
                            EditorAction::Quit => self.set_state(EditorState::Close),
                            EditorAction::MoveUp => {
                                self.command_buffer.move_up();
                            }
                            EditorAction::MoveDown => {
                                self.command_buffer.move_down();
                            }
                            EditorAction::MoveRight => {
                                self.command_buffer.move_right();
                            }
                            EditorAction::MoveLeft => {
                                self.command_buffer.move_left();
                            }
                            EditorAction::PageUp => {
                                self.command_buffer.move_up_n(
                                    self.command_buffer
                                        .get_viewport_size()
                                        .y
                                        .checked_sub(1)
                                        .unwrap_or_default(),
                                );
                            }
                            EditorAction::PageDown => {
                                self.command_buffer.move_down_n(
                                    self.command_buffer
                                        .get_viewport_size()
                                        .y
                                        .checked_sub(1)
                                        .unwrap_or_default(),
                                );
                            }
                            EditorAction::DeleteCharBack => {
                                self.command_buffer.delete_n_chars_back_from_cursor(1)?;
                                self.need_full_clear = true;
                            }
                            EditorAction::DeleteCharFront => {
                                self.command_buffer.delete_n_chars_front_from_cursor(1)?;
                                self.need_full_clear = true;
                            }
                            EditorAction::GoIntoEditMode => self.set_state(EditorState::EditMode),
                            EditorAction::DeleteAll => self.command_buffer.empty_content(),
                            EditorAction::Command(cmd) => self.execute_command(cmd)?,
                            EditorAction::GoIntoCommandMode => (),
                            EditorAction::SaveDocument => (),
                        },
                        None => match key_event.code {
                            event::KeyCode::Enter => {
                                self.execute_command(&EditorCommand::from_str(
                                    &self.command_buffer.get_contents(),
                                )?)?;
                                self.command_buffer.empty_content();
                                self.set_state(EditorState::EditMode);
                            }
                            event::KeyCode::Char(c) => {
                                self.command_buffer
                                    .add_str_at_cursor(format!("{}", c).as_str())?;
                            }
                            keycode => {
                                self.command_buffer
                                    .add_str_at_cursor(format!("{}", keycode).as_str())?;
                            }
                        },
                    }
                }
            }
            // event::Event::Mouse(mouse_event) => match mouse_event.kind {
            //     event::MouseEventKind::ScrollDown => {
            //         self.command_buffer.move_down();
            //
            //     }
            //     event::MouseEventKind::ScrollUp => {
            //         self.command_buffer.move_up();
            //
            //     }
            //     event::MouseEventKind::Down(button) => match button {
            //         event::MouseButton::Left => {
            //             self.command_buffer.move_cursor(
            //                 mouse_event.column as usize + self.scroll.x,
            //                 mouse_event.row as usize + self.scroll.y,
            //             );
            //
            //         }
            //         _ => (),
            //     },
            //     _ => (),
            // },
            _ => (),
        };

        Ok(())
    }

    fn process_event_edit_mode(&mut self, event: Event) -> Result<()> {
        match event {
            event::Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    self.last_keypress = format!("{}{}", key_event.modifiers, key_event.code);
                    match self
                        .config
                        .edit_keybindings
                        .clone()
                        .get(&self.last_keypress)
                    {
                        Some(action) => match action {
                            EditorAction::Quit => self.set_state(EditorState::Close),
                            EditorAction::MoveUp => {
                                self.edit_buffer.move_up();
                            }
                            EditorAction::MoveDown => {
                                self.edit_buffer.move_down();
                            }
                            EditorAction::MoveRight => {
                                self.edit_buffer.move_right();
                            }
                            EditorAction::MoveLeft => {
                                self.edit_buffer.move_left();
                            }
                            EditorAction::PageUp => {
                                self.edit_buffer.move_up_n(
                                    self.edit_buffer
                                        .get_viewport_size()
                                        .y
                                        .checked_sub(1)
                                        .unwrap_or_default(),
                                );
                            }
                            EditorAction::PageDown => {
                                self.edit_buffer.move_down_n(
                                    self.edit_buffer
                                        .get_viewport_size()
                                        .y
                                        .checked_sub(1)
                                        .unwrap_or_default(),
                                );
                            }
                            EditorAction::SaveDocument => {
                                if let Some(path) = self.filename.clone() {
                                    self.edit_buffer.save_to_file(path)?;
                                } else {
                                    self.command_buffer = Buffer::load_from_str("save_as ");
                                    self.command_buffer.move_line_end();
                                    self.set_state(EditorState::CommandMode);
                                }
                            }
                            EditorAction::DeleteCharBack => {
                                self.edit_buffer.delete_n_chars_back_from_cursor(1)?;
                                self.need_full_clear = true;
                            }
                            EditorAction::DeleteCharFront => {
                                self.edit_buffer.delete_n_chars_front_from_cursor(1)?;
                                self.need_full_clear = true;
                            }
                            EditorAction::GoIntoCommandMode => {
                                self.set_state(EditorState::CommandMode)
                            }
                            EditorAction::DeleteAll => self.edit_buffer.empty_content(),
                            EditorAction::Command(cmd) => self.execute_command(cmd)?,
                            EditorAction::GoIntoEditMode => (),
                        },
                        None => match key_event.code {
                            event::KeyCode::Enter => {
                                self.edit_buffer.add_line_at_cursor()?;
                            }
                            event::KeyCode::Char(c) => {
                                self.edit_buffer
                                    .add_str_at_cursor(format!("{}", c).as_str())?;
                            }
                            keycode => {
                                self.edit_buffer
                                    .add_str_at_cursor(format!("{}", keycode).as_str())?;
                            }
                        },
                    }
                }
            }
            event::Event::Mouse(mouse_event) => match mouse_event.kind {
                event::MouseEventKind::ScrollDown => {
                    self.edit_buffer.move_down();
                }
                event::MouseEventKind::ScrollUp => {
                    self.edit_buffer.move_up();
                }
                event::MouseEventKind::Down(button) => match button {
                    event::MouseButton::Left => {
                        self.edit_buffer.move_cursor_relative(
                            (mouse_event.column as usize, mouse_event.row as usize).into(),
                        );
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
            queue!(
                stdout,
                style::SetColors(self.config.color_status_bar.into()),
                terminal::Clear(terminal::ClearType::All)
            )?;
        }

        queue!(stdout, style::SetColors(self.config.color_edit_zone.into()))?;

        for (pos, line) in self.edit_buffer.get_viewport().iter() {
            queue!(
                stdout,
                cursor::MoveTo::from(*pos),
                terminal::Clear(terminal::ClearType::CurrentLine),
                style::Print(format!("{line}"))
            )?;
        }

        queue!(
            stdout,
            style::SetColors(self.config.color_command_zone.into()),
        )?;

        for (pos, line) in self.command_buffer.get_viewport().iter() {
            queue!(
                stdout,
                cursor::MoveTo::from(*pos),
                terminal::Clear(terminal::ClearType::CurrentLine),
                style::Print(format!("{line}"))
            )?;
        }

        // Displaying the UI
        queue!(
            stdout,
            cursor::MoveTo(0, self.window_size.y as u16 - 1),
            style::SetColors(self.config.color_status_bar.into()),
            terminal::Clear(terminal::ClearType::CurrentLine),
            style::Print(format!(
                "Filename : {:?}, Cursor : {}, Last Key Press : ({})",
                self.filename,
                self.edit_buffer.get_cursor(),
                self.last_keypress,
            )),
        )?;

        let terminal_cursor_pos = match self.state {
            EditorState::EditMode => self.edit_buffer.get_viewport_pos(),
            EditorState::CommandMode => self.command_buffer.get_viewport_pos(),
            _ => (0usize, 0).into(),
        };
        queue!(stdout, cursor::MoveTo::from(terminal_cursor_pos))?;

        stdout.flush()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
    DeleteAll,
    GoIntoCommandMode,
    GoIntoEditMode,
    Command(EditorCommand),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum EditorCommand {
    SetFilename(String),
    SaveAs(String),
}

impl FromStr for EditorCommand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let cmd = comma::parse_command(s).context("Unknown escape seq")?;
        match cmd.get(0).context("No command")?.as_str() {
            "set_filename" => Ok(EditorCommand::SetFilename(
                cmd.get(1).unwrap_or(&"".to_string()).clone(),
            )),
            "save_as" => Ok(EditorCommand::SaveAs(
                cmd.get(1).context("No filename")?.to_string(),
            )),
            _ => Err(anyhow!("Unknown command")),
        }
    }
}

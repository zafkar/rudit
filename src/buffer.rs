use std::{fs, path::Path};

use anyhow::{Context, Result};
use itertools::Itertools;

use crate::pos::Pos;

#[derive(Debug, Clone)]
pub struct Buffer {
    data: Vec<String>,
    cursor: Pos,
    endl: String,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            data: vec![],
            cursor: Pos::default(),
            endl: String::from("\n"),
        }
    }

    pub fn get_viewport_pos(&self, scroll: Pos) -> Pos {
        Pos::new(
            self.cursor.x.checked_sub(scroll.x).unwrap_or_default(),
            self.cursor.y.checked_sub(scroll.y).unwrap_or_default(),
        )
    }

    pub fn get_viewport(&self, scroll: Pos, size: Pos) -> Vec<String> {
        let mut viewport = vec![];
        for y in scroll.y..scroll.y + size.y {
            if let Some(line) = self.data.get(y) {
                viewport.push(line.to_string());
            }
        }
        viewport
    }

    pub fn move_up(&mut self) {
        self.move_cursor(
            self.cursor.x,
            self.cursor.y.checked_sub(1).unwrap_or(self.cursor.y),
        )
    }

    pub fn move_down(&mut self) {
        self.move_cursor(self.cursor.x, self.cursor.y + 1)
    }

    pub fn move_left_n(&mut self, n: usize) -> usize {
        let mut moved = 0;
        let mut cursor = self.cursor;
        while moved < n {
            if cursor == Pos::new(0, 0) {
                break;
            }
            if cursor.x == 0 {
                let prev_line_len = self
                    .data
                    .get(cursor.y - 1)
                    .map(|l| l.len())
                    .unwrap_or_default();
                cursor = Pos::new(prev_line_len, cursor.y - 1);
                moved += 1;
                continue;
            }
            if cursor.x >= (n - moved) {
                cursor.x -= n - moved;
                moved += n - moved;
            } else {
                moved += cursor.x;
                cursor = Pos::new(0, cursor.y);
            }
        }

        self.cursor = cursor;
        moved
    }

    pub fn move_right_n(&mut self, n: usize) -> usize {
        let mut moved = 0;
        let mut cursor = self.cursor;
        let last_cursor_y_pos = self.data.len().checked_sub(1).unwrap_or_default();
        let file_end_pos = Pos::new(
            self.data
                .get(last_cursor_y_pos)
                .map(|l| l.len())
                .unwrap_or_default(),
            last_cursor_y_pos,
        );
        while moved < n {
            if cursor == file_end_pos {
                break;
            }
            let current_line_len = self.data.get(cursor.y).map(|l| l.len()).unwrap();
            if cursor.x == current_line_len {
                cursor = Pos::new(0, cursor.y + 1);
                moved += 1;
                continue;
            }
            if (current_line_len - cursor.x) >= (n - moved) {
                cursor.x += n - moved;
                moved += n - moved;
            } else {
                moved += cursor.x;
                cursor = Pos::new(current_line_len, cursor.y);
            }
        }

        self.cursor = cursor;
        moved
    }

    pub fn move_left(&mut self) -> usize {
        self.move_left_n(1)
    }

    pub fn move_right(&mut self) -> usize {
        self.move_right_n(1)
    }

    pub fn move_cursor(&mut self, x: usize, y: usize) {
        let y = y.min(self.data.len().checked_sub(1).unwrap_or_default());
        let x = self.data.get(y).map(|l| l.len()).unwrap_or_default().min(x);
        self.cursor = Pos::new(x, y);
    }

    pub fn get_cursor(&self) -> Pos {
        self.cursor
    }

    pub fn add_str_at_cursor(&mut self, text: &str) -> Result<()> {
        let cursor = self.get_cursor();
        if let Some(line) = self.data.get_mut(cursor.y) {
            line.insert_str(cursor.x, text);
        } else {
            self.data.push(text.to_string());
        }
        self.move_cursor(cursor.x + text.len(), cursor.y);
        Ok(())
    }

    pub fn delete_n_chars_front_from_cursor(&mut self, n: usize) -> Result<()> {
        let moved = self.move_right_n(n);
        self.delete_n_chars_back_from_cursor(moved)
    }

    pub fn delete_n_chars_back_from_cursor(&mut self, n: usize) -> Result<()> {
        let mut deleted = 0;
        let mut cursor = self.get_cursor();
        while deleted < n {
            if cursor == Pos::new(0, 0) {
                break;
            }

            if cursor.x == 0 {
                let current_line = self
                    .data
                    .get(cursor.y)
                    .context("No line at cursor")?
                    .clone();
                let prev_line = self
                    .data
                    .get(cursor.y - 1)
                    .context("No line before cursor")?
                    .clone();
                self.data.remove(cursor.y);
                *(self
                    .data
                    .get_mut(cursor.y - 1)
                    .context("No line at cursor")?) = prev_line.clone() + &current_line;
                cursor = Pos::new(prev_line.len(), cursor.y - 1);
                deleted += 1;
                continue;
            }

            if cursor.x >= (n - deleted) {
                let current_line = self.data.get(cursor.y).context("No line at cursor")?;
                *(self.data.get_mut(cursor.y).context("No line at cursor")?) = current_line
                    .get(0..cursor.x + deleted - n)
                    .context("Couldn't extract start slice")?
                    .to_string()
                    + current_line
                        .get(cursor.x..current_line.len())
                        .context("Couldn't extract end slice")?;
                cursor.x -= n - deleted;
                deleted += n - deleted;
            } else {
                *(self.data.get_mut(cursor.y).context("No line at cursor")?) = String::new();
                deleted += cursor.x;
                cursor = Pos::new(0, cursor.y);
            }
        }

        self.cursor = cursor;
        Ok(())
    }

    pub fn add_line_at_cursor(&mut self) -> Result<()> {
        let cursor = self.get_cursor();
        if let Some(current_line) = self.data.get(cursor.y).map(|x| x.clone()) {
            *(self.data.get_mut(cursor.y).context("No line at cursor")?) = current_line
                .get(cursor.x..current_line.len())
                .context("Couldn't extract end slice")?
                .to_string();
            self.data.insert(
                cursor.y,
                current_line
                    .get(0..cursor.x)
                    .context("Couldn't extract start slice")?
                    .to_string(),
            );
        } else {
            self.data.push(String::new());
        }
        self.move_cursor(0, cursor.y + 1);
        Ok(())
    }

    pub fn load_from_file<P>(path: P) -> Result<Buffer>
    where
        P: AsRef<Path>,
    {
        let mut loaded_buffer = Buffer::new();
        loaded_buffer.data = fs::read_to_string(path)?
            .lines()
            .map(|l| l.to_string())
            .collect_vec();
        Ok(loaded_buffer)
    }

    pub fn save_to_file<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        fs::write(
            path,
            self.data
                .iter()
                .map(|line| (line.to_string() + &self.endl))
                .join("")
                .as_bytes(),
        )?;
        Ok(())
    }
}

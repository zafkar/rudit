use std::fs;

use anyhow::Result;
use itertools::Itertools;

use crate::pos::Pos;

#[derive(Debug, Clone, Default)]
pub struct Buffer {
    data: Vec<String>,
    cursor: Pos,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            data: vec!["First line".to_string()],
            cursor: Pos::default(),
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

    pub fn move_up(&mut self) -> Pos {
        self.move_cursor(
            self.cursor.x,
            self.cursor.y.checked_sub(1).unwrap_or(self.cursor.y),
        )
    }

    pub fn move_down(&mut self) -> Pos {
        self.move_cursor(self.cursor.x, self.cursor.y + 1)
    }

    pub fn move_left(&mut self) -> Pos {
        self.move_cursor(
            self.cursor.x.checked_sub(1).unwrap_or(self.cursor.x),
            self.cursor.y,
        )
    }

    pub fn move_right(&mut self) -> Pos {
        self.move_cursor(self.cursor.x + 1, self.cursor.y)
    }

    pub fn move_cursor(&mut self, x: usize, y: usize) -> Pos {
        let y = self.data.len().min(y);
        let x = self.data.get(y).map(|l| l.len()).unwrap_or_default().min(x);
        self.cursor = Pos::new(x, y);
        self.cursor.clone()
    }

    pub fn get_cursor(&self) -> Pos {
        self.cursor
    }

    pub fn load_from_file(path: &str) -> Result<Buffer> {
        let mut loaded_buffer = Buffer::new();
        loaded_buffer.data = fs::read_to_string(path)?
            .lines()
            .map(|l| l.to_string())
            .collect_vec();
        Ok(loaded_buffer)
    }
}

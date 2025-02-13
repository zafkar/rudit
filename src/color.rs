use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl From<Color> for crossterm::style::Color {
    fn from(value: Color) -> Self {
        crossterm::style::Color::Rgb {
            r: value.r,
            g: value.g,
            b: value.b,
        }
    }
}

impl FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("#") {
            let r = u8::from_str_radix(s.get(1..3).context("Invalid syntax for Color : red")?, 16)?;
            let g =
                u8::from_str_radix(s.get(3..5).context("Invalid syntax for Color : green")?, 16)?;
            let b =
                u8::from_str_radix(s.get(5..7).context("Invalid syntax for Color : blue")?, 16)?;
            Ok(Color { r, g, b })
        } else {
            Err(anyhow!("Invalid syntax for Color"))
        }
    }
}

impl From<&str> for Color {
    fn from(value: &str) -> Self {
        Color::from_str(value).unwrap_or_default()
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

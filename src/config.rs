use crate::{color::Color, editor::EditorAction};
use crossterm::style;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub color_edit_zone: ColorPair,
    pub color_status_bar: ColorPair,
    pub color_command_zone: ColorPair,
    pub edit_keybindings: HashMap<String, EditorAction>,
    pub command_keybindings: HashMap<String, EditorAction>,
    pub use_mouse: bool,
    pub use_paste: bool,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct ColorPair {
    #[serde(deserialize_with = "deserialize_color_from_str")]
    #[serde(serialize_with = "serialize_color_to_str")]
    bg: Color,
    #[serde(deserialize_with = "deserialize_color_from_str")]
    #[serde(serialize_with = "serialize_color_to_str")]
    fg: Color,
}

fn deserialize_color_from_str<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Color::from_str(&String::deserialize(deserializer)?).unwrap_or_default())
}

fn serialize_color_to_str<S>(x: &Color, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("{x}"))
}

impl From<ColorPair> for style::Colors {
    fn from(value: ColorPair) -> Self {
        style::Colors::new(value.fg.into(), value.bg.into())
    }
}

impl Default for Config {
    fn default() -> Self {
        let default_config = include_str!("rudit.toml");
        toml::from_str(&default_config).unwrap()
    }
}

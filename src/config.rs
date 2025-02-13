use crate::{color::Color, editor::EditorAction};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_color_from_str")]
    #[serde(serialize_with = "serialize_color_to_str")]
    pub bg_color_buffer: Color,
    #[serde(deserialize_with = "deserialize_color_from_str")]
    #[serde(serialize_with = "serialize_color_to_str")]
    pub fg_color_buffer: Color,
    #[serde(deserialize_with = "deserialize_color_from_str")]
    #[serde(serialize_with = "serialize_color_to_str")]
    pub bg_color_ui: Color,
    #[serde(deserialize_with = "deserialize_color_from_str")]
    #[serde(serialize_with = "serialize_color_to_str")]
    pub fg_color_ui: Color,
    pub keybindings: HashMap<String, EditorAction>,
    pub use_mouse: bool,
    pub use_paste: bool,
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

impl Default for Config {
    fn default() -> Self {
        let default_config = include_str!("rudit.toml");
        toml::from_str(&default_config).unwrap()
    }
}

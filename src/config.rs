use std::collections::HashMap;

use crossterm::style::Color;

use crate::editor::EditorAction;

#[derive(Debug, Clone)]
pub struct Config {
    pub bg_color_buffer: Color,
    pub fg_color_buffer: Color,
    pub bg_color_ui: Color,
    pub fg_color_ui: Color,
    pub keybindings: HashMap<String, EditorAction>,
}

impl Default for Config {
    fn default() -> Self {
        let keybindings = HashMap::from([
            ("Esc".to_string(), EditorAction::Quit),
            ("Up".to_string(), EditorAction::MoveUp),
            ("Down".to_string(), EditorAction::MoveDown),
            ("Left".to_string(), EditorAction::MoveLeft),
            ("Right".to_string(), EditorAction::MoveRight),
            ("Page Down".to_string(), EditorAction::PageDown),
            ("Page Up".to_string(), EditorAction::PageUp),
            ("Ctrls".to_string(), EditorAction::SaveDocument),
        ]);
        Self {
            bg_color_buffer: Color::Rgb {
                r: 0xb1,
                g: 0x62,
                b: 0x86,
            },
            fg_color_buffer: Color::Rgb {
                r: 0xfb,
                g: 0xf1,
                b: 0xc7,
            },
            bg_color_ui: Color::Rgb {
                r: 0x80,
                g: 0x47,
                b: 0x61,
            },
            fg_color_ui: Color::Rgb {
                r: 0xfb,
                g: 0xf1,
                b: 0xc7,
            },
            keybindings,
        }
    }
}

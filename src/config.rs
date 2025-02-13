use crossterm::style::Color;

#[derive(Debug, Clone)]
pub struct Config {
    pub bg_color_buffer: Color,
    pub fg_color_buffer: Color,
    pub bg_color_ui: Color,
    pub fg_color_ui: Color,
}

impl Default for Config {
    fn default() -> Self {
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
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Color {
    Rgb(u8, u8, u8),
    Str(String),
}

impl Color {
    pub fn to_rgb(&self) -> Option<(u8, u8, u8)> {
        match self {
            Color::Rgb(r, g, b) => Some((*r, *g, *b)),
            Color::Str(s) => {
                let s = s.trim_start_matches('#');
                if s.len() != 6 {
                    return None;
                }
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                Some((r, g, b))
            }
        }
    }

    pub fn precalculate_as_rgb(&mut self) {
        let (r, g, b) = self.to_rgb().unwrap();
        *self = Color::Rgb(r, g, b);
    }

    /// Linearly interpolates between two colors
    pub fn lerp(from: &Color, to: &Color, percent: f64) -> Color {
        let lerp = |from: u8, to: u8, perc: f64| {
            (from as f64 + perc * (to as f64 - from as f64)).round() as u8
        };

        let (r0, g0, b0) = from.to_rgb().unwrap();
        let (r1, g1, b1) = to.to_rgb().unwrap();
        Color::Rgb(
            lerp(r0, r1, percent),
            lerp(g0, g1, percent),
            lerp(b0, b1, percent),
        )
    }

    /// Linearly interpolates between a slice of colors
    pub fn lerp_many(colors: &[Color], percent: f64) -> Color {
        match colors.len() {
            0 => Color::Rgb(0, 0, 0),
            1 => colors[0].clone(),
            _ => {
                let index = (percent * (colors.len() - 1) as f64).floor() as usize;
                let p = percent * (colors.len() - 1) as f64 - index as f64;
                Color::lerp(&colors[index], &colors[index + 1], p)
            }
        }
    }
}

impl From<Color> for tui::style::Color {
    fn from(value: Color) -> Self {
        let (r, g, b) = value.to_rgb().unwrap();
        tui::style::Color::Rgb(r, g, b)
    }
}

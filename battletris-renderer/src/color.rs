#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0,   g: 0,   b: 0,   a: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const PANEL: Color = Color { r: 30,  g: 30,  b: 30,  a: 255 };
    pub const GRID:  Color = Color { r: 25,  g: 25,  b: 25,  a: 255 };

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn darken(self) -> Self {
        Self {
            r: self.r.saturating_sub(40),
            g: self.g.saturating_sub(40),
            b: self.b.saturating_sub(40),
            a: self.a,
        }
    }

    pub fn quarter(self) -> Self {
        Self { r: self.r / 4, g: self.g / 4, b: self.b / 4, a: self.a }
    }

    pub fn to_css(self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("rgba({},{},{},{:.3})", self.r, self.g, self.b, self.a as f64 / 255.0)
        }
    }
}

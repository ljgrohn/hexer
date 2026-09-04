//! Color math and formatting.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub [u8; 3]);

impl Rgb {
    pub fn hex(self) -> String {
        let [r, g, b] = self.0;
        format!("#{r:02X}{g:02X}{b:02X}")
    }

    pub fn rgb(self) -> String {
        let [r, g, b] = self.0;
        format!("rgb({r}, {g}, {b})")
    }

    pub fn hsl(self) -> String {
        let (h, s, l) = self.to_hsl();
        format!("hsl({h:.0}, {:.0}%, {:.0}%)", s * 100.0, l * 100.0)
    }

    /// (hue in degrees, saturation 0..1, lightness 0..1)
    pub fn to_hsl(self) -> (f32, f32, f32) {
        let [r, g, b] = self.0.map(|c| c as f32 / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        let d = max - min;
        if d.abs() < f32::EPSILON {
            return (0.0, 0.0, l);
        }
        let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        let mut h = if max == r {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
        h *= 60.0;
        (h, s, l)
    }

    /// Perceived luminance, used to pick readable text on top of the swatch.
    pub fn is_light(self) -> bool {
        let [r, g, b] = self.0.map(|c| c as f32);
        (0.299 * r + 0.587 * g + 0.114 * b) > 150.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        let c = Rgb([0x5B, 0x9C, 0xFF]);
        assert_eq!(c.hex(), "#5B9CFF");
        assert_eq!(c.rgb(), "rgb(91, 156, 255)");
        assert_eq!(c.hsl(), "hsl(216, 100%, 68%)");
    }

    #[test]
    fn grey_has_no_hue() {
        assert_eq!(Rgb([128, 128, 128]).to_hsl().0, 0.0);
    }
}

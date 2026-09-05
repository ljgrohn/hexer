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
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };
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

    /// Build an RGB color from HSL components. Hue wraps, while saturation and
    /// lightness are clamped so callers can safely nudge a color at the edges.
    pub fn from_hsl(hue: f32, saturation: f32, lightness: f32) -> Self {
        let h = hue.rem_euclid(360.0) / 360.0;
        let s = saturation.clamp(0.0, 1.0);
        let l = lightness.clamp(0.0, 1.0);

        if s <= f32::EPSILON {
            let value = (l * 255.0).round() as u8;
            return Self([value, value, value]);
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;
        let channel = |offset: f32| {
            let t = (h + offset).rem_euclid(1.0);
            let value = if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 0.5 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            };
            (value * 255.0).round() as u8
        };

        Self([channel(1.0 / 3.0), channel(0.0), channel(-1.0 / 3.0)])
    }

    /// Rotate this color around the color wheel while retaining its character.
    /// Very dark, light, or desaturated picks get a small correction so their
    /// theme suggestions remain distinct and useful instead of collapsing into
    /// nearly identical black, white, or grey chips.
    pub fn harmonized(self, degrees: f32) -> Self {
        let (mut h, mut s, mut l) = self.to_hsl();
        if s < 0.08 {
            h = 220.0;
            s = 0.46;
        }
        l = l.clamp(0.28, 0.72);
        Self::from_hsl(h + degrees, s, l)
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

    #[test]
    fn hsl_round_trip() {
        for color in [
            Rgb([0, 0, 0]),
            Rgb([255, 255, 255]),
            Rgb([91, 156, 255]),
            Rgb([221, 74, 108]),
        ] {
            let (h, s, l) = color.to_hsl();
            assert_eq!(Rgb::from_hsl(h, s, l), color);
        }
    }

    #[test]
    fn harmony_wraps_hue() {
        let red = Rgb([255, 0, 0]);
        assert_eq!(red.harmonized(120.0), Rgb([0, 255, 0]));
        assert_eq!(red.harmonized(240.0), Rgb([0, 0, 255]));
    }

    #[test]
    fn grey_gets_distinct_suggestions() {
        let grey = Rgb([128, 128, 128]);
        assert_ne!(grey.harmonized(30.0), grey.harmonized(180.0));
    }
}

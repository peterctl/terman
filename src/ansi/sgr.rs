use super::color::{Color, RgbColor};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Attribute {
    Reset,
    Bold,
    Dim,
    Italic,
    Underline,
    BlinkSlow,
    BlinkFast,
    Reverse,
    Hidden,
    Strike,
    CancelBold,
    CancelBoldDim,
    CancelItalic,
    CancelUnderline,
    CancelBlink,
    CancelReverse,
    CancelHidden,
    CancelStrike,
    Foreground(Color),
    Background(Color),
}

pub fn parse_color<'a>(parameters: &mut impl Iterator<Item=&'a i64>) -> Option<Color> {
    match parameters.next() {
        Some(2) => {
            match (parameters.next(), parameters.next(), parameters.next()) {
                (Some(r), Some(g), Some(b)) => {
                    let range = 0..256;
                    if !range.contains(r) || !range.contains(g) || !range.contains(b) {
                        None
                    } else {
                        Some(Color::Rgb(RgbColor {
                            r: *r as u8,
                            g: *g as u8,
                            b: *b as u8,
                        }))
                    }
                }
                _ => None,
            }
        }
        Some(5) => {
            match parameters.next() {
                Some(idx) => {
                    if !(0..256).contains(idx) {
                        None
                    } else {
                        Some(Color::Indexed(*idx as u8))
                    }
                },
                None => None,
            }
        },
        _ => None,
    }
}

pub fn parse_attributes<'a>(parameters: &mut impl Iterator<Item=&'a i64>) -> Vec<Attribute> {
    let mut vec = Vec::new();
    while let Some(param) = parameters.next() {
        let attr = match param {
            0 => Some(Attribute::Reset),
            1 => Some(Attribute::Bold),
            2 => Some(Attribute::Dim),
            3 => Some(Attribute::Italic),
            4 => Some(Attribute::Underline),
            5 => Some(Attribute::BlinkSlow),
            6 => Some(Attribute::BlinkFast),
            7 => Some(Attribute::Reverse),
            8 => Some(Attribute::Hidden),
            9 => Some(Attribute::Strike),
            21 => Some(Attribute::CancelBold),
            22 => Some(Attribute::CancelBoldDim),
            23 => Some(Attribute::CancelItalic),
            24 => Some(Attribute::CancelUnderline),
            25 => Some(Attribute::CancelBlink),
            27 => Some(Attribute::CancelReverse),
            28 => Some(Attribute::CancelHidden),
            29 => Some(Attribute::CancelStrike),
            30 => Some(Attribute::Foreground(Color::Black)),
            31 => Some(Attribute::Foreground(Color::Red)),
            32 => Some(Attribute::Foreground(Color::Green)),
            33 => Some(Attribute::Foreground(Color::Yellow)),
            34 => Some(Attribute::Foreground(Color::Blue)),
            35 => Some(Attribute::Foreground(Color::Magenta)),
            36 => Some(Attribute::Foreground(Color::Cyan)),
            37 => Some(Attribute::Foreground(Color::White)),
            38 => {
                if let Some(color) = parse_color(parameters) {
                    Some(Attribute::Foreground(color))
                } else {
                    None
                }
            },
            39 => Some(Attribute::Foreground(Color::Foreground)),
            40 => Some(Attribute::Background(Color::Black)),
            41 => Some(Attribute::Background(Color::Red)),
            42 => Some(Attribute::Background(Color::Green)),
            43 => Some(Attribute::Background(Color::Yellow)),
            44 => Some(Attribute::Background(Color::Blue)),
            45 => Some(Attribute::Background(Color::Magenta)),
            46 => Some(Attribute::Background(Color::Cyan)),
            47 => Some(Attribute::Background(Color::White)),
            48 => {
                if let Some(color) = parse_color(parameters) {
                    Some(Attribute::Background(color))
                } else {
                    None
                }
            },
            49 => Some(Attribute::Background(Color::Background)),
            90 => Some(Attribute::Foreground(Color::BrightBlack)),
            91 => Some(Attribute::Foreground(Color::BrightRed)),
            92 => Some(Attribute::Foreground(Color::BrightGreen)),
            93 => Some(Attribute::Foreground(Color::BrightYellow)),
            94 => Some(Attribute::Foreground(Color::BrightBlue)),
            95 => Some(Attribute::Foreground(Color::BrightMagenta)),
            96 => Some(Attribute::Foreground(Color::BrightCyan)),
            97 => Some(Attribute::Foreground(Color::BrightWhite)),
            100 => Some(Attribute::Background(Color::BrightBlack)),
            101 => Some(Attribute::Background(Color::BrightRed)),
            102 => Some(Attribute::Background(Color::BrightGreen)),
            103 => Some(Attribute::Background(Color::BrightYellow)),
            104 => Some(Attribute::Background(Color::BrightBlue)),
            105 => Some(Attribute::Background(Color::BrightMagenta)),
            106 => Some(Attribute::Background(Color::BrightCyan)),
            107 => Some(Attribute::Background(Color::BrightWhite)),
            _ => None,
        };
        if let Some(attr) = attr {
            vec.push(attr);
        }
    }
    return vec;
}

#[cfg(test)]
mod tests {
    use super::super::color::{Color, RgbColor};
    use super::{parse_attributes, Attribute};

    #[test]
    fn test_parse_attributes() {
        let params = &[1, 3, 4, 7, 30, 38, 2, 100, 100, 100, 48, 5, 64];

        assert_eq!(parse_attributes(&mut params.iter()), vec![
            Attribute::Bold,
            Attribute::Italic,
            Attribute::Underline,
            Attribute::Reverse,
            Attribute::Foreground(Color::Black),
            Attribute::Foreground(Color::Rgb(RgbColor { r: 100, g: 100, b: 100, })),
            Attribute::Background(Color::Indexed(64)),
        ])
    }
}

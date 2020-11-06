use crate::ansi::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub ch: Option<char>,
    pub attributes: Attributes,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: None,
            attributes: Attributes::default(),
        }
    }
}

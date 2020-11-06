use std::cmp::{PartialEq, PartialOrd};

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point {
    // NOTE `y` has to go before `x` so that it has higher
    // priority when deriving PartialOrd

    /// The point's vertical coordinate
    pub y: usize,

    /// The point's horizontal coordinate
    pub x: usize,
}

impl Point {
    #[inline]
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl From<(usize, usize)> for Point {
    #[inline]
    fn from(pair: (usize, usize)) -> Self {
        Self { x: pair.0, y: pair.1, }
    }
}

#[allow(non_snake_case)]
#[inline]
pub fn P(x: usize, y: usize) -> Point {
    Point::new(x, y)
}

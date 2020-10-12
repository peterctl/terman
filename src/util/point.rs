use std::{
    cmp::{
        PartialEq,
        PartialOrd,
    },
    ops::{
        Add,
        Div,
        Mul,
        Sub,
    }
};

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

impl Add for Point {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            y: self.y + other.y,
            x: self.x + other.x,
        }
    }
}

impl Sub for Point {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            y: self.y - other.y,
            x: self.x - other.x,
        }
    }
}

impl Mul<usize> for Point {
    type Output = Self;
    fn mul(self, other: usize) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Div<usize> for Point {
    type Output = Self;
    fn div(self, other: usize) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl From<(usize, usize)> for Point {
    #[inline]
    fn from(pair: (usize, usize)) -> Self {
        Self { x: pair.0, y: pair.1, }
    }
}

impl Into<(usize, usize)> for Point {
    #[inline]
    fn into(self) -> (usize, usize) {
        (self.x, self.y)
    }
}

#[allow(non_snake_case)]
#[inline]
pub fn P(x: usize, y: usize) -> Point {
    Point::new(x, y)
}

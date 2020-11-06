use std::ops::{Deref, DerefMut};
use super::cell::Cell;

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    vec: Vec<Cell>,
}

impl Line {
    pub fn new(size: usize) -> Self {
        Self {
            vec: vec![Cell::default(); size],
        }
    }
}

impl Deref for Line {
    type Target = Vec<Cell>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl DerefMut for Line {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

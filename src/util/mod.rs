pub mod point;

use std::iter::Iterator;
pub use point::{Point, P};

pub trait BidirectionalIterator: Iterator {
    fn prev(&mut self) -> Option<Self::Item>;
}

pub trait Each {
    type Item;

    fn each<F: Fn(Self::Item)>(&mut self, cb: F);
}

impl<I, T> Each for I
where
    I: Iterator<Item=T>,
{
    type Item = T;

    fn each<F: Fn(Self::Item)>(&mut self, cb: F) {
        for item in self {
            cb(item);
        }
    }
}

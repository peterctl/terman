mod line;
mod cell;
mod iter;

use std::ops::RangeBounds;
use crate::util::Point;
use self::{
    line::Line,
    cell::Cell,
    iter::{
        GridIterator,
        GridMutIterator,
        LinewisePointGenerator,
        BlockwisePointGenerator,
    },
};

#[derive(Debug, Clone)]
pub struct Grid {
    data: Vec<Line>,
    tabstops: Vec<bool>,
    size: Point,
}

impl Grid {
    pub fn new(size: Point) -> Self {
        let data = vec![Line::new(size.x); size.y];
        let tabstops = vec![false; size.x];
        Self {
            data,
            tabstops,
            size,
        }
    }

    fn add_line(&mut self) {
        self.data.push(Line::new(self.size.x));
    }

    fn point_to_index(&self, p: Point) -> usize {
        p.y * self.size.x + p.x
    }

    pub fn cell(&self, point: Point) -> Option<&Cell> {
        self.data.get(point.y).and_then(|gl| gl.get(point.x))
    }

    pub fn cell_mut(&mut self, point: Point) -> Option<&mut Cell> {
        self.data.get_mut(point.y).and_then(|gl| gl.get_mut(point.x))
    }

    pub fn lines<R: RangeBounds<usize>>(&self, range: R) -> GridIterator<LinewisePointGenerator> {
        GridIterator::lines(self, range)
    }

    pub fn lines_mut<R: RangeBounds<usize>>(&mut self, range: R) -> GridMutIterator<LinewisePointGenerator> {
        GridMutIterator::lines(self, range)
    }

    pub fn selection<R: RangeBounds<Point>>(&self, range: R) -> GridIterator<LinewisePointGenerator> {
        GridIterator::selection(self, range)
    }

    pub fn selection_mut<R: RangeBounds<Point>>(&mut self, range: R) -> GridMutIterator<LinewisePointGenerator> {
        GridMutIterator::selection(self, range)
    }

    pub fn block<R: RangeBounds<Point>>(&self, range: R) -> GridIterator<BlockwisePointGenerator> {
        GridIterator::block(self, range)
    }

    pub fn block_mut<R: RangeBounds<Point>>(&mut self, range: R) -> GridMutIterator<BlockwisePointGenerator> {
        GridMutIterator::block(self, range)
    }
}

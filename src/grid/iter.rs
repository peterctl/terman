use std::ops::{
    Bound,
    Range,
    RangeBounds,
};
use crate::util::Point;
use super::{
    Grid,
    Cell,
};

pub struct LinewisePointGenerator {
    range: Range<Point>,
    size: Point,
    cur: Point,
}

impl LinewisePointGenerator {
    pub fn new<R: RangeBounds<Point>>(range: R, size: Point) -> Self {
        let range = Self::normalize_range(range, size);
        let cur = range.start;
        Self {
            range,
            size,
            cur,
        }
    }

    pub fn normalize_range<R: RangeBounds<Point>>(range: R, size: Point) -> Range<Point> {
        // TODO make sure ranges are within the grid bounds
        let start = match range.start_bound() {
            Bound::Included(p) => *p,
            _ => Point::default(),
        };
        let end = match range.end_bound() {
            Bound::Unbounded => Point::new(0, size.y),
            Bound::Excluded(p) => *p,
            Bound::Included(p) => {
                let mut np = *p;
                np.x += 1;
                if np.x >= size.x {
                    np.x = 0;
                    np.y += 1;
                }
                np
            },
        };
        start..end
    }
}

impl Iterator for LinewisePointGenerator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.range.end {
            return None;
        }
        let current = self.cur;
        self.cur.x += 1;
        if self.cur.x >= self.size.x {
            self.cur.x = 0;
            self.cur.y += 1;
        }
        Some(current)
    }
}

pub struct BlockwisePointGenerator {
    range: Range<Point>,
    cur: Point,
}

impl BlockwisePointGenerator {
    pub fn new<R: RangeBounds<Point>>(range: R, size: Point) -> Self {
        // TODO make sure ranges are within the grid bounds
        let range = Self::normalize_range(range, size);
        let cur = range.start;
        Self {
            range,
            cur,
        }
    }

    pub fn normalize_range<R: RangeBounds<Point>>(range: R, size: Point) -> Range<Point> {
        let start = match range.start_bound() {
            Bound::Included(p) => *p,
            _ => Point::default(),
        };
        let end = match range.end_bound() {
            Bound::Unbounded => size,
            Bound::Excluded(p) => *p,
            Bound::Included(p) => Point::new(p.x + 1, p.y + 1),
        };
        start..end
    }
}

impl Iterator for BlockwisePointGenerator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur.x >= self.range.end.x || self.cur.y >= self.range.end.y {
            return None;
        }
        let current = self.cur;
        self.cur.x += 1;
        if self.cur.x >= self.range.end.x {
            self.cur.x = self.range.start.x;
            self.cur.y += 1;
        }
        Some(current)
    }
}

pub struct GridIterator<'a, I: Iterator<Item=Point>>
{
    grid: &'a Grid,
    point_generator: I,
}

impl<'a> GridIterator<'a, LinewisePointGenerator>
{
    pub fn selection<R>(grid: &'a Grid, range: R) -> Self
    where
        R: RangeBounds<Point>,
    {
        let point_generator = LinewisePointGenerator::new(range, grid.size);
        Self {
            grid,
            point_generator,
        }
    }

    pub fn lines<R: RangeBounds<usize>>(grid: &'a Grid, range: R) -> Self {
        // TODO make sure ranges are within the grid bounds
        let start: usize = match range.start_bound() {
            Bound::Included(n) => *n,
            _ => 0,
        };
        let end = match range.end_bound() {
            Bound::Unbounded => grid.size.y,
            Bound::Excluded(n) => *n,
            Bound::Included(n) => *n + 1,
        };
        let point_generator = LinewisePointGenerator::new(
            Point::new(0, start)..Point::new(0, end),
            grid.size,
        );
        Self {
            grid,
            point_generator,
        }
    }
}

impl<'a> GridIterator<'a, BlockwisePointGenerator> {
    pub fn block<R: RangeBounds<Point>>(grid: &'a Grid, range: R) -> Self {
        let point_generator = BlockwisePointGenerator::new(range, grid.size);
        Self {
            grid,
            point_generator,
        }
    }
}

impl<'a, I> Iterator for GridIterator<'a, I>
where
    I: Iterator<Item=Point>,
{
    type Item = (Point, &'a Cell);

    fn next(&mut self) -> Option<Self::Item> {
        self.point_generator.next().and_then(|p| {
            self.grid.cell(p).map(|c| (p, c))
        })
    }
}

pub struct GridMutIterator<'a, I: Iterator<Item=Point>>
{
    grid: &'a mut Grid,
    point_generator: I,
}

impl<'a> GridMutIterator<'a, LinewisePointGenerator>
{
    pub fn selection<R>(grid: &'a mut Grid, range: R) -> Self
    where
        R: RangeBounds<Point>,
    {
        let point_generator = LinewisePointGenerator::new(range, grid.size);
        Self {
            grid,
            point_generator,
        }
    }

    pub fn lines<R: RangeBounds<usize>>(grid: &'a mut Grid, range: R) -> Self {
        let start: usize = match range.start_bound() {
            Bound::Included(n) => *n,
            _ => 0,
        };
        let end = match range.end_bound() {
            Bound::Unbounded => grid.size.y,
            Bound::Excluded(n) => *n,
            Bound::Included(n) => *n + 1,
        };
        let point_generator = LinewisePointGenerator::new(
            Point::new(0, start)..Point::new(0, end),
            grid.size,
        );
        Self {
            grid,
            point_generator,
        }
    }
}

impl<'a> GridMutIterator<'a, BlockwisePointGenerator> {
    pub fn block<R: RangeBounds<Point>>(grid: &'a mut Grid, range: R) -> Self {
        let point_generator = BlockwisePointGenerator::new(range, grid.size);
        Self {
            grid,
            point_generator,
        }
    }
}

impl<'a, I> Iterator for GridMutIterator<'a, I>
where
    I: Iterator<Item=Point>,
{
    type Item = (Point, &'a mut Cell);

    fn next(&mut self) -> Option<Self::Item> {
        self.point_generator.next().and_then(|p| {
            self.grid.cell_mut(p)
                // Cast cell as a pointer and then back to a mutable reference
                // to stop the compiler from thinking it's an invalid reference
                .map(|c| unsafe { &mut *(c as *mut _) })
                .map(|c| (p, c))
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::util::point::P;
    use super::{
        LinewisePointGenerator,
        BlockwisePointGenerator,
    };

    #[test]
    fn test_selectionwise_point_generator_range() {
        let gen = LinewisePointGenerator::new(P(3, 2)..P(1, 4), P(5, 5));
        assert_eq!(gen.range.start, P(3, 2));
        assert_eq!(gen.range.end, P(1, 4));
        assert_eq!(
            gen.collect::<Vec<_>>(),
            vec![
                                           P(3, 2), P(4, 2),
                P(0, 3), P(1, 3), P(2, 3), P(3, 3), P(4, 3),
                P(0, 4),
            ],
        );
    }

    #[test]
    fn test_selectionwise_point_generator_range_inclusive() {
        let gen = LinewisePointGenerator::new(P(3, 2)..=P(1, 4), P(5, 5));
        assert_eq!(gen.range.start, P(3, 2));
        assert_eq!(gen.range.end, P(2, 4));
        assert_eq!(
            gen.collect::<Vec<_>>(),
            vec![
                                           P(3, 2), P(4, 2),
                P(0, 3), P(1, 3), P(2, 3), P(3, 3), P(4, 3),
                P(0, 4), P(1, 4),
            ],
        );
    }

    #[test]
    fn test_selectionwise_point_generator_range_to() {
        let gen = LinewisePointGenerator::new(..P(4, 2), P(5, 5));
        assert_eq!(gen.range.start, P(0, 0));
        assert_eq!(gen.range.end, P(4, 2));
        assert_eq!(
            gen.collect::<Vec<_>>(),
            vec![
                P(0, 0), P(1, 0), P(2, 0), P(3, 0), P(4, 0),
                P(0, 1), P(1, 1), P(2, 1), P(3, 1), P(4, 1),
                P(0, 2), P(1, 2), P(2, 2), P(3, 2),
            ],
        );
    }

    #[test]
    fn test_selectionwise_point_generator_range_to_inclusive() {
        let gen = LinewisePointGenerator::new(..=P(4, 2), P(5, 5));
        assert_eq!(gen.range.start, P(0, 0));
        assert_eq!(gen.range.end, P(0, 3));
        assert_eq!(
            gen.collect::<Vec<_>>(),
            vec![
                P(0, 0), P(1, 0), P(2, 0), P(3, 0), P(4, 0),
                P(0, 1), P(1, 1), P(2, 1), P(3, 1), P(4, 1),
                P(0, 2), P(1, 2), P(2, 2), P(3, 2), P(4, 2),
            ],
        );
    }

    #[test]
    fn test_selectionwise_point_generator_range_from() {
        let gen = LinewisePointGenerator::new(P(2, 2).., P(5, 5));
        assert_eq!(gen.range.start, P(2, 2));
        assert_eq!(gen.range.end, P(0, 5));
        assert_eq!(
            gen.collect::<Vec<_>>(),
            vec![
                                  P(2, 2), P(3, 2), P(4, 2),
                P(0, 3), P(1, 3), P(2, 3), P(3, 3), P(4, 3),
                P(0, 4), P(1, 4), P(2, 4), P(3, 4), P(4, 4),
            ],
        );
    }

    #[test]
    fn test_selectionwise_point_generator_range_full() {
        let gen = LinewisePointGenerator::new(.., P(5, 5));
        assert_eq!(gen.range.start, P(0, 0));
        assert_eq!(gen.range.end, P(0, 5));
        assert_eq!(
            gen.collect::<Vec<_>>(),
            vec![
                P(0, 0), P(1, 0), P(2, 0), P(3, 0), P(4, 0),
                P(0, 1), P(1, 1), P(2, 1), P(3, 1), P(4, 1),
                P(0, 2), P(1, 2), P(2, 2), P(3, 2), P(4, 2),
                P(0, 3), P(1, 3), P(2, 3), P(3, 3), P(4, 3),
                P(0, 4), P(1, 4), P(2, 4), P(3, 4), P(4, 4),
            ],
        );
    }

    #[test]
    fn test_blockwise_point_generator_range() {
        let gen = BlockwisePointGenerator::new(P(1, 1)..P(4, 3), P(5, 5));
        assert_eq!(gen.range.start, P(1, 1));
        assert_eq!(gen.range.end, P(4, 3));
        assert_eq!(
            gen.collect::<Vec<_>>(),
            vec![
                         P(1, 1), P(2, 1), P(3, 1),
                         P(1, 2), P(2, 2), P(3, 2),
            ],
        );
    }

    #[test]
    fn test_blockwise_point_generator_range_inclusive() {
        let gen = BlockwisePointGenerator::new(P(1, 1)..=P(4, 3), P(5, 5));
        assert_eq!(gen.range.start, P(1, 1));
        assert_eq!(gen.range.end, P(5, 4));
        assert_eq!(
            gen.collect::<Vec<_>>(),
            vec![
                         P(1, 1), P(2, 1), P(3, 1), P(4, 1),
                         P(1, 2), P(2, 2), P(3, 2), P(4, 2),
                         P(1, 3), P(2, 3), P(3, 3), P(4, 3),
            ],
        );
    }

    #[test]
    fn test_blockwise_point_generator_range_to() {
        let gen = BlockwisePointGenerator::new(..P(2, 3), P(5, 5));
        assert_eq!(gen.range.start, P(0, 0));
        assert_eq!(gen.range.end, P(2, 3));
        assert_eq!(
            gen.collect::<Vec<_>>(),
            vec![
                P(0, 0), P(1, 0),
                P(0, 1), P(1, 1),
                P(0, 2), P(1, 2),
            ],
        );
    }

    #[test]
    fn test_blockwise_point_generator_range_to_inclusive() {
        let gen = BlockwisePointGenerator::new(..=P(2, 3), P(5, 5));
        assert_eq!(gen.range.start, P(0, 0));
        assert_eq!(gen.range.end, P(3, 4));
        assert_eq!(
            gen.collect::<Vec<_>>(),
            vec![
                P(0, 0), P(1, 0), P(2, 0),
                P(0, 1), P(1, 1), P(2, 1),
                P(0, 2), P(1, 2), P(2, 2),
                P(0, 3), P(1, 3), P(2, 3),
            ],
        );
    }

    #[test]
    fn test_blockwise_point_generator_range_from() {
        let gen = BlockwisePointGenerator::new(P(2, 3).., P(5, 5));
        assert_eq!(gen.range.start, P(2, 3));
        assert_eq!(gen.range.end, P(5, 5));
        assert_eq!(
            gen.collect::<Vec<_>>(),
            vec![
                                  P(2, 3), P(3, 3), P(4, 3),
                                  P(2, 4), P(3, 4), P(4, 4),
            ],
        );
    }

    #[test]
    fn test_blockwise_point_generator_range_full() {
        let gen = BlockwisePointGenerator::new(.., P(5, 5));
        assert_eq!(gen.range.start, P(0, 0));
        assert_eq!(gen.range.end, P(5, 5));
        assert_eq!(
            gen.collect::<Vec<_>>(),
            vec![
                P(0, 0), P(1, 0), P(2, 0), P(3, 0), P(4, 0),
                P(0, 1), P(1, 1), P(2, 1), P(3, 1), P(4, 1),
                P(0, 2), P(1, 2), P(2, 2), P(3, 2), P(4, 2),
                P(0, 3), P(1, 3), P(2, 3), P(3, 3), P(4, 3),
                P(0, 4), P(1, 4), P(2, 4), P(3, 4), P(4, 4),
            ],
        );
    }
}

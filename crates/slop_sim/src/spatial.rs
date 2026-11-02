//! Uniform grid over the arena for neighbour queries, rebuilt every tick by
//! counting sort (adapted from Lineage).

use crate::vec::V2;
use crate::world::ARENA;

/// Grid cell size in world units.
pub const CELL: f32 = 64.0;
const SIDE: usize = (ARENA / CELL) as usize + 1;

pub struct Grid {
    /// `start[c]..start[c + 1]` indexes `items` for cell `c`.
    start: Vec<u32>,
    items: Vec<u32>,
    cell_of: Vec<u32>,
}

fn cell(v: f32) -> usize {
    ((v / CELL).max(0.0) as usize).min(SIDE - 1)
}

impl Grid {
    pub fn new() -> Grid {
        Grid { start: vec![0; SIDE * SIDE + 1], items: Vec::new(), cell_of: Vec::new() }
    }

    pub fn rebuild(&mut self, positions: impl Iterator<Item = V2>) {
        self.cell_of.clear();
        self.start.fill(0);
        for p in positions {
            let c = (cell(p.y) * SIDE + cell(p.x)) as u32;
            self.cell_of.push(c);
            self.start[c as usize + 1] += 1;
        }
        for c in 1..self.start.len() {
            self.start[c] += self.start[c - 1];
        }
        self.items.clear();
        self.items.resize(self.cell_of.len(), 0);
        let mut fill = self.start.clone();
        for (i, &c) in self.cell_of.iter().enumerate() {
            self.items[fill[c as usize] as usize] = i as u32;
            fill[c as usize] += 1;
        }
    }

    /// Calls `f` with every item whose cell overlaps the square around the
    /// query circle. Callers still check the exact distance.
    pub fn for_each_near(&self, at: V2, radius: f32, mut f: impl FnMut(usize)) {
        let (x0, x1) = (cell(at.x - radius), cell(at.x + radius));
        let (y0, y1) = (cell(at.y - radius), cell(at.y + radius));
        for gy in y0..=y1 {
            for gx in x0..=x1 {
                let c = gy * SIDE + gx;
                for &i in &self.items[self.start[c] as usize..self.start[c + 1] as usize] {
                    f(i as usize);
                }
            }
        }
    }
}

impl Default for Grid {
    fn default() -> Grid {
        Grid::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_neighbours_and_skips_far_items() {
        let mut grid = Grid::new();
        let pts = [V2::new(100.0, 100.0), V2::new(110.0, 105.0), V2::new(900.0, 900.0)];
        grid.rebuild(pts.iter().copied());
        let mut found = Vec::new();
        grid.for_each_near(V2::new(100.0, 100.0), CELL, |i| found.push(i));
        found.sort();
        assert_eq!(found, vec![0, 1]);
    }

    #[test]
    fn positions_outside_the_arena_are_clamped() {
        let mut grid = Grid::new();
        grid.rebuild([V2::new(-50.0, ARENA + 80.0)].into_iter());
        let mut found = 0;
        grid.for_each_near(V2::new(0.0, ARENA), 10.0, |_| found += 1);
        assert_eq!(found, 1);
    }
}

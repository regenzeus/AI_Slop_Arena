//! Uniform grid for neighbour queries, rebuilt every tick by counting sort
//! (adapted from Lineage). The map is endless, so the grid is a window
//! centred where the action is; anything outside it lands in the edge cells.

use crate::vec::V2;

/// Grid cell size in world units.
pub const CELL: f32 = 64.0;
/// Side of the window the grid covers, in world units.
pub const SPAN: f32 = 4800.0;
const SIDE: usize = (SPAN / CELL) as usize + 1;

pub struct Grid {
    /// World position of the window's lower corner.
    origin: V2,
    /// `start[c]..start[c + 1]` indexes `items` for cell `c`.
    start: Vec<u32>,
    items: Vec<u32>,
    cell_of: Vec<u32>,
}

impl Grid {
    pub fn new() -> Grid {
        Grid { origin: V2::new(-SPAN / 2.0, -SPAN / 2.0), start: vec![0; SIDE * SIDE + 1], items: Vec::new(), cell_of: Vec::new() }
    }

    fn cell(&self, v: f32, o: f32) -> usize {
        (((v - o) / CELL).max(0.0) as usize).min(SIDE - 1)
    }

    /// Rebuilds the grid with its window centred on `centre`.
    pub fn rebuild(&mut self, centre: V2, positions: impl Iterator<Item = V2>) {
        self.origin = centre - V2::new(SPAN / 2.0, SPAN / 2.0);
        self.cell_of.clear();
        self.start.fill(0);
        for p in positions {
            let c = (self.cell(p.y, self.origin.y) * SIDE + self.cell(p.x, self.origin.x)) as u32;
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
        let (x0, x1) = (self.cell(at.x - radius, self.origin.x), self.cell(at.x + radius, self.origin.x));
        let (y0, y1) = (self.cell(at.y - radius, self.origin.y), self.cell(at.y + radius, self.origin.y));
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
        grid.rebuild(V2::ZERO, pts.iter().copied());
        let mut found = Vec::new();
        grid.for_each_near(V2::new(100.0, 100.0), CELL, |i| found.push(i));
        found.sort();
        assert_eq!(found, vec![0, 1]);
    }

    // REQ: MAP-01
    #[test]
    fn the_window_follows_its_centre_far_from_the_origin() {
        let mut grid = Grid::new();
        let far = V2::new(250_000.0, -180_000.0);
        grid.rebuild(far, [far + V2::new(10.0, 0.0), far + V2::new(3000.0, 0.0)].into_iter());
        let mut found = Vec::new();
        grid.for_each_near(far, 50.0, |i| found.push(i));
        assert_eq!(found, vec![0], "only the near item, far from the origin");
    }
}

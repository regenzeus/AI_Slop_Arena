//! The endless map (MAP-01 to MAP-04). It is cut into square chunks, and
//! each chunk is generated from the run's seed and its coordinates alone, so
//! the same seed always gives the same map, in whatever order you explore
//! it. Chunks hold walls (server racks, billboards, desks) and props: things
//! to break, to stand in, and to spend Clout on. Maintenance fences close
//! off parts of the map for a while.

use std::collections::HashMap;

use crate::rng::Rng;
use crate::vec::V2;

/// Side of a chunk in world units.
pub const CHUNK: f32 = 1400.0;
/// Chunks this far from yours (in chunks) always exist.
const KEEP: i32 = 2;
/// No walls or props this close to where a run starts.
const START_CLEAR: f32 = 420.0;
/// Ids of map things live above this, apart from the world's ids.
const ID_BASE: u32 = 1 << 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WallKind {
    ServerRack,
    Billboard,
    Desk,
    /// "Under maintenance": a temporary fence.
    Fence,
}

/// An axis-aligned block that you, the slop and projectiles cannot pass.
#[derive(Clone, Copy, Debug)]
pub struct Wall {
    pub id: u32,
    pub kind: WallKind,
    pub min: V2,
    pub max: V2,
    /// Fences: ticks before it closes (0 when solid), and ticks it stays closed.
    pub warning: u16,
    pub ticks: u16,
}

impl Wall {
    pub fn solid(&self) -> bool {
        self.warning == 0
    }

    pub fn centre(&self) -> V2 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> V2 {
        self.max - self.min
    }

    pub fn contains(&self, p: V2) -> bool {
        p.x > self.min.x && p.x < self.max.x && p.y > self.min.y && p.y < self.max.y
    }

    /// Where a circle at `p` with radius `r` has to move to stop overlapping
    /// the wall, if it overlaps.
    pub fn push_out(&self, p: V2, r: f32) -> Option<V2> {
        let c = V2::new(p.x.clamp(self.min.x, self.max.x), p.y.clamp(self.min.y, self.max.y));
        let d = p - c;
        let len_sq = d.length_sq();
        if len_sq >= r * r {
            return None;
        }
        if len_sq > 1e-6 {
            let len = len_sq.sqrt();
            return Some(c + d * (r / len));
        }
        // The centre is inside: out through the nearest side.
        let gaps = [(p.x - self.min.x, V2::new(self.min.x - r, p.y)), (self.max.x - p.x, V2::new(self.max.x + r, p.y)), (p.y - self.min.y, V2::new(p.x, self.min.y - r)), (self.max.y - p.y, V2::new(p.x, self.max.y + r))];
        gaps.iter().min_by(|a, b| a.0.total_cmp(&b.0)).map(|g| g.1)
    }

    /// Whether the segment from `a` to `b` crosses the wall.
    pub fn blocks(&self, a: V2, b: V2) -> bool {
        // Slab test.
        let d = b - a;
        let (mut t0, mut t1) = (0.0f32, 1.0f32);
        for (o, v, lo, hi) in [(a.x, d.x, self.min.x, self.max.x), (a.y, d.y, self.min.y, self.max.y)] {
            if v.abs() < 1e-6 {
                if o <= lo || o >= hi {
                    return false;
                }
            } else {
                let (mut ta, mut tb) = ((lo - o) / v, (hi - o) / v);
                if ta > tb {
                    std::mem::swap(&mut ta, &mut tb);
                }
                t0 = t0.max(ta);
                t1 = t1.min(tb);
                if t0 > t1 {
                    return false;
                }
            }
        }
        true
    }
}

/// Things on the map you can use (MAP-03).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PropKind {
    /// Breakable: walk into it.
    AdBillboard,
    PopupAd,
    CryptoMiner,
    /// Stand inside to heal, until its data runs out.
    Hotspot,
    /// Stand in the ring to use it.
    Vending,
    Gacha,
    Shrine,
    DataCenter,
    CommentSection,
    /// Somebody's phone: a chest.
    LostPhone,
}

impl PropKind {
    pub const ALL: [PropKind; 10] = [
        PropKind::AdBillboard,
        PropKind::PopupAd,
        PropKind::CryptoMiner,
        PropKind::Hotspot,
        PropKind::Vending,
        PropKind::Gacha,
        PropKind::Shrine,
        PropKind::DataCenter,
        PropKind::CommentSection,
        PropKind::LostPhone,
    ];

    pub fn name(self) -> &'static str {
        [
            "Ad Billboard",
            "Pop-up Ad",
            "Crypto Miner",
            "Free Wi-Fi",
            "Vending Machine",
            "Gacha Machine",
            "Shrine of the Algorithm",
            "Data Center",
            "Comment Section",
            "Lost Phone",
        ][self as usize]
    }

    pub fn blurb(self) -> &'static str {
        [
            "Walk into it. Ads always hide something.",
            "Close it. There is a prize behind the X.",
            "Smash it before it mines your Mental Health.",
            "Stand inside to heal while the data lasts.",
            "Stand in the ring: pick an item for Clout.",
            "Stand in the ring: a random upgrade for Clout.",
            "Stand in the ring: pick one of three blessings.",
            "Stand inside while it uploads. The slop wants it too.",
            "Stand in the ring to open it. Elites come out. Survive them.",
            "Walk over it: a chest.",
        ][self as usize]
    }

    /// Broken by walking into it.
    pub fn breakable(self) -> bool {
        matches!(self, PropKind::AdBillboard | PropKind::PopupAd | PropKind::CryptoMiner | PropKind::LostPhone)
    }

    /// Radius of its body, or of the ring you stand in.
    pub fn radius(self) -> f32 {
        match self {
            PropKind::AdBillboard => 34.0,
            PropKind::PopupAd => 24.0,
            PropKind::CryptoMiner => 26.0,
            PropKind::Hotspot => 150.0,
            PropKind::DataCenter => 190.0,
            PropKind::LostPhone => 22.0,
            _ => 80.0,
        }
    }

    /// Ticks you stand in the ring before it is used.
    pub fn charge(self) -> u16 {
        match self {
            PropKind::Vending | PropKind::Gacha | PropKind::Shrine => 50,
            PropKind::CommentSection => 70,
            PropKind::DataCenter => 30,
            _ => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropState {
    Idle,
    /// In use: a data center uploading, a comment section with elites out.
    Active,
    Done,
}

#[derive(Clone, Debug)]
pub struct Prop {
    pub id: u32,
    pub kind: PropKind,
    pub pos: V2,
    /// Ticks you have stood in its ring.
    pub charge: u16,
    pub state: PropState,
    /// What is left: a hotspot's data, a data center's upload, in ticks.
    pub left: u32,
    /// What it costs to use, in Clout; set by the rules.
    pub price: u32,
}

/// Chunk coordinates of a point.
pub fn chunk_of(p: V2) -> (i32, i32) {
    ((p.x / CHUNK).floor() as i32, (p.y / CHUNK).floor() as i32)
}

/// A seed for one chunk, the same whatever order chunks are made in.
pub fn chunk_seed(seed: u64, c: (i32, i32)) -> u64 {
    let mut h = seed ^ 0x9e37_79b9_7f4a_7c15;
    for v in [c.0 as i64 as u64, c.1 as i64 as u64] {
        h ^= v.wrapping_add(0x9e37_79b9_7f4a_7c15).wrapping_add(h << 6).wrapping_add(h >> 2);
        h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    }
    h ^ (h >> 33)
}

#[derive(Default)]
pub struct Map {
    seed: u64,
    made: HashMap<(i32, i32), Vec<usize>>,
    /// Every wall made so far; indexes are stable.
    pub walls: Vec<Wall>,
    /// Maintenance fences, warning or closed.
    pub fences: Vec<Wall>,
    pub props: Vec<Prop>,
    next_id: u32,
}

impl Map {
    pub fn new(seed: u64) -> Map {
        Map { seed, next_id: ID_BASE, ..Map::default() }
    }

    fn id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    /// Makes the chunks around `at` that do not exist yet.
    pub fn ensure(&mut self, at: V2) {
        let (cx, cy) = chunk_of(at);
        for dy in -KEEP..=KEEP {
            for dx in -KEEP..=KEEP {
                let c = (cx + dx, cy + dy);
                if !self.made.contains_key(&c) {
                    let walls = self.generate(c);
                    self.made.insert(c, walls);
                }
            }
        }
    }

    /// One chunk: a few wall clusters and some props, never at the start.
    fn generate(&mut self, c: (i32, i32)) -> Vec<usize> {
        let mut rng = Rng::new(chunk_seed(self.seed, c), 11);
        let origin = V2::new(c.0 as f32 * CHUNK, c.1 as f32 * CHUNK);
        let mut out = Vec::new();
        let mut taken: Vec<(V2, f32)> = vec![(V2::ZERO, START_CLEAR)];
        let free = |taken: &Vec<(V2, f32)>, at: V2, r: f32| taken.iter().all(|(p, pr)| p.distance(at) > pr + r);
        let clusters = 1 + rng.below(3);
        for _ in 0..clusters {
            let at = origin + V2::new(rng.range(150.0, CHUNK - 150.0), rng.range(150.0, CHUNK - 150.0));
            if !free(&taken, at, 220.0) {
                continue;
            }
            taken.push((at, 220.0));
            let blocks: Vec<(WallKind, V2, V2)> = match rng.below(3) {
                // A row of server racks with gaps between them.
                0 => {
                    let vertical = rng.chance(0.5);
                    let n = 2 + rng.below(3) as i32;
                    (0..n)
                        .map(|k| {
                            let off = (k as f32 - (n - 1) as f32 / 2.0) * 110.0;
                            let p = if vertical { at + V2::new(0.0, off) } else { at + V2::new(off, 0.0) };
                            (WallKind::ServerRack, p, V2::new(36.0, 36.0))
                        })
                        .collect()
                }
                // A long billboard wall.
                1 => {
                    let half = if rng.chance(0.5) { V2::new(rng.range(120.0, 200.0), 20.0) } else { V2::new(20.0, rng.range(120.0, 200.0)) };
                    vec![(WallKind::Billboard, at, half)]
                }
                // An open-plan office: desks in a grid.
                _ => {
                    let mut v = Vec::new();
                    for gy in 0..2 {
                        for gx in 0..2 {
                            v.push((WallKind::Desk, at + V2::new((gx as f32 - 0.5) * 150.0, (gy as f32 - 0.5) * 110.0), V2::new(48.0, 26.0)));
                        }
                    }
                    v
                }
            };
            for (kind, centre, half) in blocks {
                let id = self.id();
                self.walls.push(Wall { id, kind, min: centre - half, max: centre + half, warning: 0, ticks: 0 });
                out.push(self.walls.len() - 1);
            }
        }
        // Breakables, and now and then something to use.
        let breakables = 2 + rng.below(3);
        for _ in 0..breakables {
            let at = origin + V2::new(rng.range(60.0, CHUNK - 60.0), rng.range(60.0, CHUNK - 60.0));
            let kind = match rng.below(10) {
                0..=4 => PropKind::PopupAd,
                5..=8 => PropKind::AdBillboard,
                _ => PropKind::CryptoMiner,
            };
            self.place(kind, at, &mut taken);
        }
        if rng.chance(0.75) {
            let at = origin + V2::new(rng.range(250.0, CHUNK - 250.0), rng.range(250.0, CHUNK - 250.0));
            let kind = match rng.below(100) {
                0..=17 => PropKind::Hotspot,
                18..=33 => PropKind::Vending,
                34..=48 => PropKind::Gacha,
                49..=62 => PropKind::Shrine,
                63..=76 => PropKind::DataCenter,
                77..=88 => PropKind::CommentSection,
                _ => PropKind::LostPhone,
            };
            self.place(kind, at, &mut taken);
        }
        out
    }

    fn place(&mut self, kind: PropKind, at: V2, taken: &mut Vec<(V2, f32)>) {
        let r = kind.radius() + 30.0;
        if !taken.iter().all(|(p, pr)| p.distance(at) > pr + r) {
            return;
        }
        taken.push((at, r));
        let left = match kind {
            PropKind::Hotspot => 60 * 25,
            PropKind::DataCenter => 60 * 20,
            _ => 0,
        };
        let id = self.id();
        self.props.push(Prop { id, kind, pos: at, charge: 0, state: PropState::Idle, left, price: 0 });
    }

    /// Solid walls and closed fences that may touch a circle at `at`.
    pub fn walls_near(&self, at: V2, r: f32) -> impl Iterator<Item = &Wall> {
        let (cx, cy) = chunk_of(at);
        let reach = if r > CHUNK / 2.0 { 2 } else { 1 };
        let mut list: Vec<&Wall> = Vec::new();
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                if let Some(ws) = self.made.get(&(cx + dx, cy + dy)) {
                    list.extend(ws.iter().map(|&i| &self.walls[i]));
                }
            }
        }
        list.extend(self.fences.iter().filter(|f| f.solid()));
        list.into_iter()
    }

    /// Moves a circle out of every wall it overlaps.
    pub fn resolve(&self, mut p: V2, r: f32) -> V2 {
        for _ in 0..2 {
            let mut moved = false;
            for w in self.walls_near(p, r) {
                if let Some(q) = w.push_out(p, r) {
                    p = q;
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }
        p
    }

    /// Whether a point is free of walls (with some room around it).
    pub fn is_free(&self, p: V2, r: f32) -> bool {
        self.walls_near(p, r).all(|w| w.push_out(p, r).is_none())
    }

    /// Whether a straight line from `a` to `b` hits a wall.
    pub fn blocked(&self, a: V2, b: V2) -> bool {
        self.walls_near(b, (b - a).length()).any(|w| w.blocks(a, b))
    }

    /// Closes off a rectangle for maintenance after a warning.
    pub fn add_fence(&mut self, centre: V2, half: V2, warning: u16, ticks: u16) {
        let id = self.id();
        self.fences.push(Wall { id, kind: WallKind::Fence, min: centre - half, max: centre + half, warning, ticks });
    }

    /// Fences count down; finished ones open again.
    pub fn tick_fences(&mut self) {
        for f in &mut self.fences {
            if f.warning > 0 {
                f.warning -= 1;
            } else {
                f.ticks = f.ticks.saturating_sub(1);
            }
        }
        self.fences.retain(|f| f.warning > 0 || f.ticks > 0);
    }

    /// Props within `range` of `at`.
    pub fn props_near(&self, at: V2, range: f32) -> impl Iterator<Item = (usize, &Prop)> {
        self.props.iter().enumerate().filter(move |(_, p)| p.pos.distance_sq(at) < range * range)
    }

    /// Every chunk made so far, for the client's background.
    pub fn chunks(&self) -> impl Iterator<Item = &(i32, i32)> {
        self.made.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: MAP-01
    #[test]
    fn the_same_seed_makes_the_same_map_in_any_order() {
        let mut a = Map::new(9);
        a.ensure(V2::ZERO);
        a.ensure(V2::new(9000.0, -4000.0));
        let mut b = Map::new(9);
        b.ensure(V2::new(9000.0, -4000.0));
        b.ensure(V2::ZERO);
        let walls = |m: &Map| {
            let mut v: Vec<(i32, i32, i32, i32)> = m.walls.iter().map(|w| (w.min.x as i32, w.min.y as i32, w.max.x as i32, w.max.y as i32)).collect();
            v.sort();
            v
        };
        assert_eq!(walls(&a), walls(&b));
        assert!(!a.walls.is_empty() && !a.props.is_empty());
        assert!(a.is_free(V2::ZERO, 60.0), "the start is clear");
    }

    // REQ: MAP-02
    #[test]
    fn circles_are_pushed_out_of_walls_and_lines_are_blocked() {
        let w = Wall { id: 1, kind: WallKind::Desk, min: V2::new(0.0, 0.0), max: V2::new(100.0, 50.0), warning: 0, ticks: 0 };
        let out = w.push_out(V2::new(50.0, 55.0), 10.0).expect("overlaps");
        assert!((out.y - 60.0).abs() < 0.01);
        assert!(w.push_out(V2::new(50.0, 80.0), 10.0).is_none());
        assert!(w.push_out(V2::new(50.0, 20.0), 10.0).is_some(), "a centre inside is pushed out too");
        assert!(w.blocks(V2::new(-20.0, 25.0), V2::new(120.0, 25.0)));
        assert!(!w.blocks(V2::new(-20.0, 80.0), V2::new(120.0, 80.0)));
    }
}

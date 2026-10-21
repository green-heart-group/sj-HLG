//! U13: seeded, repeatedly reconfigured Echo geography. Surface data is immutable.
use crate::{content::*, game::Game};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};

pub const ZONES: usize = 16;
pub const SIDE: usize = 8;
pub const CELLS: usize = SIDE * SIDE;
pub const ROOMS: usize = ZONES * CELLS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EchoLayout {
    pub seed: u64,
    pub revision: u32,
    pub epochs: Vec<u32>,
    pub zones: Vec<usize>,
    pub positions: Vec<[i32; 3]>,
    pub cells: Vec<Vec<usize>>,
    pub graph: Vec<Vec<usize>>,
}
fn rand(state: &mut u64, n: usize) -> usize {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    (*state as usize) % n
}
fn link(g: &mut [Vec<usize>], a: usize, b: usize) {
    if a != b && !g[a].contains(&b) {
        g[a].push(b);
        g[b].push(a);
    }
}
impl EchoLayout {
    pub fn new(seed: u64) -> Self {
        let base = *SOURCE_NODE_COUNT;
        let mut e = Self {
            seed: seed.max(1),
            revision: 0,
            epochs: vec![0; ZONES],
            zones: vec![0; NODES.len()],
            positions: vec![[0; 3]; NODES.len()],
            cells: vec![vec![]; ZONES],
            graph: vec![vec![]; NODES.len()],
        };
        for i in 0..base {
            e.zones[i] = i % ZONES;
        }
        for z in 0..ZONES {
            for i in 0..CELLS {
                e.zones[base + z * CELLS + i] = z;
            }
        }
        e.rebuild();
        e
    }
    pub fn zone(&self, n: usize) -> usize {
        self.zones[n]
    }
    pub fn position(&self, n: usize) -> [i32; 3] {
        self.positions[n]
    }
    pub fn neighbors(&self, n: usize) -> Vec<usize> {
        self.graph[n].clone()
    }
    pub fn regenerate(&mut self, z: Option<usize>) {
        self.revision += 1;
        match z {
            Some(z) => self.epochs[z] += 1,
            None => {
                for x in &mut self.epochs {
                    *x += 1;
                }
            }
        }
        self.rebuild();
    }
    fn rebuild(&mut self) {
        let base = *SOURCE_NODE_COUNT;
        self.graph = vec![vec![]; NODES.len()];
        for z in 0..ZONES {
            let mut rng = (self.seed
                ^ (z as u64 + 1).wrapping_mul(0x9e3779b97f4a7c15)
                ^ (self.epochs[z] as u64 + 1).wrapping_mul(0xd1b54a32d192ed03))
            .max(1);
            let mut ids: Vec<usize> = (0..CELLS).map(|i| base + z * CELLS + i).collect();
            for i in (1..CELLS).rev() {
                let j = rand(&mut rng, i + 1);
                ids.swap(i, j);
            }
            for (c, &id) in ids.iter().enumerate() {
                self.positions[id] = [
                    ((z % 4) * 10 + c % SIDE) as i32,
                    (c / SIDE) as i32,
                    (z / 4 + 1) as i32,
                ];
            }
            // A spanning tree plus extra links: every room is reachable; each epoch
            // has its own corridors, branches and loops.
            let mut seen = [false; CELLS];
            seen[0] = true;
            let mut stack = vec![0usize];
            while let Some(&c) = stack.last() {
                let opts = grid_neighbors(c)
                    .into_iter()
                    .filter(|&n| !seen[n])
                    .collect::<Vec<_>>();
                if opts.is_empty() {
                    stack.pop();
                    continue;
                }
                let n = opts[rand(&mut rng, opts.len())];
                seen[n] = true;
                link(&mut self.graph, ids[c], ids[n]);
                stack.push(n);
            }
            for c in 0..CELLS {
                for n in grid_neighbors(c) {
                    if n > c && rand(&mut rng, 5) == 0 {
                        link(&mut self.graph, ids[c], ids[n]);
                    }
                }
            }
            for id in 0..base {
                if self.zones[id] != z {
                    continue;
                }
                let host = ids[rand(&mut rng, CELLS)];
                self.positions[id] = self.positions[host];
                link(&mut self.graph, id, host);
            }
            self.cells[z] = ids;
        }
        for z in 0..ZONES {
            if z % 4 < 3 {
                link(
                    &mut self.graph,
                    self.cells[z][4 * SIDE + 7],
                    self.cells[z + 1][4 * SIDE],
                );
            }
            if z / 4 < 3 {
                link(
                    &mut self.graph,
                    self.cells[z][7 * SIDE + 4],
                    self.cells[z + 4][4],
                );
            }
        }
        for list in &mut self.graph {
            list.sort_unstable();
        }
    }
    pub fn edges(&self) -> BTreeSet<(usize, usize)> {
        self.graph
            .iter()
            .enumerate()
            .flat_map(|(a, bs)| bs.iter().filter(move |&&b| b > a).map(move |&b| (a, b)))
            .collect()
    }
    pub fn validate(&self) -> Result<(), String> {
        let n = NODES.len();
        if self.zones.len() != n
            || self.positions.len() != n
            || self.graph.len() != n
            || self.cells.len() != ZONES
            || self.epochs.len() != ZONES
        {
            return Err("里世界地图长度异常".into());
        }
        if self.zones.iter().any(|&z| z >= ZONES)
            || self.graph.iter().enumerate().any(|(a, bs)| {
                bs.iter()
                    .any(|&b| b >= n || b == a || !self.graph[b].contains(&a))
            })
        {
            return Err("里世界连接异常".into());
        }
        if self
            .cells
            .iter()
            .flatten()
            .copied()
            .collect::<BTreeSet<_>>()
            .len()
            != ROOMS
            || self.cells.iter().any(|c| c.len() != CELLS)
        {
            return Err("里世界房间索引异常".into());
        }
        let mut seen = vec![false; n];
        let mut q = VecDeque::from([0]);
        seen[0] = true;
        while let Some(a) = q.pop_front() {
            for &b in &self.graph[a] {
                if !seen[b] {
                    seen[b] = true;
                    q.push_back(b);
                }
            }
        }
        if seen.iter().any(|&b| !b) {
            return Err("里世界出现孤立房间".into());
        }
        Ok(())
    }
}
fn grid_neighbors(c: usize) -> Vec<usize> {
    let mut out = vec![];
    if c % SIDE > 0 {
        out.push(c - 1)
    }
    if c % SIDE < SIDE - 1 {
        out.push(c + 1)
    }
    if c >= SIDE {
        out.push(c - SIDE)
    }
    if c + SIDE < CELLS {
        out.push(c + SIDE)
    }
    out
}
impl Game {
    pub fn neighbors(&self, n: usize) -> Vec<usize> {
        if self.world == World::Echo {
            self.echo.neighbors(n)
        } else {
            neighbors(n, self.world)
        }
    }
    pub fn position(&self, n: usize) -> [i32; 3] {
        if self.world == World::Echo {
            self.echo.position(n)
        } else {
            [NODES[n].x, NODES[n].y, NODES[n].z]
        }
    }
    pub fn map_index(&self) -> i32 {
        if self.world == World::Echo {
            self.echo.zone(self.current) as i32
        } else {
            NODES[self.current].z
        }
    }
    pub fn shortest_step(&self, start: usize, end: usize) -> Option<usize> {
        let mut seen = vec![false; NODES.len()];
        let mut q = VecDeque::from([(start, None)]);
        seen[start] = true;
        while let Some((n, first)) = q.pop_front() {
            if n == end {
                return first;
            }
            for b in self.neighbors(n) {
                if !seen[b] && self.entry_reason(b).is_none() {
                    seen[b] = true;
                    q.push_back((b, first.or(Some(b))));
                }
            }
        }
        None
    }
    pub fn route_to(&self, end: usize) -> Vec<usize> {
        let mut parent = vec![usize::MAX; NODES.len()];
        let mut q = VecDeque::from([self.current]);
        parent[self.current] = self.current;
        while let Some(n) = q.pop_front() {
            if n == end {
                let mut path = vec![n];
                let mut at = n;
                while at != self.current {
                    at = parent[at];
                    path.push(at)
                }
                path.reverse();
                return path;
            }
            for b in self.neighbors(n) {
                if parent[b] == usize::MAX && self.entry_reason(b).is_none() {
                    parent[b] = n;
                    q.push_back(b);
                }
            }
        }
        vec![]
    }
    pub fn nearest_mirror(&self) -> usize {
        if self.world == World::Surface {
            return node("clock");
        }
        self.nearest_matching(|n| NODES[n].mirror)
            .unwrap_or(node("clock"))
    }
    pub fn nearest_matching(&self, p: impl Fn(usize) -> bool) -> Option<usize> {
        let mut q = VecDeque::from([self.current]);
        let mut seen = BTreeSet::from([self.current]);
        while let Some(n) = q.pop_front() {
            if p(n) {
                return Some(n);
            }
            for b in self.neighbors(n) {
                if seen.insert(b) {
                    q.push_back(b)
                }
            }
        }
        None
    }
    pub(crate) fn reconfigure_echo(&mut self, zone: Option<usize>, reason: &str) {
        let old_edges = self.echo.edges();
        let old_positions = self.echo.positions.clone();
        self.echo.regenerate(zone);
        let new_edges = self.echo.edges();
        let removed = old_edges.difference(&new_edges).count();
        let added = new_edges.difference(&old_edges).count();
        let moved = old_positions
            .iter()
            .zip(&self.echo.positions)
            .filter(|(a, b)| a != b)
            .count();
        let before = self.known[1].len();
        self.known[1].retain(|&n| zone.is_some_and(|z| self.echo.zone(n) != z));
        self.pits
            .retain(|&n, _| zone.is_some_and(|z| self.echo.zone(n) != z));
        let forgotten = before - self.known[1].len();
        self.last.major = true;
        self.note(format!("{reason}：里世界第 {} 次重组，{}。移动了 {moved} 个房间，移除 {removed} 条旧通路，生成 {added} 条新通路；{forgotten} 处旧地图标记需要重新确认。物品、成长和剧情记录保留。",
            self.echo.revision,zone.map_or("所有区段".into(),|z|format!("第 {} 区段",z+1))));
    }
    pub(crate) fn survey_echo(&mut self) {
        let z = self.echo.zone(self.current);
        // A successful observation traces the actual local corridors. It does
        // not invent a teleport or expose every other region.
        let ids = (0..NODES.len())
            .filter(|&n| self.echo.zone(n) == z)
            .collect::<Vec<_>>();
        self.known[1].extend(ids);
        self.note(format!(
            "已核实第 {} 区段当前的房间与通路；下一次本区重组后，这份地图会失效。",
            z + 1
        ));
    }
}

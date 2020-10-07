use crate::{Cell, Grid};
use js_sys::Math;

use priority_queue::PriorityQueue;
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
    fn h_cost(&self, another: &Self) -> usize {
        (self.x - another.x).pow(2) + (self.y - another.y).pow(2)
    }
}

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

use std::f32::consts::PI;

impl Direction {
    pub fn get_coordinate(&self, x: isize, y: isize) -> (isize, isize) {
        match self {
            Direction::North => (x, y - 1),
            Direction::NorthEast => (x + 1, y - 1),
            Direction::East => (x + 1, y),
            Direction::SouthEast => (x + 1, y + 1),
            Direction::South => (x, y + 1),
            Direction::SouthWest => (x - 1, y + 1),
            Direction::West => (x - 1, y),
            Direction::NorthWest => (x - 1, y - 1),
        }
    }
    pub fn g_cost(&self) -> usize {
        match self {
            Direction::North | Direction::East | Direction::South | Direction::West => 10,
            Direction::SouthEast
            | Direction::SouthWest
            | Direction::NorthEast
            | Direction::NorthWest => 14,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Node {
    pub pos: Position,
    pub parent: Option<Box<Node>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Cost {
    pub f_cost: usize,
    pub g_cost: usize,
    pub h_cost: usize,
}

impl Default for Cost {
    fn default() -> Self {
        let max = usize::MAX;
        Cost {
            f_cost: max,
            g_cost: max,
            h_cost: max,
        }
    }
}

use std::cmp::Ordering;

impl Ord for Cost {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.f_cost <= other.f_cost && self.h_cost < other.h_cost {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl PartialOrd for Cost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Node {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            pos: Position::new(x, y),
            parent: None,
        }
    }
    pub fn get_neighbour(&self, direction: Direction, grid: &Grid) -> Result<Self, &'static str> {
        let (x, y) = direction.get_coordinate(self.pos.x as isize, self.pos.y as isize);
        if x >= 0 && y >= 0 {
            let pos = Position::new(x as usize, y as usize);
            let neighbour = Self {
                pos,
                parent: Some(Box::new(self.clone())),
            };
            if let Some(cell) = grid.try_get(neighbour.pos.x, neighbour.pos.y) {
                if cell != Cell::Block {
                    return Ok(neighbour);
                }
            }
        }
        Err("This neighbour is outside the board!")
    }
    pub fn set_parent(&mut self, parent: Node) {
        self.parent = Some(Box::new(parent));
    }
}

#[derive(Clone)]
pub struct AStar {
    open: PriorityQueue<Node, Cost>,
    closed: HashSet<Position>,
    start: Position,
    target: Position,
}

impl AStar {
    pub fn not_start_nor_end(&self, pos: Position) -> bool {
        pos != self.start && pos != self.target
    }
    pub fn new(grid: &mut Grid) -> Self {
        let rand = Math::random();
        let (w, h) = (grid.width, grid.height);
        let get_x_y = || {
            let rand = Math::random();
            let x = (rand * w as f64) as usize;
            let y = (rand * h as f64) as usize;
            (x, y)
        };
        let mut open = PriorityQueue::new();
        let closed = HashSet::new();
        let (x, y) = get_x_y();
        grid.set(x, y, Cell::Start);
        let start = Position::new(x, y);
        let start_node = Node::new(x as usize, y as usize);
        open.push(start_node, Default::default());
        let (x, y1) = get_x_y();
        grid.set(x, y, Cell::End);
        let target = Position::new(x, y);
        Self {
            open,
            closed,
            start,
            target,
        }
    }
    pub fn find(&mut self, grid: &mut Grid) {
        let (current_node, current_cost) = self.open.pop().unwrap();
        self.closed.insert(current_node.pos);
        if current_node.pos == self.target {
            return;
        }
        for dir in Direction::iter() {
            if let Ok(mut neighbour) = current_node.get_neighbour(dir, grid) {
                if !self.closed.contains(&neighbour.pos) {
                    //let g_cost = neighbour.pos.h_cost(&self.target);
                    let cost = current_cost.g_cost + dir.g_cost();
                    let h_cost = neighbour.pos.h_cost(&self.target);
                    let neighbour_cost = Cost {
                        g_cost: cost,
                        h_cost,
                        f_cost: cost + h_cost,
                    };
                    let mut in_open = false;
                    for (old_n, old_n_cost) in self.open.iter_mut() {
                        if old_n.pos == neighbour.pos {
                            if cost < old_n_cost.g_cost {
                                old_n.set_parent(current_node.clone());
                                *old_n_cost = neighbour_cost;
                            }
                            in_open = true;
                            break;
                        }
                    }
                    if !in_open {
                        neighbour.set_parent(current_node.clone());
                        self.open.push(neighbour, neighbour_cost);
                    }
                }
            }
        }
    }
    pub fn fill(&mut self, grid: &mut Grid) -> bool {
        let (open, _) = self.open.peek().unwrap();
        let open = open.clone();
        if !self.open.is_empty() {
            if open.pos == self.target {
                true
            } else {
                self.find(grid);
                for (each, _) in self.open.iter() {
                    if self.not_start_nor_end(*&each.pos) {
                        grid.set(each.pos.x as usize, each.pos.y as usize, Cell::Visiting);
                    }
                }
                for each in self.closed.iter() {
                    if self.not_start_nor_end(*each) {
                        grid.set(each.x as usize, each.y as usize, Cell::Visited);
                    }
                }
                if self.not_start_nor_end(open.pos) {
                    grid.set(open.pos.x as usize, open.pos.y as usize, Cell::ShortestPath);
                }
                false
            }
        } else {
            false
        }
    }
    pub fn trace(&mut self, grid: &mut Grid) {
        if let Some((head, _)) = self.open.pop() {
            let mut current = head;
            while let Some(parent) = current.parent {
                if self.not_start_nor_end(parent.pos) {
                    grid.set(
                        parent.pos.x as usize,
                        parent.pos.y as usize,
                        Cell::ShortestPath,
                    );
                }
                current = *parent;
            }
        }
    }
}

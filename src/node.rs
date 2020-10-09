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
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
    pub fn h_cost(&self, another: &Self) -> usize {
        (self.x - another.x).pow(2) + (self.y - another.y).pow(2)
    }
}

pub fn is_odd(num: usize) -> bool {
    num & 1 == 0
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
    pub g_cost: usize,
    pub h_cost: usize,
}

impl Default for Cost {
    fn default() -> Self {
        let max = usize::MAX;
        Cost {
            g_cost: max,
            h_cost: max,
        }
    }
}

impl Cost {
    pub fn f_cost(&self) -> usize {
        self.g_cost + self.h_cost
    }
}

use std::cmp::Ordering;

impl Ord for Cost {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.f_cost() <= other.f_cost() && self.h_cost < other.h_cost {
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
        if x >= 0 && y >= 0 && x < grid.width as isize && y < grid.height as isize {
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
    diagonal: bool,
}

impl AStar {
    pub fn not_start_nor_end(&self, pos: Position) -> bool {
        pos != self.start && pos != self.target
    }
    pub fn new(grid: &mut Grid) -> Self {
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
        let start_node = Node::new(x, y);
        let (x, y) = get_x_y();
        grid.set(x, y, Cell::End);
        let target = Position::new(x, y);
        open.push(
            start_node,
            Cost {
                g_cost: 0,
                h_cost: start.h_cost(&target),
            },
        );
        let diagonal = false;
        Self {
            open,
            closed,
            start,
            target,
            diagonal,
        }
    }
    pub fn diagonal(&mut self, set_diagonal: bool) {
        self.diagonal = set_diagonal;
    }
    pub fn find(&mut self, grid: &mut Grid) {
        let (current_node, current_cost) = self.open.pop().unwrap();
        self.closed.insert(current_node.pos);
        if current_node.pos == self.target {
            return;
        }
        for (i, dir) in Direction::iter().enumerate() {
            if is_odd(i) || self.diagonal {
                if let Ok(mut neighbour) = current_node.get_neighbour(dir, grid) {
                    if !self.closed.contains(&neighbour.pos) {
                        //let g_cost = neighbour.pos.h_cost(&self.target);
                        let cost = current_cost.g_cost + dir.g_cost();
                        let h_cost = neighbour.pos.h_cost(&self.target);
                        let neighbour_cost = Cost {
                            g_cost: cost,
                            h_cost,
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
    }
    pub fn set_start(&mut self, start: Position) {
        self.start = start;
        self.clear();
    }
    pub fn set_target(&mut self, target: Position) {
        self.target = target;
        self.clear();
    }
    pub fn clear(&mut self) {
        self.open.clear();
        let start_node = Node::new(self.start.x, self.start.y);
        self.open.push(
            start_node,
            Cost {
                g_cost: 0,
                h_cost: self.start.h_cost(&self.target),
            },
        );
        self.closed.clear();
    }
    pub fn fill(&mut self, grid: &mut Grid) -> bool {
        if let Some(top) = self.open.peek() {
            let (open, _) = top;
            let open = open.clone();
            if !self.open.is_empty() {
                if open.pos == self.target {
                    return true;
                } else {
                    self.find(grid);
                    for (each, _) in self.open.iter() {
                        if self.not_start_nor_end(*&each.pos) {
                            grid.set(each.pos.x, each.pos.y, Cell::Visiting);
                        }
                    }
                    for each in self.closed.iter() {
                        if self.not_start_nor_end(*each) {
                            grid.set(each.x, each.y, Cell::Visited);
                        }
                    }
                    if self.not_start_nor_end(open.pos) {
                        grid.set(open.pos.x, open.pos.y, Cell::ShortestPath);
                    }
                }
            }
        }
        false
    }
    pub fn trace(&mut self) -> Vec<Position> {
        let mut path = Vec::new();
        if let Some((head, _)) = self.open.pop() {
            let mut current = head;
            while let Some(parent) = current.parent {
                if self.not_start_nor_end(parent.pos) {
                    path.push(parent.pos);
                }
                current = *parent;
            }
        }
        self.open.clear();
        path
    }
}

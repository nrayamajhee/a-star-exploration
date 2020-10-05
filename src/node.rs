use crate::{Cell, Grid, Renderer};
use js_sys::Math;

use std::collections::BinaryHeap;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Position<T> {
    pub x: T,
    pub y: T,
}

impl<T> Position<T> {
    fn new(x: T, y: T) -> Self {
        Self { x, y }
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
    pub fn get_coordinate(&self, x: i32, y: i32) -> (i32, i32) {
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
    pub fn distance_to(&self) -> f32 {
        match self {
            Direction::North | Direction::East | Direction::South | Direction::West => 1.0,
            Direction::SouthEast
            | Direction::SouthWest
            | Direction::NorthEast
            | Direction::NorthWest => f32::sqrt(2.),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Node {
    pub cost: usize,
    pub pos: Position<usize>,
    pub parent: Option<Box<Node>>,
}

use std::cmp::Ordering;
impl Ord for Node {
    fn cmp(&self, other: &Node) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Node {
    pub fn distance_to(&self, another: &Self) -> f32 {
        let dx = self.pos.x as f32 - another.pos.x as f32;
        let dy = self.pos.y as f32 - another.pos.y as f32;
        (dx.powi(2) + dy.powi(2)).sqrt()
    }
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            pos: Position::new(x, y),
            cost: usize::MAX,
            parent: None,
        }
    }
    pub fn set_parent(&mut self, parent: Node) {
        self.parent = Some(Box::new(parent));
    }
}

#[derive(Clone)]
pub struct AStar {
    open: BinaryHeap<Node>,
    closed: Vec<Node>,
    start: Node,
    target: Node,
}

impl AStar {
    pub fn new(grid: &mut Grid) -> Self {
        let rand = Math::random();
        let (w, h) = (grid.width, grid.height);
        let get_x_y = || {
            let rand = Math::random();
            let x = (rand * w as f64) as usize;
            let y = (rand * h as f64) as usize;
            (x, y)
        };
        let mut open = BinaryHeap::new();
        let closed = Vec::new();
        let (x, y) = get_x_y();
        grid.set(x, y, Cell::Start);
        let mut start = Node::new(x as usize, y as usize);
        let (x, y) = get_x_y();
        grid.set(x, y, Cell::End);
        let target = Node::new(x as usize, y as usize);
        open.push(start.clone());
        Self {
            open,
            closed,
            start,
            target,
        }
    }
    pub fn find(&mut self, grid: &mut Grid) {
        let current = self.open.pop().unwrap();
        self.closed.push(current.clone());
        if current == self.target {
            return;
        }
        for each in Direction::iter() {
            let (x, y) = each.get_coordinate(current.pos.x as i32, current.pos.y as i32);
            if x >= 0 && y >= 0 {
                let (x, y) = (x as usize, y as usize);
                let mut neighbour = Node::new(x, y);
                if let Some(cell) = grid.try_get(x, y) {
                    if !self.closed.iter().any(|e| e.pos == neighbour.pos) && cell != Cell::Block {
                        let n_cost = (each.distance_to() + neighbour.distance_to(&self.target))
                            .ceil() as usize;
                        let n_in_open = self.open.iter().any(|e| e.pos == neighbour.pos);
                        if n_cost < current.cost || !n_in_open {
                            neighbour.set_parent(current.clone());
                            neighbour.cost = n_cost;
                            if !n_in_open {
                                self.open.push(neighbour);
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn fill(&mut self, grid: &mut Grid) -> bool {
        if let Some(first) = self.open.peek() {
            if first.pos == self.target.pos {
                true
            } else {
                self.find(grid);
                //crate::log!("OPENED", self.start.pos);
                for each in self.open.iter() {
                    //crate::log!("EACh", each.pos);
                    if each.pos != self.start.pos && each.pos != self.target.pos {
                        grid.set(each.pos.x as usize, each.pos.y as usize, Cell::Visiting);
                    }
                }
                for each in self.closed.iter() {
                    if each.pos != self.start.pos && each.pos != self.target.pos {
                        grid.set(each.pos.x as usize, each.pos.y as usize, Cell::Visited);
                    }
                }
                let top = self.open.peek().unwrap();
                grid.set(top.pos.x as usize, top.pos.y as usize, Cell::ShortestPath);
                false
            }
        } else {
            false
        }
    }
    pub fn trace(&mut self, grid: &mut Grid) {
        if let Some(mut head) = self.open.peek() {
            while let Some(ref parent) = head.parent {
                grid.set(
                    parent.pos.x as usize,
                    parent.pos.y as usize,
                    Cell::ShortestPath,
                );
                head = parent;
            }
        }
    }
}

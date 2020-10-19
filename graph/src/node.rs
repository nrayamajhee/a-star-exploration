use crate::{Cell, Grid};
use priority_queue::PriorityQueue;
use rayon::prelude::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
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

impl From<u8> for Direction {
    fn from(from: u8) -> Self {
        for each in Direction::iter() {
            if each as u8 == from {
                return each;
            }
        }
        Direction::NorthWest
    }
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
    pub fn new_from_pos(pos: Position) -> Self {
        Self { pos, parent: None }
    }
    pub fn get_neighbour_grid_config(
        &self,
        direction: Direction,
        grid: &GridConfig,
    ) -> Result<Self, &'static str> {
        let (x, y) = direction.get_coordinate(self.pos.x as isize, self.pos.y as isize);
        let (width, height) = grid.dimension;
        if x >= 0 && y >= 0 && x < width as isize && y < height as isize {
            let pos = Position::new(x as usize, y as usize);
            let neighbour = Self {
                pos,
                parent: Some(Box::new(self.clone())),
            };
            if grid.blocked.contains(&neighbour.pos) {
                return Ok(neighbour);
            }
        }
        Err("This neighbour is outside the board!")
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
    pub open: PriorityQueue<Node, Cost>,
    pub closed: HashSet<Position>,
    pub start: Position,
    pub target: Position,
    pub diagonal: bool,
}

fn trace(open: Arc<Mutex<PriorityQueue<Node, Cost>>>) -> Vec<Position> {
    let mut path = Vec::new();
    if let Some((head, _)) = open.lock().unwrap().pop() {
        let mut current = head;
        while let Some(parent) = current.parent {
            path.push(parent.pos);
            current = *parent;
        }
    }
    path
}


impl AStar {
    pub fn new(start: Position, target: Position) -> Self {
        let mut open = PriorityQueue::new();
        let closed = HashSet::new();
        let start_node = Node::new_from_pos(start);
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
    pub fn not_start_nor_end(&self, pos: Position) -> bool {
        pos != self.start && pos != self.target
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

pub fn par_solve(
    start: Position,
    grid: &GridConfig,
    graph: &GraphConfig,
) -> Option<Vec<Position>> {
    if let Some(top) = graph.open.lock().expect("Cannot lock open").peek() {
        let (top, _) = top;
        if !graph.open.lock().expect("Cannot lock open").is_empty() {
            if top.pos == graph.target {
                return Some(trace(graph.open.clone()));
            } else {
                par_step(grid, graph);
                par_solve(start, grid, graph);
            }
        }
    }
    None
}

pub struct GridConfig {
    pub dimension: (usize, usize),
    pub blocked: HashSet<Position>,
}

pub struct GraphConfig {
    pub open: Arc<Mutex<PriorityQueue<Node, Cost>>>,
    pub closed: Arc<Mutex<HashSet<Position>>>,
    pub diagonal: bool,
    pub target: Position,
}

fn par_step(grid: &GridConfig, graph: &GraphConfig) {
    let (current_node, current_cost) = graph.open.lock().unwrap().pop().unwrap();
    graph.closed.lock().unwrap().insert(current_node.pos);
    if current_node.pos == graph.target {
        return;
    }
    let range = if graph.diagonal { 0..4 } else { 0..8 };
    range.into_par_iter().for_each(|i| {
        let dir = if graph.diagonal {
            Direction::from(i as u8)
        } else {
            Direction::from((i * 2) as u8)
        };
        if let Ok(mut neighbour) = current_node.get_neighbour_grid_config(dir, grid) {
            if !graph.closed.lock().unwrap().contains(&neighbour.pos) {
                //let g_cost = neighbour.pos.h_cost(&self.target);
                let cost = current_cost.g_cost + dir.g_cost();
                let h_cost = neighbour.pos.h_cost(&graph.target);
                let neighbour_cost = Cost {
                    g_cost: cost,
                    h_cost,
                };
                let mut in_open = false;
                for (old_n, old_n_cost) in graph.open.lock().unwrap().iter_mut() {
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
                    graph.open.lock().unwrap().push(neighbour, neighbour_cost);
                }
            }
        }
    });
}

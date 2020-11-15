use crate::{Cell, Grid, GridSet};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
    pub fn h_cost(&self, another: &Self) -> usize {
        (self.x as isize - another.x as isize).pow(2) as usize
            + (self.y as isize - another.y as isize).pow(2) as usize
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct Node {
    pub pos: Position,
    pub parent: Option<Box<Node>>,
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos.hash(state);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl Eq for Node {}

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
    pub fn get_neighbour(
        &self,
        direction: Direction,
        bounds: (usize, usize),
        check_block: &dyn Fn(Position) -> bool,
    ) -> Result<Self, String> {
        let (x, y) = direction.get_coordinate(self.pos.x as isize, self.pos.y as isize);
        if Self::within_bounds((x, y), bounds) {
            let pos = Position::new(x as usize, y as usize);
            if check_block(pos) {
                return Ok(Self {
                    pos,
                    parent: Some(Box::new(self.clone())),
                });
            } else {
                return Err(format!(
                    "There is a block {:?} of node at {:?}!",
                    direction, self.pos
                ));
            }
        }
        Err("This neighbour is outside the board!".into())
    }
    pub fn get_neighbour_from_grid(
        &self,
        direction: Direction,
        grid: &Grid,
    ) -> Result<Self, String> {
        self.get_neighbour(direction, grid.dimension(), &|pos| {
            if let Some(cell) = grid.try_get(pos.x, pos.y) {
                cell != Cell::Block
            } else {
                false
            }
        })
    }
    pub fn get_neighbour_from_set(
        &self,
        direction: Direction,
        grid: &GridSet,
    ) -> Result<Self, String> {
        self.get_neighbour(direction, grid.dimension(), &|pos| {
            !grid.set.par_iter().any(|e| *e == pos)
        })
    }
    fn within_bounds(pos: (isize, isize), dimension: (usize, usize)) -> bool {
        let (x, y) = pos;
        let (width, height) = dimension;
        x >= 0 && y >= 0 && x < width as isize && y < height as isize
    }
    pub fn set_parent(&mut self, parent: Node) {
        self.parent = Some(Box::new(parent));
    }
}

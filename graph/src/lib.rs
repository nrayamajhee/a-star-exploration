mod a_star;
mod grid;
mod graph;
mod node;

pub use a_star::*;
pub use graph::*;
pub use grid::*;
pub use node::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AStarConfig {
    pub start: Position,
    pub target: Position,
    pub diagonal: bool,
    pub multithreaded: bool,
    pub bidirectional: bool,
}

impl Default for AStarConfig {
    fn default() -> Self {
        Self {
            diagonal: true,
            start: Position::new(0, 0),
            target: Position::new(0, 0),
            multithreaded: false,
            bidirectional: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Request {
    pub dimension: (usize, usize),
    pub blocked: Vec<Position>,
    pub a_star: AStarConfig,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            dimension: (0, 0),
            blocked: Vec::new(),
            a_star: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Response {
    pub path: Vec<Position>,
    pub open: Vec<Position>,
    pub closed: Vec<Position>,
    pub time: usize,
}

impl Default for Response {
    fn default() -> Self {
        Self {
            path: Vec::new(),
            open: Vec::new(),
            closed: Vec::new(),
            time: 0,
        }
    }
}

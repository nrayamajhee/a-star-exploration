use crate::{is_odd, Cell, Cost, Direction, Grid, GridType, Node, Position};
use priority_queue::PriorityQueue;
use rayon::prelude::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

pub trait AStarTrait {
    fn top(&self) -> Option<Node>;
    fn diagonal(&self) -> bool;
    fn find(&mut self, grid: GridType);
    fn end_points(&self) -> (Position, Position);
    fn not_start_nor_end(&self, pos: Position) -> bool {
        let (start, target) = self.end_points();
        pos != start && pos != target
    }
    fn get_open_and_closed_list(&self) -> (Vec<Position>, Vec<Position>);
    fn solve(&mut self, grid: GridType) -> Vec<Position> {
        loop {
            let (_, target) = self.end_points();
            if let Some(top) = self.top() {
                if top.pos == target {
                    break;
                }
            } else {
                break;
            }
            self.find(grid);
        }
        self.trace()
    }
    fn trace(&self) -> Vec<Position> {
        let mut path = Vec::new();
        if let Some(mut current) = self.top() {
            while let Some(parent) = current.parent {
                if self.not_start_nor_end(current.pos) {
                    path.push(current.pos);
                }
                current = *parent;
            }
        }
        path
    }
}

#[derive(Clone)]
pub struct AStar {
    open: PriorityQueue<Node, Cost>,
    closed: HashSet<Position>,
    start: Position,
    pub target: Position,
    pub diagonal: bool,
}

#[derive(Clone)]
pub struct ParallelAStar {
    open: Arc<Mutex<PriorityQueue<Node, Cost>>>,
    closed: Arc<Mutex<HashSet<Position>>>,
    start: Position,
    pub target: Position,
    pub diagonal: bool,
}

impl AStar {
    pub fn new(start: Position, target: Position, diagonal: bool) -> Self {
        let mut open = PriorityQueue::new();
        let start_node = Node::new_from_pos(start);
        open.push(
            start_node,
            Cost {
                g_cost: 0,
                h_cost: start.h_cost(&target),
            },
        );
        let closed = HashSet::new();
        Self {
            open,
            closed,
            start,
            target,
            diagonal,
        }
    }
    pub fn set_start(&mut self, start: Position) {
        self.start = start;
        self.clear();
    }
    pub fn start(&self) -> Position {
        self.start
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
    pub fn step(&mut self, grid: &mut Grid) -> bool {
        if let Some(open) = self.top() {
            if !self.open.is_empty() {
                if open.pos == self.target {
                    return true;
                } else {
                    self.find(GridType::Full(&grid));
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
}

impl AStarTrait for AStar {
    fn top(&self) -> Option<Node> {
        if let Some((node, _)) = self.open.peek() {
            Some(node.clone())
        } else {
            None
        }
    }
    fn diagonal(&self) -> bool {
        self.diagonal
    }
    fn end_points(&self) -> (Position, Position) {
        (self.start, self.target)
    }
    fn get_open_and_closed_list(&self) -> (Vec<Position>, Vec<Position>) {
        let mut o = Vec::new();
        let mut c = Vec::new();
        for (each, _) in self.open.iter() {
            o.push(each.pos);
        }
        for each in self.closed.iter() {
            c.push(*each);
        }
        (o, c)
    }
    fn find(&mut self, grid_type: GridType) {
        let (current_node, current_cost) = self.open.pop().unwrap();
        self.closed.insert(current_node.pos);
        if current_node.pos == self.target {
            return;
        }
        for (i, dir) in Direction::iter().enumerate() {
            if is_odd(i) || self.diagonal {
                let neighbour = match grid_type {
                    GridType::Full(grid) => current_node.get_neighbour_from_grid(dir, grid),
                    GridType::Set(set) => current_node.get_neighbour_from_set(dir, set),
                };
                if let Ok(mut neighbour) = neighbour {
                    if !self.closed.contains(&neighbour.pos) {
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
}

impl ParallelAStar {
    pub fn new(start: Position, target: Position, diagonal: bool) -> Self {
        let mut open = PriorityQueue::new();
        let start_node = Node::new_from_pos(start);
        open.push(
            start_node,
            Cost {
                g_cost: 0,
                h_cost: start.h_cost(&target),
            },
        );
        let open = Arc::new(Mutex::new(open));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        Self {
            open,
            closed,
            start,
            target,
            diagonal,
        }
    }
}
impl AStarTrait for ParallelAStar {
    fn top(&self) -> Option<Node> {
        if let Some((node, _)) = self.open.lock().unwrap().peek() {
            Some(node.clone())
        } else {
            None
        }
    }
    fn diagonal(&self) -> bool {
        self.diagonal
    }
    fn end_points(&self) -> (Position, Position) {
        (self.start, self.target)
    }
    fn get_open_and_closed_list(&self) -> (Vec<Position>, Vec<Position>) {
        let mut o = Vec::new();
        let c = Vec::new();
        let clo = Arc::new(Mutex::new(c));
        self.closed
            .lock()
            .unwrap()
            .par_iter()
            .for_each(|e| clo.lock().unwrap().push(*e));
        for (each, _) in self.open.lock().unwrap().iter() {
            o.push(each.pos);
        }
        let c = Arc::try_unwrap(clo).unwrap().into_inner().unwrap();
        (o, c)
    }
    fn find(&mut self, grid_type: GridType) {
        let (current_node, current_cost) = self.open.lock().unwrap().pop().unwrap();
        self.closed.lock().unwrap().insert(current_node.pos);
        if current_node.pos == self.target {
            return;
        }
        let range = if self.diagonal { 0..8 } else { 0..4 };
        range.into_par_iter().for_each(|i| {
            let dir = if self.diagonal {
                Direction::from(i as u8)
            } else {
                Direction::from((i * 2) as u8)
            };
            let neighbour = match grid_type {
                GridType::Full(grid) => current_node.get_neighbour_from_grid(dir, grid),
                GridType::Set(set) => current_node.get_neighbour_from_set(dir, set),
            };
            if let Ok(mut neighbour) = neighbour {
                if !self
                    .closed
                    .lock()
                    .unwrap()
                    .par_iter()
                    .any(|e| *e == neighbour.pos)
                {
                    //let g_cost = neighbour.pos.h_cost(&self.target);
                    let cost = current_cost.g_cost + dir.g_cost();
                    let h_cost = neighbour.pos.h_cost(&self.target);
                    let neighbour_cost = Cost {
                        g_cost: cost,
                        h_cost,
                    };
                    let mut in_open = false;
                    for (old_n, old_n_cost) in self.open.lock().unwrap().iter_mut() {
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
                        self.open.lock().unwrap().push(neighbour, neighbour_cost);
                    }
                }
            }
        });
    }
}

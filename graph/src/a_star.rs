use crate::{is_odd, Cell, Cost, Direction, Grid, GridType, Node, Position};
use priority_queue::PriorityQueue;
use rayon::prelude::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

#[derive(Clone)]
pub enum AStarStructure {
    SingleThreaded(PriorityQueue<Node, Cost>, HashSet<Node>),
    Multithreaded(
        Arc<Mutex<PriorityQueue<Node, Cost>>>,
        Arc<Mutex<HashSet<Node>>>,
    ),
}

impl AStarStructure {
    pub fn new(start: Position, target: Position, multithreaded: bool) -> Self {
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
        if multithreaded {
            let open = Arc::new(Mutex::new(open));
            let closed = Arc::new(Mutex::new(closed));
            Self::Multithreaded(open, closed)
        } else {
            Self::SingleThreaded(open, closed)
        }
    }
    pub fn top(&self) -> Option<Node> {
        let mut top = None;
        match self {
            Self::SingleThreaded(open, _) => {
                if let Some((t, _)) = open.peek() {
                    top = Some(t.clone());
                }
            }
            Self::Multithreaded(open, _) => {
                if let Some((t, _)) = open.lock().unwrap().peek() {
                    top = Some(t.clone());
                }
            }
        }
        top
    }
    fn get_lists(&self) -> (Vec<Position>, Vec<Position>) {
        let mut o = Vec::new();
        let mut c = Vec::new();
        match self {
            Self::SingleThreaded(open, closed) => {
                for (each, _) in open.iter() {
                    o.push(each.pos);
                }
                for each in closed.iter() {
                    c.push(each.pos);
                }
            }
            Self::Multithreaded(open, closed) => {
                for (each, _) in open.lock().unwrap().iter() {
                    o.push(each.pos);
                }
                // This wasn't faster in parallel so resorting to default
                for each in closed.lock().unwrap().iter() {
                    c.push(each.pos);
                }
            }
        };
        (o, c)
    }
    pub fn clear(&mut self) {
        match self {
            Self::SingleThreaded(open, closed) => {
                open.clear();
                closed.clear();
            }
            Self::Multithreaded(open, closed) => {
                open.lock().unwrap().clear();
                closed.lock().unwrap().clear();
            }
        };
    }
    pub fn push_open(&mut self, start: Position, h_cost: usize) {
        let node = Node::new_from_pos(start);
        let cost = Cost { g_cost: 0, h_cost };
        match self {
            Self::SingleThreaded(open, _) => {
                open.push(node, cost);
            }
            Self::Multithreaded(open, _) => {
                open.lock().unwrap().push(node, cost);
            }
        };
    }
    pub fn push_closed(&mut self, node: Node) {
        match self {
            Self::SingleThreaded(_, closed) => closed.insert(node),
            Self::Multithreaded(_, closed) => closed.lock().unwrap().insert(node),
        };
    }
    pub fn pop_open(&mut self) -> Option<(Node, Cost)> {
        match self {
            Self::SingleThreaded(open, _) => open.pop(),
            Self::Multithreaded(open, _) => open.lock().unwrap().pop(),
        }
    }
    pub fn in_closed(&self, top: &Node) -> bool {
        match self {
            Self::SingleThreaded(_, closed) => closed.contains(&top),
            Self::Multithreaded(_, closed) => {
                // This wasn't faster in parallel so resorting to default
                closed.lock().unwrap().contains(&top)
            }
        }
    }
    pub fn solved(&self, target: Position, another: Option<&Self>) -> Option<Node> {
        if let Some(top) = self.top() {
            let mut solved = target == top.pos;
            if let Some(another) = another {
                solved = solved || another.in_closed(&top);
            }
            if solved {
                return Some(top);
            }
        }
        None
    }
    pub fn find(&mut self, grid_type: GridType, target: &Position, diagonal: bool) {
        let (current_node, current_cost) = self.pop_open().unwrap();
        for (i, dir) in Direction::iter().enumerate() {
            if is_odd(i) || diagonal {
                let neighbour = match grid_type {
                    GridType::Full(grid) => current_node.get_neighbour_from_grid(dir, grid),
                    GridType::Set(set) => current_node.get_neighbour_from_set(dir, set),
                };
                if let Ok(mut neighbour) = neighbour {
                    if !self.in_closed(&neighbour) {
                        let cost = current_cost.g_cost + dir.g_cost();
                        let h_cost = neighbour.pos.h_cost(target);
                        let neighbour_cost = Cost {
                            g_cost: cost,
                            h_cost,
                        };
                        let mut in_open = false;
                        let mut update_open = |node: (&mut Node, &mut Cost)| {
                            let (old_n, old_n_cost) = node;
                            if old_n.pos == neighbour.pos {
                                if cost < old_n_cost.g_cost {
                                    old_n.set_parent(current_node.clone());
                                    *old_n_cost = neighbour_cost;
                                }
                                in_open = true;
                                true
                            } else {
                                false
                            }
                        };
                        match self {
                            Self::SingleThreaded(open, _) => {
                                for each in open.iter_mut() {
                                    if update_open(each) {
                                        break;
                                    }
                                }
                            }
                            Self::Multithreaded(open, _) => {
                                for each in open.lock().unwrap().iter_mut() {
                                    if update_open(each) {
                                        break;
                                    }
                                }
                            }
                        };
                        if !in_open {
                            neighbour.set_parent(current_node.clone());
                            match self {
                                Self::SingleThreaded(open, _) => {
                                    open.push(neighbour, neighbour_cost)
                                }
                                Self::Multithreaded(open, _) => {
                                    open.lock().unwrap().push(neighbour, neighbour_cost)
                                }
                            };
                        }
                    }
                }
            }
        }
        self.push_closed(current_node);
    }
    pub fn trace(&self, common_node: &Node, end_points: (Position, Position)) -> Vec<Position> {
        let mut path = Vec::new();
        match self {
            Self::SingleThreaded(_, closed) => {
                for each in closed.iter() {
                    if each == common_node {
                        path = traverse(each, end_points);
                        break;
                    }
                }
            }
            Self::Multithreaded(_, closed) => {
                // This wasn't faster in parallel so resorting to default
                for each in closed.lock().unwrap().iter() {
                    if each == common_node {
                        path = traverse(each, end_points);
                        break;
                    }
                }
            }
        }
        path
    }
    pub fn modify_grid(&mut self, grid: &mut Grid, end_points: (Position, Position)) {
        let mut grid_set = |pos: Position, cell| {
            if pos != end_points.0 && pos != end_points.1 {
                grid.set(pos.x, pos.y, cell);
            }
        };
        match self {
            Self::SingleThreaded(open, closed) => {
                for (each, _) in open.iter() {
                    grid_set(each.pos, Cell::Visiting);
                }
                for each in closed.iter() {
                    grid_set(each.pos, Cell::Visited);
                }
            }
            Self::Multithreaded(open, closed) => {
                for (each, _) in open.lock().unwrap().iter() {
                    grid_set(each.pos, Cell::Visiting);
                }
                for each in closed.lock().unwrap().iter() {
                    grid_set(each.pos, Cell::Visited);
                }
            }
        }
        if let Some(top_node) = self.top() {
            grid_set(top_node.pos, Cell::ShortestPath);
        }
    }
}

#[derive(Clone)]
pub struct AStarBidirectional {
    start_data: AStarStructure,
    target_data: AStarStructure,
    start: Position,
    target: Position,
    multithreaded: bool,
    pub diagonal: bool,
    pub bidirectional: bool,
    common_node: Option<Node>,
}

impl AStarBidirectional {
    pub fn new(
        start: Position,
        target: Position,
        bidirectional: bool,
        diagonal: bool,
        multithreaded: bool,
    ) -> Self {
        let start_data = AStarStructure::new(start, target, multithreaded);
        let target_data = AStarStructure::new(target, start, multithreaded);
        Self {
            start_data,
            target_data,
            start,
            target,
            diagonal,
            bidirectional,
            multithreaded,
            common_node: None,
        }
    }
    pub fn solved(&mut self) -> bool {
        let mut solved = false;
        let mut another = None;
        if self.bidirectional {
            another = Some(&self.target_data);
        }
        if let Some(node) = self.start_data.solved(self.target, another) {
            self.common_node = Some(node);
            solved = true;
        }
        if self.bidirectional {
            if let Some(node) = self.target_data.solved(self.start, Some(&self.start_data)) {
                self.common_node = Some(node);
                solved = true;
            }
        }
        solved
    }
    pub fn clear(&mut self) {
        let h_cost = self.start.h_cost(&self.target);
        self.start_data.clear();
        self.start_data.push_open(self.start, h_cost);
        if self.bidirectional {
            self.target_data.clear();
            self.target_data.push_open(self.target, h_cost);
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
    pub fn start(&self) -> Position {
        self.start
    }
    pub fn target(&self) -> Position {
        self.target
    }
    pub fn step(&mut self, grid: &mut Grid) {
        if let Some(_) = self.start_data.top() {
            self.find(GridType::Full(&grid));
            let end_points = self.end_points();
            self.start_data.modify_grid(grid, end_points);
            if self.bidirectional {
                self.target_data.modify_grid(grid, end_points);
            }
        }
    }
    pub fn top(&self) -> Option<Node> {
        self.start_data.top()
    }
    pub fn diagonal(&self) -> bool {
        self.diagonal
    }
    pub fn end_points(&self) -> (Position, Position) {
        (self.start, self.target)
    }
    pub fn not_start_nor_end(&self, pos: Position) -> bool {
        let (start, target) = self.end_points();
        pos != start && pos != target
    }
    pub fn get_open_and_closed_list(&self) -> (Vec<Position>, Vec<Position>) {
        let (mut o, mut c) = self.start_data.get_lists();
        if self.bidirectional {
            let (o2, c2) = self.target_data.get_lists();
            o.extend(o2);
            c.extend(c2);
        }
        (o, c)
    }
    fn find(&mut self, grid_type: GridType) {
        if self.solved() {
            return;
        }
        self.start_data.find(grid_type, &self.target, self.diagonal);
        if self.bidirectional {
            self.target_data.find(grid_type, &self.start, self.diagonal);
        }
    }
    pub fn solve(&mut self, grid: GridType) -> Vec<Position> {
        loop {
            let (top, bottom) = (self.start_data.top(), self.target_data.top());
            if top == None || bottom == None {
                break;
            } else {
                if self.solved() {
                    break;
                } else {
                    self.find(grid);
                }
            }
        }
        self.trace()
    }
    pub fn trace(&self) -> Vec<Position> {
        let mut path = Vec::new();
        let end_points = self.end_points();
        if let Some(node) = &self.common_node {
            path = traverse(node, end_points);
            path.append(&mut self.start_data.trace(node, end_points));
            if self.bidirectional {
                path.append(&mut self.target_data.trace(node, end_points));
            }
        }
        path
    }
}

fn traverse(node: &Node, end_points: (Position, Position)) -> Vec<Position> {
    let mut path = Vec::new();
    let mut current = node.clone();
    let (start, target) = end_points;
    while let Some(parent) = current.parent {
        if current.pos != start && current.pos != target {
            path.push(current.pos);
        }
        current = *parent;
    }
    path
}

use crate::{is_odd, Cell, Cost, Direction, Grid, GridType, Node, Position};
use priority_queue::PriorityQueue;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
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

#[derive(Clone)]
pub enum AStarData {
    SingleThreaded(PriorityQueue<Node, Cost>, HashSet<Node>),
    Multithreaded(
        Arc<Mutex<PriorityQueue<Node, Cost>>>,
        Arc<Mutex<HashSet<Node>>>,
    ),
}

impl AStarData {
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
            Self::Multithreaded(Arc::new(Mutex::new(open)), Arc::new(Mutex::new(closed)))
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
                for each in closed.lock().unwrap().iter() {
                    c.push(each.pos);
                }
            }
        }
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
    pub fn push_node_open(&mut self, node: Node, cost: Cost) {
        match self {
            Self::SingleThreaded(open, _) => {
                open.push(node, cost);
            }
            Self::Multithreaded(open, _) => {
                open.lock().unwrap().push(node, cost);
            }
        }
    }
    pub fn push_open(&mut self, start: Position, h_cost: usize) {
        self.push_node_open(Node::new_from_pos(start), Cost { g_cost: 0, h_cost });
    }
    pub fn trace(&self, common_node: &Node, end_points: (Position, Position)) -> Vec<Position> {
        let mut path = Vec::new();
        let mut trace_node = |node: &Node| {
            let mut current = node.clone();
            let (start, target) = end_points;
            while let Some(parent) = current.parent {
                if current.pos != start && current.pos != target {
                    path.push(current.pos);
                }
                current = *parent;
            }
        };
        match self {
            Self::SingleThreaded(_, closed) => {
                for each in closed.iter() {
                    if each == common_node {
                        trace_node(each);
                        break;
                    }
                }
            }
            Self::Multithreaded(_, closed) => {
                for each in closed.lock().unwrap().iter() {
                    if each == common_node {
                        trace_node(each);
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
    pub fn get_neighbour(
        grid_type: GridType,
        current: (&Node, Cost),
        dir: Direction,
        target: &Position,
    ) -> Result<(Node, Cost), String> {
        let neighbour_node = match grid_type {
            GridType::Full(grid) => current.0.get_neighbour_from_grid(dir, grid),
            GridType::Set(set) => current.0.get_neighbour_from_set(dir, set),
        }?;
        let cost = current.1.g_cost + dir.g_cost();
        let h_cost = neighbour_node.pos.h_cost(target);
        let neighbour_cost = Cost {
            g_cost: cost,
            h_cost,
        };
        Ok((neighbour_node, neighbour_cost))
    }
    pub fn find_in_dir(
        target: &Position,
        current: (&Node, Cost),
        a_star: (&mut PriorityQueue<Node, Cost>, &mut HashSet<Node>),
        grid_type: GridType,
        dir: Direction,
    ) {
        let (current_node, current_cost) = current;
        let (open, closed) = a_star;
        if let Ok((mut neighbour, neighbour_cost)) =
            Self::get_neighbour(grid_type, (&current_node, current_cost), dir, target)
        {
            if !closed.contains(&neighbour) {
                let update_open = |(old_n, old_n_cost): (&mut Node, &mut Cost)| {
                    if old_n.pos == neighbour.pos {
                        if neighbour_cost.g_cost < old_n_cost.g_cost {
                            old_n.set_parent(current_node.clone());
                            *old_n_cost = neighbour_cost;
                        }
                        true
                    } else {
                        false
                    }
                };
                let mut in_open = false;
                for each in open.iter_mut() {
                    if update_open(each) {
                        in_open = true;
                        break;
                    }
                }
                if !in_open {
                    neighbour.set_parent(current_node.clone());
                    open.push(neighbour, neighbour_cost);
                }
            }
        }
    }
    pub fn find_in_dir_par(
        target: &Position,
        current: (&Node, Cost),
        a_star: (
            Arc<Mutex<PriorityQueue<Node, Cost>>>,
            Arc<Mutex<HashSet<Node>>>,
        ),
        grid_type: GridType,
        dir: Direction,
    ) {
        let (current_node, current_cost) = current;
        let (open, closed) = a_star;
        if let Ok((mut neighbour, neighbour_cost)) =
            Self::get_neighbour(grid_type, (&current_node, current_cost), dir, target)
        {
            if !closed.lock().unwrap().contains(&neighbour) {
                let update_open = |(old_n, old_n_cost): (&mut Node, &mut Cost)| {
                    if old_n.pos == neighbour.pos {
                        if neighbour_cost.g_cost < old_n_cost.g_cost {
                            old_n.set_parent(current_node.clone());
                            *old_n_cost = neighbour_cost;
                        }
                        true
                    } else {
                        false
                    }
                };
                let mut in_open = false;
                for each in open.lock().unwrap().iter_mut() {
                    if update_open(each) {
                        in_open = true;
                        break;
                    }
                }
                if !in_open {
                    neighbour.set_parent(current_node.clone());
                    open.lock().unwrap().push(neighbour, neighbour_cost);
                }
            }
        }
    }
    pub fn find(&mut self, grid_type: GridType, target: &Position, diagonal: bool) {
        match self {
            Self::SingleThreaded(open, closed) => {
                let (current_node, current_cost) = open.pop().unwrap();
                for (i, dir) in Direction::iter().enumerate() {
                    if is_odd(i) || diagonal {
                        Self::find_in_dir(
                            target,
                            (&current_node, current_cost),
                            (open, closed),
                            grid_type,
                            dir,
                        )
                    }
                }
                closed.insert(current_node);
            }
            Self::Multithreaded(_, _) => {
                self.find_par(grid_type, target, diagonal);
            }
        }
    }
    pub fn find_par(&self, grid_type: GridType, target: &Position, diagonal: bool) {
        if let Self::Multithreaded(open, closed) = self {
            let (current_node, current_cost) = open.lock().unwrap().pop().unwrap();
            let range = if diagonal { 0..8 } else { 0..4 };
            range.into_par_iter().for_each(|i| {
                let dir = if diagonal {
                    Direction::from(i as u8)
                } else {
                    Direction::from((i * 2) as u8)
                };
                Self::find_in_dir_par(
                    target,
                    (&current_node, current_cost),
                    (open.clone(), closed.clone()),
                    grid_type,
                    dir,
                );
            });
            closed.lock().unwrap().insert(current_node);
        }
    }
    fn in_closed(&self, node: &Node) -> bool {
        match self {
            Self::SingleThreaded(_, closed) => closed.contains(node),
            Self::Multithreaded(_, closed) => closed.lock().unwrap().contains(node),
        }
    }
    pub fn solved(&self, target: Position, another: Option<&Self>) -> Option<Node> {
        if let Some(top) = self.top() {
            let mut solved = target == top.pos;
            if let Some(another) = another {
                solved = solved || another.in_closed(&top);
            }
            if solved {
                return Some(top.clone());
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct AStarBidirectional {
    start: Position,
    start_data: AStarData,
    target: Position,
    target_data: Option<AStarData>,
    common_node: Option<Node>,
    pub diagonal: bool,
}

impl AStarBidirectional {
    pub fn new(config: AStarConfig) -> Self {
        let (start_data, target_data) = (
            AStarData::new(config.start, config.target, config.multithreaded),
            if config.bidirectional {
                Some(AStarData::new(
                    config.target,
                    config.start,
                    config.multithreaded,
                ))
            } else {
                None
            },
        );
        Self {
            start: config.start,
            start_data,
            target: config.target,
            target_data,
            common_node: None,
            diagonal: config.diagonal,
        }
    }
    pub fn multithreaded(&self) -> bool {
        if let AStarData::Multithreaded(_, _) = self.start_data {
            true
        } else {
            false
        }
    }
    pub fn bidirectional(&self) -> bool {
        if let Some(_) = self.target_data {
            true
        } else {
            false
        }
    }
    pub fn set_bidirectional(&mut self, bidirectional: bool) {
        if bidirectional {
            self.target_data = Some(AStarData::new(
                self.target,
                self.start,
                self.multithreaded(),
            ));
        } else {
            self.target_data = None;
        }
    }
    pub fn solved(&mut self) -> bool {
        let mut solved = false;
        if let Some(node) = self
            .start_data
            .solved(self.target, self.target_data.as_ref())
        {
            self.common_node = Some(node);
            solved = true;
        }
        if let Some(ref t_d) = self.target_data {
            if let Some(node) = t_d.solved(self.start, Some(&self.start_data)) {
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
        if let Some(ref mut t_d) = self.target_data {
            t_d.clear();
            t_d.push_open(self.target, h_cost);
        }
    }
    pub fn start(&self) -> Position {
        self.start
    }
    pub fn set_start(&mut self, start: Position) {
        self.start = start;
        self.clear();
    }
    pub fn target(&self) -> Position {
        self.target
    }
    pub fn set_target(&mut self, target: Position) {
        self.target = target;
        self.clear();
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
    pub fn step(&mut self, grid: &mut Grid) {
        if let Some(_) = self.start_data.top() {
            self.find(GridType::Full(&grid));
            let end_points = self.end_points();
            self.start_data.modify_grid(grid, end_points);
            if let Some(ref mut t_d) = self.target_data {
                t_d.modify_grid(grid, end_points);
            }
        }
    }
    pub fn not_start_nor_end(&self, pos: Position) -> bool {
        let (start, target) = self.end_points();
        pos != start && pos != target
    }
    pub fn get_open_and_closed_list(&self) -> (Vec<Position>, Vec<Position>) {
        let (mut o, mut c) = self.start_data.get_lists();
        if let Some(ref t_d) = self.target_data {
            let (o2, c2) = t_d.get_lists();
            o.extend(o2);
            c.extend(c2);
        }
        (o, c)
    }
    fn find(&mut self, grid_type: GridType) {
        if self.solved() {
            return;
        }
        let (multithreaded, bidirectional) = (self.multithreaded(), self.bidirectional());
        if multithreaded && bidirectional {
            let (start_data, target_data) = (
                self.start_data.clone(),
                self.target_data.as_ref().unwrap().clone(),
            );
            (0..2).into_par_iter().for_each(move |i| {
                if i == 0 {
                    start_data.find_par(grid_type, &self.target, self.diagonal);
                } else {
                    target_data.find_par(grid_type, &self.start, self.diagonal);
                }
            });
        } else {
            self.start_data.find(grid_type, &self.target, self.diagonal);
            if let Some(ref mut t_d) = self.target_data {
                t_d.find(grid_type, &self.start, self.diagonal);
            }
        }
    }
    pub fn solve(&mut self, grid: GridType) -> Vec<Position> {
        loop {
            let top = self.start_data.top();
            let mut open_empty = top == None;
            if let Some(ref t_d) = self.target_data {
                let bottom = t_d.top();
                open_empty = open_empty || bottom == None;
            }
            if open_empty || self.solved() {
                break;
            } else {
                self.find(grid);
            }
        }
        self.trace()
    }
    pub fn trace(&self) -> Vec<Position> {
        let mut path = Vec::new();
        let end_points = self.end_points();
        if let Some(node) = &self.common_node {
            let mut current = node.clone();
            let (start, target) = end_points;
            while let Some(parent) = current.parent {
                if current.pos != start && current.pos != target {
                    path.push(current.pos);
                }
                current = *parent;
            }
            path.append(&mut self.start_data.trace(node, end_points));
            if let Some(ref t_d) = self.target_data {
                path.append(&mut t_d.trace(node, end_points));
            }
        }
        path
    }
}

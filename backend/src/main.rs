use graph::{Cost, GraphConfig, GridConfig, Node, Position};
use priority_queue::PriorityQueue;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::sync::{Arc, Mutex};
use warp::Filter;

#[derive(Deserialize, Serialize)]
pub struct Graph {
    pub dimension: (usize, usize),
    pub blocked: Vec<Position>,
    pub diagonal: bool,
    pub start: Position,
    pub target: Position,
}

fn solve(graph: Graph) -> String {
    let blocked = HashSet::from_iter(graph.blocked);
    let mut open = PriorityQueue::new();
    let start = graph.start;
    let start_node = Node::new_from_pos(start);
    open.push(
        start_node,
        Cost {
            g_cost: 0,
            h_cost: start.h_cost(&graph.target),
        },
    );
    let open = Arc::new(Mutex::new(open));
    let closed = Arc::new(Mutex::new(HashSet::new()));
    let grid = GridConfig {
        dimension: graph.dimension,
        blocked,
    };
    let graph = GraphConfig {
        open,
        closed,
        diagonal: graph.diagonal,
        target: graph.target,
    };
    let solved = graph::par_solve(start, &grid, &graph);
    serde_json::to_string(&solved).unwrap()
}

#[tokio::main]
async fn main() {
    let solve = warp::post()
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(solve);
    warp::serve(solve).run(([127, 0, 0, 1], 8000)).await;
}

use a_star_graph::{AStarBidirectional, GridSet, GridType, Request, Response};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::time::Instant;
use warp::Filter;

fn solve(request: Request) -> String {
    let graph = request.a_star;
    let mut msg = format!(
        "{} -> {} \nBlockades: {}",
        graph.start,
        graph.target,
        request.blocked.len()
    );
    let set = HashSet::from_iter(request.blocked);
    let grid = GridSet {
        width: request.dimension.0,
        height: request.dimension.1,
        set,
    };
    if graph.diagonal {
        msg.push_str("\nWith diagonal search")
    }
    if graph.bidirectional {
        msg.push_str("\nWith bidirectional search")
    }
    if graph.multithreaded {
        msg.push_str(&format!(
            "\nMultithreaded from a pool of {} threads.",
            rayon::current_num_threads(),
        ));
    } else {
        msg.push_str("\nSingle threaded.")
    }
    let mut a_s = AStarBidirectional::new(graph);
    let then = Instant::now();
    let path = a_s.solve(GridType::Set(&grid));
    let time = then.elapsed().as_millis() as usize;
    println!("{}\nTook: {}ms", msg, time);
    let (open, closed) = a_s.get_open_and_closed_list();
    let response = Response {
        path,
        time,
        open,
        closed,
    };
    serde_json::to_string(&response).unwrap()
}

#[tokio::main]
async fn main() {
    //rayon::ThreadPoolBuilder::new()
    //.num_threads(4)
    //.build_global()
    //.unwrap();
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["POST"])
        .allow_header("content-type");
    let solve = warp::any()
        .and(warp::post())
        .and(warp::body::json())
        .map(solve)
        .with(cors);
    warp::serve(solve).run(([127, 0, 0, 1], 8000)).await;
}

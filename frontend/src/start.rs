use a_star_graph::{AStarBidirectional, Grid};
use js_sys::Math;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

use crate::{
    app::App,
    dom::{add_style, body, create_el},
    renderer::Renderer,
};

/// The entry point to our app
pub fn start() {
    add_style(
        "
        body {
            background: #222;
            height: 100vh;
            margin: 0;
            overflow: hidden;
            font: 16px/1 sans-serif;
            color: white;
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: center;
        }
        canvas {
            display: block;
        }
    ",
    );
    let canvas = create_el("canvas");
    body().append_child(&canvas).unwrap();
    let canvas = canvas.dyn_into::<HtmlCanvasElement>().unwrap();
    let renderer = Renderer::new(&canvas, 0., None);
    let mut grid = Grid::new(100, 50);
    let (start, target) = grid.set_rand_start_n_end(&|| Math::random());
    let graph = AStarBidirectional::new(start, target, false, false, false);
    let app = App::new(canvas, grid, graph, renderer);
    app.start();
}

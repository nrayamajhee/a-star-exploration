use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

use crate::{
    app::App,
    dom::{add_style, body, create_el, document},
    grid::Grid,
    renderer::Renderer,
    AStar, Node,
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
    document().set_title("A Star Exploration");
    let canvas = canvas.clone().dyn_into::<HtmlCanvasElement>().unwrap();
    let mut grid = Grid::new(50, 25);
    let renderer = Renderer::new(&canvas, 4., Some(2.));
    let graph = AStar::new(&mut grid);
    let app = App::new(canvas, grid, graph, renderer);
    app.start();
}

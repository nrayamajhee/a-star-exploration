use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

use crate::{
    app::App,
    dom::{add_style, body, create_el},
    grid::Grid,
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
            display: flex;
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
    let canvas = canvas.clone().dyn_into::<HtmlCanvasElement>().unwrap();
    let grid = Grid::new(150, 75);
    let renderer = Renderer::new(&canvas, 1., None);
    let app = App::new(canvas, grid, renderer);
    app.bind_events();
    app.start();
}

//use rand::prelude::*;
use wasm_bindgen::prelude::*;

mod grid;
pub use grid::Grid;

mod dom;
use dom::{add_event, add_style, body, create_el, window};

mod renderer;
use renderer::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen(start)]
pub async fn run() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
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
    let grid = Rc::new(Grid::new(100, 50));
    let renderer = Renderer::new(canvas.dyn_into::<HtmlCanvasElement>().unwrap(), grid.clone());
    //console::log_1(&format!("{} x {}\n{} x {}", width, height, dx, dy).into());
    //console::log_1(&format!("{} x {}", x, y).into());
    renderer.draw_grid(&grid, 2.);
    add_event(&window(), "resize", move |_| {
        renderer.draw_grid(&grid, 2.);
    });
    Ok(())
}

use crate::{
    dom::{add_event, add_style, body, create_el, document, window, RcCell},
    grid::{Cell, Grid},
    renderer::Renderer,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, MouseEvent};

pub async fn start() -> Result<(), JsValue> {
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
    let h_canvas = canvas.clone().dyn_into::<HtmlCanvasElement>().unwrap();
    let grid = RcCell::new(Grid::new(50, 25));
    let renderer = Renderer::new(&h_canvas, grid.clone(), 5);
    let renderer = RcCell::new(renderer);
    let r = renderer.clone();
    add_event(&canvas, "mousedown", move |e| {
        let me = e.dyn_into::<MouseEvent>().unwrap();
        let (x, y) = (me.client_x(), me.client_y());
        let (i, j) = r.borrow().get_indices(x as usize, y as usize);
        grid.borrow_mut().set(i, j, Cell::Path);
        r.borrow().draw_grid();
    });
    let r = renderer.clone();
    add_event(&window(), "resize", move |_| {
        r.borrow_mut().resize_canvas(&h_canvas);
    });
    renderer.borrow().draw_grid();
    Ok(())
}

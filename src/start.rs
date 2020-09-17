use crate::{
    dom::{add_event, window, add_style, body, create_el, loop_animation_frame, RcCell},
    grid::{Cell, Grid},
    renderer::Renderer,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, MouseEvent};

use crate::log;

#[derive(Clone, Copy)]
pub enum MouseState {
    Down(usize, usize),
    Move(usize, usize),
}

#[derive(Clone, Copy)]
pub enum AppEvent {
    Fill(usize, usize),
    Resize,
    None,
}
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
    let canvas = canvas.clone().dyn_into::<HtmlCanvasElement>().unwrap();
    let eve = RcCell::new(AppEvent::None);
    let events = eve.clone();
    add_event(&canvas, "mousedown", move |e| {
        let me = e.dyn_into::<MouseEvent>().unwrap();
        events.mutate(AppEvent::Fill(
            me.offset_x() as usize,
            me.offset_y() as usize,
        ));
    });
    let events = eve.clone();
    add_event(&canvas, "mousemove", move |e| {
        let me = e.dyn_into::<MouseEvent>().unwrap();
        if let AppEvent::Fill(ref mut x, ref mut y) = *events.borrow_mut() {
            *x = me.offset_x() as usize;
            *y = me.offset_y() as usize;
        }
    });
    let events = eve.clone();
    add_event(&canvas, "mouseup", move |e| {
        events.mutate(AppEvent::None);
    });
    let mut grid = Grid::new(50, 25);
    let mut renderer = Renderer::new(&canvas, 5);
    let events = eve.clone();
    add_event(&window(), "resize", move |e| {
        events.mutate(AppEvent::Resize);
    });
    let events = eve.clone();
    loop_animation_frame(
        move |_| {
            match *events.borrow() {
                AppEvent::Fill(x, y) => {
                    let (col, row) = renderer.get_indices(x, y);
                    if grid.get(row, col) == Cell::Path  {
                        grid.set(row, col, Cell::Block);
                    }
                }
                AppEvent::Resize => {
                    renderer.resize(&canvas, &grid);
                }
                _ => (),
            }
            renderer.draw_grid(&grid);
        },
        None,
    );
    eve.mutate(AppEvent::Resize);
    Ok(())
}

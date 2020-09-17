use crate::{
    dom::{add_event_mut, loop_animation_frame, window},
    grid::{Cell, Grid},
    renderer::Renderer,
    RcCell,
};
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, MouseEvent};

#[derive(Clone, Copy)]
pub enum AppEvent {
    Fill(Cell, usize, usize),
    Resize,
    None,
}

#[derive(Clone)]
pub struct App {
    grid: Grid,
    canvas: HtmlCanvasElement,
    renderer: Renderer,
    event: RcCell<AppEvent>,
}

impl App {
    pub fn new(canvas: HtmlCanvasElement, grid: Grid, renderer: Renderer) -> Self {
        let event = RcCell::new(AppEvent::None);
        event.mutate(AppEvent::Resize);
        Self {
            grid,
            renderer,
            canvas,
            event,
        }
    }
    pub fn bind_events(&self) {
        add_event_mut(&self.canvas, "mousedown", &self.event, |event, e| {
            let me = e.dyn_into::<MouseEvent>().unwrap();
            let button = me.buttons();
            let fill_type = if button == 1 {
                Some(Cell::Block)
            } else if button == 2 {
                Some(Cell::Path)
            } else {
                None
            };
            if let Some(fill_type) = fill_type {
                *event = AppEvent::Fill(fill_type, me.offset_x() as usize, me.offset_y() as usize);
            }
        });
        add_event_mut(&self.canvas, "mousemove", &self.event, |event, e| {
            let me = e.dyn_into::<MouseEvent>().unwrap();
            if let AppEvent::Fill(_, ref mut x, ref mut y) = event {
                *x = me.offset_x() as usize;
                *y = me.offset_y() as usize;
            }
        });
        add_event_mut(&self.canvas, "mouseup", &self.event, |event, _| {
            *event = AppEvent::None;
        });
        add_event_mut(&window(), "resize", &self.event, |event, _| {
            *event = AppEvent::Resize;
        });
    }
    pub fn start(mut self) {
        loop_animation_frame(
            move |_| {
                let mut reset = false;
                match *self.event.borrow() {
                    AppEvent::Fill(fill_type, x, y) => {
                        let (col, row) = self.renderer.get_indices(x, y);
                        match self.grid.get(row, col) {
                            Cell::Path | Cell::Block => {
                                self.grid.set(row, col, fill_type);
                            }
                            _ => (),
                        }
                    }
                    AppEvent::Resize => {
                        self.renderer.resize(&self.canvas, &self.grid);
                        reset = true;
                    }
                    _ => (),
                }
                self.renderer.draw_grid(&self.grid);
                if reset {
                    self.event.mutate(AppEvent::None);
                }
            },
            None,
        );
    }
}

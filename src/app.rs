use crate::{
    dom::{add_event_mut, body, create_el, loop_animation_frame, window},
    grid::{Cell, Grid},
    renderer::Renderer,
    AStar, RcCell,
};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlCanvasElement, MouseEvent};

#[derive(Clone, Copy)]
pub enum AppEvent {
    Fill(Cell, usize, usize),
    Resize,
    Next,
    Play,
    None,
}

#[derive(Clone)]
pub struct App {
    grid: Grid,
    canvas: HtmlCanvasElement,
    renderer: Renderer,
    graph: AStar,
    event: RcCell<AppEvent>,
}

impl App {
    pub fn new(canvas: HtmlCanvasElement, grid: Grid, graph: AStar, renderer: Renderer) -> Self {
        let event = RcCell::new(AppEvent::None);
        let button = create_el("button");
        button.set_inner_html("Next");
        body().append_child(&button).unwrap();
        let button_p = create_el("button");
        button_p.set_inner_html("Play");
        body().append_child(&button_p).unwrap();
        event.mutate(AppEvent::Resize);
        add_event_mut(&button, "click", &event, |event, e| {
            *event = AppEvent::Next;
        });
        add_event_mut(&button_p, "click", &event, |event, e| {
            *event = AppEvent::Play;
        });
        add_event_mut(&canvas, "mousedown", &event, |event, e| {
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
        add_event_mut(&canvas, "mousemove", &event, |event, e| {
            let me = e.dyn_into::<MouseEvent>().unwrap();
            if let AppEvent::Fill(_, ref mut x, ref mut y) = event {
                *x = me.offset_x() as usize;
                *y = me.offset_y() as usize;
            }
        });
        add_event_mut(&canvas, "mouseup", &event, |event, _| {
            *event = AppEvent::None;
        });
        add_event_mut(&window(), "resize", &event, |event, _| {
            *event = AppEvent::Resize;
        });
        Self {
            grid,
            graph,
            renderer,
            canvas,
            event,
        }
    }
    pub fn start(mut self) {
        loop_animation_frame(
            move |_| {
                let mut reset = true;
                match *self.event.borrow() {
                    AppEvent::Fill(fill_type, x, y) => {
                        let (col, row) = self.renderer.get_indices(y, x);
                        match self.grid.get(col, row) {
                            Cell::Path | Cell::Block => {
                                self.grid.set(row, col, fill_type);
                            }
                            _ => (),
                        }
                        reset = false;
                    }
                    AppEvent::Resize => {
                        self.renderer.resize(&self.canvas, &self.grid);
                    }
                    AppEvent::Play => {
                        if self.graph.fill(&mut self.grid) {
                            self.graph.trace(&mut self.grid);
                        } else {
                            reset = false;
                        }
                    }
                    AppEvent::Next => {
                        self.graph.fill(&mut self.grid);
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

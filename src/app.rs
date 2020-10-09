use crate::{
    dom::{
        add_event_mut, add_style, body, document, event_as_input, get_el, insert_html_at,
        loop_animation_frame, window, HtmlPosition,
    },
    AStar, Cell, DrawMode, Grid, Position, RcCell, Renderer,
};
use maud::html;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, MouseEvent};

#[derive(Clone, Copy, PartialEq)]
pub enum AppEvent {
    Fill(Position, Cell),
    Drag(Position, Option<Position>, Cell),
    Trace,
    Play,
    Diagonal(bool),
    SetNodes(Position, Cell),
    Step,
    Clear,
    Resize,
    None,
}

#[derive(Clone)]
pub struct App {
    grid: Grid,
    path: Vec<Position>,
    canvas: HtmlCanvasElement,
    renderer: Renderer,
    graph: AStar,
    event: RcCell<AppEvent>,
}

impl App {
    pub fn new(canvas: HtmlCanvasElement, grid: Grid, graph: AStar, renderer: Renderer) -> Self {
        document().set_title("A Star Exploration");
        add_style(
            "
            #bottom-bar {
                width: 100vw;
                display: flex;
                justify-content: space-around;
                margin: 5px 0;
            }
            #bottom-bar p {
                line-height: 1;
                border: 2px solid #555;
                display: inline-block;
                padding: 5px;
                background-color: #333;
                border-radius: 5px;
                margin: 0 5px;
            }
            label {
                user-select: none;
            }
            button {
                margin: 0 5px;
                line-height: 1;
                padding: 5px;
                border: 2px solid #555;
                background: #333;
                border-radius: 5px;
                color: white;
            }
        ",
        );
        let html = html! {
            section#bottom-bar {
                #div.left {
                    button#play { "Play" }
                    button#step { "Step" }
                    button#clear { "Clear" }
                }
                #div.right {
                    input id="diag" type="checkbox" {}
                    label for="diag" {"Diagonal Search"}
                    p{"Left Click: Draw"}
                    p{"Right Click: Erase"}
                    p{"Drag start/end position"}
                }
            }
        }
        .into_string();
        insert_html_at(&body(), html.as_str(), HtmlPosition::End);
        let event = RcCell::new(AppEvent::Resize);
        let path = Vec::new();
        let app = Self {
            grid,
            path,
            graph,
            renderer,
            canvas,
            event,
        };
        app.bind_events();
        app
    }
    pub fn bind_events(&self) {
        add_event_mut(&get_el("play"), "click", &self.event, |event, _| {
            *event = AppEvent::Play;
        });
        add_event_mut(&get_el("step"), "click", &self.event, |event, _| {
            *event = AppEvent::Step;
        });
        add_event_mut(&get_el("clear"), "click", &self.event, |event, _| {
            *event = AppEvent::Clear;
        });
        add_event_mut(&get_el("diag"), "input", &self.event, |event, e| {
            *event = AppEvent::Diagonal(event_as_input(&e).checked());
        });
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
                *event = AppEvent::Fill(
                    Position::new(me.offset_x() as usize, me.offset_y() as usize),
                    fill_type,
                );
            }
        });
        add_event_mut(&self.canvas, "mousemove", &self.event, |event, e| {
            let me = e.dyn_into::<MouseEvent>().unwrap();
            if let AppEvent::Fill(ref mut pos, _) = event {
                pos.x = me.offset_x() as usize;
                pos.y = me.offset_y() as usize;
            } else if let AppEvent::Drag(ref mut old_pos, ref mut pos, _) = event {
                if let Some(n_p) = pos {
                    *old_pos = *n_p;
                }
                *pos = Some(Position::new(
                    me.offset_x() as usize,
                    me.offset_y() as usize,
                ));
            }
        });
        let window = window();
        add_event_mut(&window, "mouseup", &self.event, |event, _| {
            if let AppEvent::Drag(_, new_pos, cell_type) = event {
                if let Some(n_p) = new_pos {
                    *event = AppEvent::SetNodes(*n_p, *cell_type);
                }
            } else {
                *event = AppEvent::None;
            }
        });
        add_event_mut(&window, "resize", &self.event, |event, _| {
            *event = AppEvent::Resize;
        });
    }
    pub fn start(mut self) {
        loop_animation_frame(
            move |_| {
                let mut reset = true;
                let mut event = self.event.borrow_mut();
                match *event {
                    AppEvent::Fill(pos, fill_type) => {
                        let (row, col) = self.renderer.get_indices(pos.x, pos.y);
                        match self.grid.get(col, row) {
                            Cell::Path | Cell::Block => {
                                self.grid.set(row, col, fill_type);
                            }
                            Cell::Start => *event = AppEvent::Drag(pos, None, Cell::Start),
                            Cell::End => *event = AppEvent::Drag(pos, None, Cell::End),
                            _ => (),
                        }
                        reset = false;
                    }
                    AppEvent::Drag(old_pos, new_pos, fill_type) => {
                        let (row, col) = self.renderer.get_indices(old_pos.x, old_pos.y);
                        self.grid.set(row, col, Cell::Path);
                        if let Some(n_p) = new_pos {
                            let (row, col) = self.renderer.get_indices(n_p.x, n_p.y);
                            self.grid.set(row, col, fill_type);
                        }
                        reset = false;
                    }
                    AppEvent::SetNodes(pos, cell) => {
                        let (row, col) = self.renderer.get_indices(pos.x, pos.y);
                        let pos = Position::new(row, col);
                        if cell == Cell::Start {
                            self.graph.set_start(pos);
                        } else if cell == Cell::End {
                            self.graph.set_target(pos);
                        }
                    }
                    AppEvent::Resize => {
                        self.renderer.resize(&self.canvas, &self.grid);
                    }
                    AppEvent::Play | AppEvent::Step => {
                        crate::log!("STEP");
                        if self.graph.fill(&mut self.grid) {
                            self.path = self.graph.trace();
                            *event = AppEvent::Trace;
                        }
                        match *event {
                            AppEvent::Play | AppEvent::Trace => {
                                reset = false;
                            }
                            _ => (),
                        }
                    }
                    AppEvent::Trace => {
                        for each in &self.path {
                            self.grid.set(each.x, each.y, Cell::ShortestPath);
                        }
                    }
                    AppEvent::Diagonal(diag) => {
                        self.graph.diagonal(diag);
                    }
                    AppEvent::Clear => {
                        self.graph.clear();
                        self.grid.clear();
                    }
                    _ => (),
                }
                self.renderer.draw_grid(&self.grid, DrawMode::Circle);
                if reset {
                    *event = AppEvent::None;
                }
            },
            None,
        );
    }
}

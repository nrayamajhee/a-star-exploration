use crate::{
    dom::{
        add_event_mut, add_style, body, document, event_as_input, fetch_then, get_el, get_value,
        insert_html_at, loop_animation_frame, window, FetchMethod, HtmlPosition,
    },
    DrawMode, RcCell, Renderer,
};
use a_star_graph::{AStarBidirectional, AStarConfig, Cell, Grid, Position, Request, Response};
use js_sys::Math;
use maud::html;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, MouseEvent};

#[derive(Clone, PartialEq, Debug)]
pub enum AppEvent {
    Mouse(Position, Option<Position>, Cell),
    Trace,
    TraceResponse,
    Play,
    Solve,
    Diagonal(bool),
    Multithreaded(bool),
    Bidirectional(bool),
    Step,
    Clear(bool),
    Resize,
    ResizeGrid,
    None,
}

pub struct App {
    grid: Grid,
    canvas: HtmlCanvasElement,
    renderer: Renderer,
    graph: AStarBidirectional,
    event: RcCell<AppEvent>,
    response: RcCell<Response>,
    multithreaded: bool,
    solved: bool,
}

impl App {
    pub fn new(
        canvas: HtmlCanvasElement,
        grid: Grid,
        graph: AStarBidirectional,
        renderer: Renderer,
    ) -> Self {
        document().set_title("A Star Exploration");
        add_style(
            "
            .bar {
                width: 100vw;
                display: flex;
                justify-content: space-around;
                margin: 5px 0;
            }
            .bar p, button, input[type=number], input[type=checkbox] + label::before {
                display: inline-block;
                background-color: #333;
                border-radius: 5px;
                border: 2px solid transparent;
                margin: 0 2px;
                line-height: 1;
                padding: 5px;
                color: white;
            }
            label, .bar span {
                display: inline-block;
                user-select: none;
                margin: 7px 2px;
                line-height: 1;
            }
            button, intput {
                border-color: #555;
            }
            input[type=number] {
                height: 1em;
            }
            input[type=checkbox] {
                display: none;
            }
            input[type=checkbox] + label::before {
                content: '';
                width: 1rem;
                height: 1rem;
                padding: 0;
                border: 0;
                margin-right: 5px;
                padding: 0 4px;
                box-sizing: border-box;
            }
            input[type=checkbox]:checked + label::before {
                content: 'x';
            }
            input[type=number] {
                width: 4em;
            }
            input::-webkit-outer-spin-button,
            input::-webkit-inner-spin-button {
                -webkit-appearance: none;
                margin: 0;
            }
            input[type=number] {
                -moz-appearance: textfield;
            }
        ",
        );
        let top_bar = html! {
            #top.bar {
                .left {
                    button#play { "Play" }
                    button#step { "Step" }
                    button#solve { "Solve" }
                    span#time {}
                }
                .center {
                    input id="multi" type="checkbox" {}
                    label for="multi" {"Multithreaded"}
                    input id="bi" type="checkbox" {}
                    label for="bi" {"Bi-directional"}
                    input id="diag" type="checkbox" {}
                    label for="diag" {"Diagonal"}
                }
                .right {
                    button#clear { "Clear" }
                    button#clear-w { "Clear All" }
                }
            }
        }
        .into_string();
        let bottom_bar = html! {
            #bottom.bar {
                .left {
                    p{"Left Click: Draw"}
                    p{"Right Click: Erase"}
                    p{"Drag start/end position"}
                }
                .center {
                }
                .right {
                    label { "Grid Size" }
                    input#width min="0" value="100" type="number" {}
                    span {" x "}
                    input#height min="0" value="50" type="number" {}
                }
            }
        }
        .into_string();
        insert_html_at(&body(), top_bar.as_str(), HtmlPosition::Start);
        insert_html_at(&body(), bottom_bar.as_str(), HtmlPosition::End);
        let event = RcCell::new(AppEvent::Resize);
        let app = Self {
            grid,
            graph,
            renderer,
            canvas,
            event,
            response: RcCell::new(Default::default()),
            multithreaded: false,
            solved: false,
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
            *event = AppEvent::Clear(false);
        });
        add_event_mut(&get_el("clear-w"), "click", &self.event, |event, _| {
            *event = AppEvent::Clear(true);
        });
        add_event_mut(&get_el("solve"), "click", &self.event, |event, _| {
            *event = AppEvent::Solve;
        });
        add_event_mut(&get_el("diag"), "input", &self.event, |event, e| {
            *event = AppEvent::Diagonal(event_as_input(&e).checked());
        });
        add_event_mut(&get_el("bi"), "input", &self.event, |event, e| {
            *event = AppEvent::Bidirectional(event_as_input(&e).checked());
        });
        add_event_mut(&get_el("multi"), "input", &self.event, |event, e| {
            *event = AppEvent::Multithreaded(event_as_input(&e).checked());
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
                *event = AppEvent::Mouse(
                    Position::new(me.offset_x() as usize, me.offset_y() as usize),
                    None,
                    fill_type,
                );
            }
        });
        add_event_mut(&self.canvas, "mousemove", &self.event, |event, e| {
            let me = e.dyn_into::<MouseEvent>().unwrap();
            if let AppEvent::Mouse(_, ref mut pos, _) = event {
                *pos = Some(Position::new(
                    me.offset_x() as usize,
                    me.offset_y() as usize,
                ));
            }
        });
        let window = window();
        add_event_mut(&window, "mouseup", &self.event, |event, _| {
            *event = AppEvent::None;
        });
        add_event_mut(&window, "resize", &self.event, |event, _| {
            *event = AppEvent::Resize;
        });
        add_event_mut(&get_el("width"), "input", &self.event, |event, _| {
            *event = AppEvent::ResizeGrid;
        });
        add_event_mut(&get_el("height"), "input", &self.event, |event, _| {
            *event = AppEvent::ResizeGrid;
        });
    }
    pub fn start(mut self) {
        loop_animation_frame(
            move |_| {
                let mut reset = true;
                let ev = self.event.clone();
                let mut event = self.event.borrow_mut();
                match &*event {
                    AppEvent::Mouse(old_pos, new_pos, fill) => {
                        let (row, col) = self.renderer.get_indices(old_pos.x, old_pos.y);
                        let old_cell = self.grid.get(col, row);
                        let drag = if old_cell == Cell::Start || old_cell == Cell::End {
                            true
                        } else {
                            self.grid.set(row, col, *fill);
                            false
                        };
                        if let Some(n_p) = new_pos {
                            let old_i = Position::new(row, col);
                            let (row, col) = self.renderer.get_indices(n_p.x, n_p.y);
                            let new_i = Position::new(row, col);
                            let (start, target) = self.graph.end_points();
                            if old_i != new_i && new_i != start && new_i != target {
                                let new_cell = self.grid.get(col, row);
                                if drag {
                                    self.grid.set(old_i.x, old_i.y, Cell::Path);
                                } else {
                                    self.grid.draw_line(new_i, old_i, *fill);
                                }
                                self.grid.set(new_i.x, new_i.y, old_cell);
                                if old_cell == Cell::Start {
                                    self.graph.set_start(new_i);
                                } else if old_cell == Cell::End {
                                    self.graph.set_target(new_i)
                                }
                                let new_cell = if drag { new_cell } else { *fill };
                                *event = AppEvent::Mouse(*n_p, Some(*n_p), new_cell);
                            }
                        } else {
                        }
                        reset = false;
                    }
                    AppEvent::Resize => {
                        self.renderer.resize(&self.canvas, &self.grid);
                    }
                    AppEvent::ResizeGrid => {
                        if let Ok(width) = get_value("width").parse() {
                            if let Ok(height) = get_value("height").parse() {
                                self.grid.resize(width, height);
                                let (start, target) =
                                    self.grid.set_rand_start_n_end(&|| Math::random());
                                self.graph.set_start(start);
                                self.graph.set_target(target);
                                self.renderer.resize(&self.canvas, &self.grid);
                            }
                        }
                    }
                    AppEvent::Play | AppEvent::Step => {
                        if self.solved {
                            self.grid.clear(false);
                            self.solved = false;
                        }
                        self.graph.step(&mut self.grid);
                        if self.graph.solved() {
                            let mut result = self.response.borrow_mut();
                            result.path = self.graph.trace();
                            self.graph.clear();
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
                        self.solved = true;
                        let result = self.response.borrow();
                        for each in result.path.iter() {
                            if self.graph.not_start_nor_end(*each) {
                                self.grid.set(each.x, each.y, Cell::ShortestPath);
                            }
                            get_el("time").set_inner_html(&format!("{} ms", result.time));
                        }
                    }
                    AppEvent::TraceResponse => {
                        self.solved = true;
                        let result = self.response.borrow();
                        for each in &result.open {
                            if self.graph.not_start_nor_end(*each) {
                                self.grid.set(each.x, each.y, Cell::Visiting);
                            }
                        }
                        for each in &result.closed {
                            if self.graph.not_start_nor_end(*each) {
                                self.grid.set(each.x, each.y, Cell::Visited);
                            }
                        }
                        *event = AppEvent::Trace;
                        reset = false;
                    }
                    AppEvent::Diagonal(diag) => {
                        self.graph.diagonal = *diag;
                    }
                    AppEvent::Multithreaded(multi) => {
                        self.multithreaded = *multi;
                    }
                    AppEvent::Bidirectional(bidir) => {
                        self.graph.set_bidirectional(*bidir);
                    }
                    AppEvent::Solve => {
                        let mut blocked = Vec::new();
                        let (width, height) = (self.grid.width, self.grid.height);
                        for (i, each) in self.grid.iter().enumerate() {
                            let row = i / width;
                            let column = i % width;
                            if *each == Cell::Block {
                                blocked.push(Position::new(column, row));
                            }
                        }
                        self.grid.clear(false);
                        let (start, target) = self.graph.end_points();
                        let request = Request {
                            dimension: (width, height),
                            blocked,
                            a_star: AStarConfig {
                                diagonal: self.graph.diagonal,
                                start,
                                target,
                                multithreaded: self.multithreaded,
                                bidirectional: self.graph.bidirectional(),
                            },
                        };
                        let res = self.response.clone();
                        fetch_then(
                            "http:///localhost:8000/".into(),
                            FetchMethod::post(&request),
                            move |response: Response| {
                                if !response.path.is_empty() {
                                    ev.mutate(AppEvent::TraceResponse);
                                }
                                res.mutate(response);
                            },
                        );
                    }
                    AppEvent::Clear(walls) => {
                        self.graph.clear();
                        self.grid.clear(*walls);
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

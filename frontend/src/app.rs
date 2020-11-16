use crate::{
    dom::{
        add_event, add_event_mut, add_style, body, document, event_as_input, fetch_then, for_each,
        get_el, get_target_el, get_value, html_el_from, insert_html_at, loop_animation_frame,
        query_els, window, FetchMethod, HtmlPosition,
    },
    DrawMode, RcCell, Renderer,
};
use a_star_graph::{AStarBidirectional, AStarConfig, Cell, Grid, Position, Request, Response};
use js_sys::Math;
use maud::html;
use std::str::FromStr;
use strum_macros::EnumString;
use wasm_bindgen::JsCast;
use web_sys::{DomStringMap, HtmlCanvasElement, MouseEvent};

#[derive(Clone, PartialEq, Debug, EnumString)]
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
    Clear,
    ClearAll,
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
                    button data-event="Play" { "Play" }
                    button data-event="Step" { "Step" }
                    button data-event="Solve" { "Solve" }
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
                    button data-event="Clear" { "Clear" }
                    button data-event="ClearAll" { "Clear All" }
                }
            }
        };
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
                    input data-event="ResizeGrid" min="8" value="100" type="number" {}
                    span {" x "}
                    input data-event="ResizeGrid" min="8" value="50" type="number" {}
                }
            }
        };
        insert_html_at(&body(), top_bar.into_string().as_str(), HtmlPosition::Start);
        insert_html_at(
            &body(),
            bottom_bar.into_string().as_str(),
            HtmlPosition::End,
        );
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
        let ev = self.event.clone();
        for_each(&query_els("input[data-event]"), move |each| {
            let ev = ev.clone();
            add_event(&each, "input", move |e| {
                let event = AppEvent::from_str(&event_as_input(&e).dataset().get("event").unwrap())
                    .unwrap();
                *ev.borrow_mut() = event;
            });
        });
        let ev = self.event.clone();
        for_each(&query_els("button[data-event]"), move |each| {
            let ev = ev.clone();
            add_event(&each, "click", move |e| {
                let event = AppEvent::from_str(
                    &html_el_from(get_target_el(&e))
                        .dataset()
                        .get("event")
                        .unwrap(),
                )
                .unwrap();
                *ev.borrow_mut() = event;
            });
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
                                if width >= 8 && height >= 8 {
                                    self.grid.resize(width, height);
                                    let (start, target) =
                                        self.grid.set_rand_start_n_end(&|| Math::random());
                                    self.graph.set_start(start);
                                    self.graph.set_target(target);
                                    self.renderer.resize(&self.canvas, &self.grid);
                                }
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
                    AppEvent::Clear => {
                        self.graph.clear();
                        self.grid.clear(false);
                    }
                    AppEvent::ClearAll => {
                        self.graph.clear();
                        self.grid.clear(true);
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

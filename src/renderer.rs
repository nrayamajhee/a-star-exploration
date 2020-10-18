use crate::{
    dom::{add_event, body},
    grid::{Cell, Grid},
};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Path2d};

#[derive(Clone)]
pub struct RendererConfig {
    pub width: usize,
    pub height: usize,
    pub gap: f64,
    pub cell_size: f64,
    pub stroke_width: Option<f64>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum DrawMode {
    Rectangle,
    Circle,
    Point,
}

#[derive(Clone)]
pub struct Renderer {
    ctx: CanvasRenderingContext2d,
    config: RendererConfig,
    colors: HashMap<Cell, (JsValue, JsValue)>, // caching color names so that wasm doesn't create new string
}

impl Renderer {
    pub fn new(canvas: &HtmlCanvasElement, gap: f64, stroke_width: Option<f64>) -> Self {
        add_event(canvas, "contextmenu", |e| {
            e.prevent_default();
        });
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let mut colors = HashMap::new();
        for each in Cell::iter() {
            colors.insert(
                each,
                (
                    JsValue::from(each.fill_color()),
                    JsValue::from(each.stroke_color()),
                ),
            );
        }
        Self {
            ctx,
            config: RendererConfig {
                width: 0,
                height: 0,
                gap,
                cell_size: 0.,
                stroke_width,
            },
            colors,
        }
    }
    pub fn resize(&mut self, canvas: &HtmlCanvasElement, grid: &Grid) {
        let width = body().offset_width();
        let height = body().offset_height() - 40;
        let window_ar = width as f64 / height as f64;
        let grid_ar = grid.width as f64 / grid.height as f64;
        let (width, height, cell_size) = if window_ar > grid_ar {
            let cell_size = (height as f64 - grid.height as f64 * self.config.gap - self.config.gap)
                as f64
                / grid.height as f64;
            let width = (grid.width as f64 * (cell_size + self.config.gap as f64) + self.config.gap)
                as usize;
            canvas.set_width(width as u32);
            canvas.set_height(height as u32);
            (width, height as usize, cell_size)
        } else {
            let cell_size = (width as f64 - grid.width as f64 * self.config.gap - self.config.gap)
                as f64
                / grid.width as f64;
            let height = (grid.height as f64 * (cell_size + self.config.gap as f64)
                + self.config.gap) as usize;
            canvas.set_width(width as u32);
            canvas.set_height(height as u32);
            (width as usize, height, cell_size)
        };
        self.config = RendererConfig {
            width,
            height,
            cell_size,
            ..self.config
        };
    }
    pub fn draw_grid(&self, grid: &Grid, draw_mode: DrawMode) {
        self.ctx
            .clear_rect(0., 0., self.config.width as f64, self.config.height as f64);
        for i in 0..grid.height {
            for j in 0..grid.width {
                let (x, y) = self.get_offset(j, i);
                let cell = grid.get(i, j);
                let d_m = match cell {
                    Cell::ShortestPath => DrawMode::Point,
                    _ => draw_mode,
                };
                self.draw_cell(
                    x as f64,
                    y as f64,
                    self.config.cell_size,
                    self.config.cell_size,
                    cell,
                    d_m,
                );
            }
        }
    }
    pub fn draw_cell(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        cell: Cell,
        draw_mode: DrawMode,
    ) {
        let (fill_color, stroke_color) = self.colors.get(&cell).unwrap();
        self.ctx.set_fill_style(fill_color);
        let circle = Path2d::new().unwrap();
        let r = width / 2.;
        let d_m = if draw_mode == DrawMode::Point {
            circle
                .arc(x + r, y + r, r / 2., 0., std::f64::consts::TAU)
                .unwrap();
            DrawMode::Circle
        } else {
            circle
                .arc(x + r, y + r, r, 0., std::f64::consts::TAU)
                .unwrap();
            draw_mode
        };
        match d_m {
            DrawMode::Rectangle => {
                self.ctx.fill_rect(x, y, width, height);
            }
            _ => {
                self.ctx.fill_with_path_2d(&circle);
            }
        }
        if let Some(stroke_w) = self.config.stroke_width {
            self.ctx.set_line_width(stroke_w);
            self.ctx.set_stroke_style(stroke_color);
            match d_m {
                DrawMode::Rectangle => {
                    self.ctx.stroke_rect(x, y, width, height);
                }
                _ => {
                    self.ctx.stroke_with_path(&circle);
                }
            }
        }
    }
    fn get_offset(&self, row: usize, column: usize) -> (f64, f64) {
        (
            self.config.gap as f64
                + (row as f64 * (self.config.cell_size + self.config.gap as f64)),
            self.config.gap as f64
                + (column as f64 * (self.config.cell_size + self.config.gap as f64)),
        )
    }
    pub fn get_indices(&self, x: usize, y: usize) -> (usize, usize) {
        let calc = |val| {
            let actual_val = val as f64 - self.config.gap;
            let val = actual_val / (self.config.cell_size + self.config.gap);
            val as usize
        };
        (calc(x), calc(y))
    }
}

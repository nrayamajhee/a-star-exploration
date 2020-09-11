use crate::{dom::body, Grid};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub struct Renderer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    grid: Rc<Grid>,
    gap: usize,
}

impl Renderer {
    pub fn new(canvas: HtmlCanvasElement, grid: Rc<Grid>, gap: usize) -> Self {
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        Self {
            canvas,
            ctx,
            grid,
            gap,
        }
    }
}

impl Renderer {
    pub fn resize_canvas(&self) -> (usize, usize, f64) {
        let width = body().offset_width();
        let height = body().offset_height();
        let window_ar = width as f64 / height as f64;
        let grid_ar = self.grid.width as f64 / self.grid.height as f64;
        if window_ar > grid_ar {
            let cell_size = (height as usize - self.grid.height * self.gap - self.gap) as f64
                / self.grid.height as f64;
            let width =
                (self.grid.width as f64 * (cell_size + self.gap as f64)) as usize + self.gap;
            self.canvas.set_width(width as u32);
            self.canvas.set_height(height as u32);
            (width, height as usize, cell_size)
        } else {
            let cell_size = (width as usize - self.grid.width * self.gap - self.gap) as f64
                / self.grid.width as f64;
            let height =
                (self.grid.height as f64 * (cell_size + self.gap as f64)) as usize + self.gap;
            self.canvas.set_width(width as u32);
            self.canvas.set_height(height as u32);
            (width as usize, height, cell_size)
        }
    }
    pub fn draw_grid(&self) {
        let (width, height, cell_size) = self.resize_canvas();
        self.ctx.clear_rect(0., 0., width as f64, height as f64);
        for i in 0..self.grid.height {
            for j in 0..self.grid.width {
                let (x, y) = self.get_offset(j, i, cell_size);
                let cell = self.grid.get(i, j);
                self.draw_cell(
                    x as f64,
                    y as f64,
                    cell_size,
                    cell_size,
                    cell.fill_color(),
                    cell.stroke_color(),
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
        fill_color: &str,
        stroke_color: &str,
    ) {
        self.ctx.set_fill_style(&JsValue::from(fill_color));
        self.ctx.fill_rect(x, y, width, height);
        self.ctx.set_line_width(2.);
        self.ctx.set_stroke_style(&JsValue::from(stroke_color));
        self.ctx.stroke_rect(x, y, width, height);
    }
    fn get_offset(&self, row: usize, column: usize, cell_size: f64) -> (f64, f64) {
        (
            self.gap as f64 + (row as f64 * (cell_size + self.gap as f64)),
            self.gap as f64 + (column as f64 * (cell_size + self.gap as f64)),
        )
    }
}

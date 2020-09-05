use crate::{dom::body, Grid};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub struct Renderer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    grid: Rc<Grid>,
    gap_width: f64,
}

impl Renderer {
    pub fn new(canvas: HtmlCanvasElement, grid: Rc<Grid>) -> Self {
        let gap_width = 2.;
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
            gap_width,
        }
    }
}

impl Renderer {
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
        self.ctx.set_stroke_style(&JsValue::from(stroke_color));
        self.ctx.set_line_width(1.);
        self.ctx.fill_rect(x, y, width, height);
        self.ctx.stroke_rect(x, y, width, height);
    }
    pub fn resize_canvas(&self) {
        let width = body().offset_width();
        let height = body().offset_height();
        let cell_size = if self.grid.width < self.grid.height {
            (width as f64 - self.grid.width as f64 * self.gap_width - self.gap_width)
                / self.grid.width as f64
        } else {
            (height as f64 - self.grid.height as f64 * self.gap_width - self.gap_width)
                / self.grid.height as f64
        };
        let aspect_ratio = self.grid.width / self.grid.height;
        self.canvas.set_width(cell_size * self.grid.width);
        self.canvas.set_height(aspect_ratio * self.grid.height);
    }
    pub fn draw_grid(&self, grid: &Grid, gap_width: f64) {
        let extent = self.resize_canvas();
        let extent = extent as f64;
        self.ctx.clear_rect(0., 0., extent as f64, extent as f64);
        for i in 0..grid.width {
            for j in 0..grid.height {
                let (x, y) = Self::get_offset(i as f64, j as f64, cell_width, cell_height, 2.);
                web_sys::console::log_1(&format!("{} x {} : {} x {}", i, j, x, y).into());
                self.draw_cell(x, y, cell_width, cell_height, "red", "blue");
            }
        }
    }

    pub fn get_dimension(
        width: f64,
        height: f64,
        num_cells_row: usize,
        num_cells_column: usize,
        gap_width: f64,
    ) -> (f64, f64) {
        let dx = (width - num_cells_row as f64 * gap_width - gap_width) / num_cells_row as f64;
        let dy = (height - num_cells_row as f64 * gap_width - gap_width) / num_cells_column as f64;
        (dx, dy)
    }

    fn get_offset(row: f64, column: f64, width: f64, height: f64, gap_width: f64) -> (f64, f64) {
        (
            gap_width + (row * (width + gap_width)),
            gap_width + (column * (height + gap_width)),
        )
    }
}

use crate::{
    dom::{body, RcCell},
    grid::Grid,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[derive(Clone)]
pub struct CanvasInfo {
    width: usize,
    height: usize,
    gap: usize,
    cell_size: f64,
}

#[derive(Clone)]
pub struct Renderer {
    ctx: CanvasRenderingContext2d,
    grid: RcCell<Grid>,
    config: CanvasInfo,
}

impl Renderer {
    pub fn new(canvas: &HtmlCanvasElement, grid: RcCell<Grid>, gap: usize) -> Self {
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        let mut r = Self {
            ctx,
            grid,
            config: CanvasInfo {
                width: 0,
                height: 0,
                gap,
                cell_size: 0.,
            },
        };
        r.resize_canvas(canvas);
        r
    }
}

impl Renderer {
    pub fn resize_canvas(&mut self, canvas: &HtmlCanvasElement) {
        let width = body().offset_width();
        let height = body().offset_height();
        let window_ar = width as f64 / height as f64;
        let grid = self.grid.borrow();
        let grid_ar = grid.width as f64 / grid.height as f64;
        let (width, height, cell_size) = if window_ar > grid_ar {
            let cell_size = (height as usize - grid.height * self.config.gap - self.config.gap)
                as f64
                / grid.height as f64;
            let width = (grid.width as f64 * (cell_size + self.config.gap as f64)) as usize
                + self.config.gap;
            canvas.set_width(width as u32);
            canvas.set_height(height as u32);
            (width, height as usize, cell_size)
        } else {
            let cell_size = (width as usize - grid.width * self.config.gap - self.config.gap)
                as f64
                / grid.width as f64;
            let height = (grid.height as f64 * (cell_size + self.config.gap as f64)) as usize
                + self.config.gap;
            canvas.set_width(width as u32);
            canvas.set_height(height as u32);
            (width as usize, height, cell_size)
        };
        self.config = CanvasInfo {
            width,
            height,
            cell_size,
            ..self.config
        };
    }
    pub fn draw_grid(&self) {
        crate::log!("DRAWING");
        let grid = self.grid.borrow();
        self.ctx
            .clear_rect(0., 0., self.config.width as f64, self.config.height as f64);
        for i in 0..grid.height {
            for j in 0..grid.width {
                let (x, y) = self.get_offset(j, i);
                let cell = grid.get(i, j);
                self.draw_cell(
                    x as f64,
                    y as f64,
                    self.config.cell_size,
                    self.config.cell_size,
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
            let gap = self.config.gap as f64;
            let actual_val = val as f64 - gap;
            crate::log!("BEFORE" val actual_val self.config.cell_size gap);
            let val = (actual_val / (self.config.cell_size + gap));
            crate::log!("AFTEr" val);
            val as usize
        };
        (calc(x), calc(y))
    }
}

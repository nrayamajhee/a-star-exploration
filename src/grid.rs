use wasm_bindgen::JsValue;

pub enum Cell {
    Block,
    Path,
    Start,
    End,
}



impl Cell {
    pub fn fill_color(&self) -> JsValue {
        let color = match self {
            Cell::Block => "#111",
            Cell::Path => "#888",
            Cell::Start => "#128",
            Cell::End => "#821",
        };
        JsValue::from(color)
    }
}

pub struct Grid {
    pub width: usize,
    pub height: usize,
    data: Vec<Cell>,
}

fn grid_new() _> Grid;
fn grid_modify(grid, ) {}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        //let rng = rand::thread_rng();
        let mut data = Vec::new();
        for _ in 0..(width * height) {
            data.push(Cell::Path);
        }
        Self {
            width,
            height,
            data,
        }
    }
}

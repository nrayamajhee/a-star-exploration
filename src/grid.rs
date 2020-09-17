use js_sys::Math;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Block,
    Path,
    Start,
    End,
}

impl From<usize> for Cell {
    fn from(from: usize) -> Self {
        match from {
            0 => Cell::Block,
            1 => Cell::Path,
            2 => Cell::Start,
            _ => Cell::End,
        }
    }
}

impl Cell {
    pub fn fill_color(&self) -> &'static str {
        match self {
            Cell::Block => "#111",
            Cell::Path => "#333",
            Cell::Start => "#ddd",
            Cell::End => "#0a0",
        }
    }
    pub fn stroke_color(&self) -> &'static str {
        match self {
            Cell::Block => "#222",
            Cell::Path => "#444",
            Cell::Start => "#fff",
            Cell::End => "#0b0",
        }
    }
}

#[derive(Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    data: Vec<Cell>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let mut data = Vec::new();
        for _ in 0..(width * height) {
            data.push(Cell::Path);
        }
        let rand = Math::random();
        let len = data.len();
        data[(rand * len as f64) as usize] = Cell::Start;
        let rand = Math::random();
        data[(rand * len as f64) as usize] = Cell::End;
        Self {
            width,
            height,
            data,
        }
    }
    pub fn get(&self, row: usize, column: usize) -> Cell {
        *self
            .data
            .get(row * self.width + column)
            .unwrap_or_else(|| panic!("Couldn't find at {} {}", row, column))
    }
    pub fn set(&mut self, row: usize, column: usize, cell: Cell) {
        let index = row * self.width + column;
        if index >= self.data.len() {
            panic!("Couldn't find at {} {}", row, column);
        } else {
            self.data[index] = cell;
        }
    }
}

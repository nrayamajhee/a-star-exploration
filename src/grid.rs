#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Block,
    Path,
    Start,
    End,
    ShortestPath,
    Visiting,
    Visited,
}

impl Cell {
    pub fn fill_color(&self) -> &'static str {
        match self {
            Cell::Block => "#111",
            Cell::Path => "#333",
            Cell::Start => "#0a0",
            Cell::End => "#aa0",
            Cell::ShortestPath => "#aaa",
            Cell::Visiting => "#03c",
            Cell::Visited => "#a00",
        }
    }
    pub fn stroke_color(&self) -> &'static str {
        match self {
            Cell::Block => "#0a0a0a",
            Cell::Path => "#3e3e3e",
            Cell::Start => "#0c0",
            Cell::End => "#cc0",
            Cell::ShortestPath => "#ccc",
            Cell::Visiting => "#05c",
            Cell::Visited => "#c00",
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
        Self {
            width,
            height,
            data,
        }
    }
    pub fn clear(&mut self) {
        for i in 0..(self.width * self.height) {
            let each = self.data.get_mut(i).unwrap();
            match each {
                Cell::Start | Cell::End => (),
                _ => {
                    *each = Cell::Path;
                }
            }
        }
    }
    pub fn get(&self, column: usize, row: usize) -> Cell {
        self.try_get(row, column)
            .unwrap_or_else(|| panic!("Couldn't find at {} {}", row, column))
    }
    pub fn try_get(&self, column: usize, row: usize) -> Option<Cell> {
        if let Some(data) = self.data.get(row * self.width + column) {
            Some(*data)
        } else {
            None
        }
    }
    pub fn set(&mut self, column: usize, row: usize, cell: Cell) {
        let index = row * self.width + column;
        if index >= self.data.len() {
            panic!("Couldn't find at {} {}", row, column);
        } else {
            self.data[index] = cell;
        }
    }
}

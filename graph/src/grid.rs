use crate::Position;
use std::collections::HashSet;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, EnumIter, Eq, Hash)]
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

#[derive(Clone, Debug)]
pub struct GridSet {
    pub width: usize,
    pub height: usize,
    pub set: HashSet<Position>,
}

impl GridSet {
    pub fn dimension(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

#[derive(Clone, Copy)]
pub enum GridType<'a> {
    Full(&'a Grid),
    Set(&'a GridSet),
}

#[derive(Clone, Debug)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    data: Vec<Cell>,
}

impl std::ops::Deref for Grid {
    type Target = Vec<Cell>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
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
    pub fn clear(&mut self, walls: bool) {
        for i in 0..(self.width * self.height) {
            let each = self.data.get_mut(i).unwrap();
            let clear = match each {
                Cell::Start | Cell::End | Cell::Block => *each == Cell::Block && walls,
                _ => true,
            };
            if clear {
                *each = Cell::Path;
            }
        }
    }
    pub fn dimension(&self) -> (usize, usize) {
        (self.width, self.height)
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
    pub fn resize(&mut self, width: usize, height: usize) {
        let mut new_data = Vec::new();
        for row in 0..height {
            for col in 0..width {
                let mut done = false;
                if let Some(cell) = self.try_get(col, row) {
                    if cell != Cell::Start && cell != Cell::End {
                        new_data.push(cell);
                        done = true;
                    }
                }
                if !done {
                    new_data.push(Cell::Path);
                }
            }
        }
        self.width = width;
        self.height = height;
        self.data = new_data;
    }
    pub fn set_rand_start_n_end(&mut self, rand: &dyn Fn() -> f64) -> (Position, Position) {
        let (w, h) = (self.width, self.height);
        let get_x_y = || {
            let x = (rand() * w as f64) as usize;
            let y = (rand() * h as f64) as usize;
            (x, y)
        };
        let (x, y) = get_x_y();
        let start = Position::new(x, y);
        self.set(x, y, Cell::Start);
        let (x, y) = get_x_y();
        let target = Position::new(x, y);
        self.set(x, y, Cell::End);
        (start, target)
    }
    fn plot_line(&mut self, start: Position, target: Position, cell: Cell, high: bool) {
        let mut dx = target.x as isize - start.x as isize;
        let mut dy = target.y as isize - start.y as isize;
        let xi: isize = if dx < 0 { -1 } else { 1 };
        let yi: isize = if dy < 0 { -1 } else { 1 };
        if high {
            if dx < 0 {
                dx = -dx;
            }
        } else {
            if dy < 0 {
                dy = -dy;
            }
        }
        let mut d = if high { 2 * dx - dy } else { 2 * dy - dx };
        if high {
            let mut x = start.x as isize;
            for y in start.y..target.y {
                if let Some(e_c) = self.try_get(x as usize, y) {
                    if e_c != Cell::Start && e_c != Cell::End {
                        self.set(x as usize, y, cell);
                    }
                }
                if d > 0 {
                    x += xi;
                    d += 2 * (dx - dy);
                } else {
                    d += 2 * dx;
                }
            }
        } else {
            let mut y = start.y as isize;
            for x in start.x..target.x {
                if let Some(e_c) = self.try_get(x, y as usize) {
                    if e_c != Cell::Start && e_c != Cell::End {
                        self.set(x, y as usize, cell);
                    }
                }
                if d > 0 {
                    y += yi;
                    d += 2 * (dy - dx);
                } else {
                    d += 2 * dy;
                }
            }
        }
    }
    pub fn draw_line(&mut self, start: Position, target: Position, cell: Cell) {
        if (target.y as isize - start.y as isize).abs()
            < (target.x as isize - start.x as isize).abs()
        {
            if start.x < target.x {
                self.plot_line(start, target, cell, false);
            } else {
                self.plot_line(target, start, cell, false);
            }
        } else {
            if start.y < target.y {
                self.plot_line(start, target, cell, true);
            } else {
                self.plot_line(target, start, cell, true);
            }
        }
    }
}

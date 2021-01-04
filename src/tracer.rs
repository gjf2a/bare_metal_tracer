use crate::vga_buffer::{BUFFER_WIDTH, BUFFER_HEIGHT, plot, plot_str, plot_num, clear_row, ColorCode, Color};
use pc_keyboard::{KeyCode, DecodedKey};
use crate::{println,serial_println};
use core::ops::Sub;

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
#[repr(u8)]
pub enum Dir {
    N, S, E, W
}

impl Dir {
    fn icon(&self) -> char {
        match self {
            Dir::N => 'v',
            Dir::S => '^',
            Dir::E => '<',
            Dir::W => '>'
        }
    }

    fn reverse(&self) -> Dir {
        match self {
            Dir::N => Dir::S,
            Dir::S => Dir::N,
            Dir::E => Dir::W,
            Dir::W => Dir::E
        }
    }

    fn left(&self) -> Dir {
        match self {
            Dir::N => Dir::W,
            Dir::S => Dir::E,
            Dir::E => Dir::N,
            Dir::W => Dir::S
        }
    }

    fn right(&self) -> Dir {
        match self {
            Dir::N => Dir::E,
            Dir::S => Dir::W,
            Dir::E => Dir::S,
            Dir::W => Dir::N
        }
    }
}

impl From<char> for Dir {
    fn from(icon: char) -> Self {
        match icon {
            '^' => Dir::S,
            'v' => Dir::N,
            '>' => Dir::W,
            '<' => Dir::E,
            _ => panic!("Illegal pacman icon: '{}'", icon)
        }
    }
}

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum Cell {
    Trace(Dir),
    Empty
}

impl Cell {
    fn to_icon(&self) -> char {
        match self {
            Cell::Trace(d) => match d {
                Dir::N => '^',
                Dir::S => 'v',
                Dir::E => '>',
                Dir::W => '<'
            }
            Cell::Empty => ' '
        }
    }
}

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
pub struct Position {
    col: i16, row: i16
}

impl Position {
    pub fn is_legal(&self) -> bool {
        0 <= self.col && self.col < BUFFER_WIDTH as i16 && 0 <= self.row && self.row < BUFFER_HEIGHT as i16
    }

    pub fn row_col(&self) -> (usize, usize) {
        (self.row as usize, self.col as usize)
    }

    pub fn neighbor(&self, d: Dir) -> Position {
        match d {
            Dir::N => Position {row: self.row - 1, col: self.col},
            Dir::S => Position {row: self.row + 1, col: self.col},
            Dir::E => Position {row: self.row,     col: self.col + 1},
            Dir::W => Position {row: self.row,     col: self.col - 1}
        }
    }
}

const UPDATE_FREQUENCY: usize = 1;

#[derive(Debug,Clone,Eq,PartialEq)]
pub struct TracerGame {
    cells: [[Cell; BUFFER_WIDTH]; BUFFER_HEIGHT],
    pos: Position,
    countdown: usize,
    last_key: Option<Dir>
}

impl TracerGame {
    pub fn new() -> Self {
        let mut game = TracerGame {
            cells: [[Cell::Empty; BUFFER_WIDTH]; BUFFER_HEIGHT],
            pos: Position {col: (BUFFER_WIDTH / 2) as i16, row: (BUFFER_HEIGHT / 2) as i16},
            countdown: UPDATE_FREQUENCY, last_key: Some(Dir::E),
        };
        game.update();
        game
    }

    fn cell(&self, p: Position) -> Cell {
        self.cells[p.row as usize][p.col as usize]
    }

    fn draw(&self) {
        let color = ColorCode::new(Color::Cyan, Color::Black);
        for (row, contents) in self.cells.iter().enumerate() {
            for (col, cell) in contents.iter().enumerate() {
                plot(cell.to_icon(), col, row, color);
            }
        }
    }

    fn update(&mut self) {
        if let Some(key) = self.last_key {
            let (row, col) = self.pos.row_col();
            self.cells[row][col] = Cell::Trace(key);
            let future = self.pos.neighbor(key);
            if future.is_legal() {
                self.pos = future;
            }
        }
        self.last_key = None;
    }

    pub fn tick(&mut self) {
        if self.countdown == 0 {
            self.update();
            self.draw();
            self.countdown = UPDATE_FREQUENCY;
        } else {
            self.countdown -= 1;
        }
    }

    pub fn key(&mut self, key: Option<DecodedKey>) {
        let key = key2dir(key);
        if key.is_some() {
            self.last_key = key;
        }
    }
}

fn key2dir(key: Option<DecodedKey>) -> Option<Dir> {
    match key {
        None => None,
        Some(key) => match key {
            DecodedKey::RawKey(k) => match k {
                KeyCode::ArrowUp => Some(Dir::N),
                KeyCode::ArrowDown => Some(Dir::S),
                KeyCode::ArrowLeft => Some(Dir::W),
                KeyCode::ArrowRight => Some(Dir::E),
                _ => None
            }
            DecodedKey::Unicode(c) => match c {
                'w' => Some(Dir::N),
                'a' => Some(Dir::W),
                's' => Some(Dir::S),
                'd' => Some(Dir::E),
                _ => None
            }
        }
    }
}
/*
#[test_case]
fn test_neighbor_dir() {
    let p = Position {col: 4, row: 2};
    for (d, col, row) in [(Dir::N, 4, 1), (Dir::S, 4, 3), (Dir::E, 5, 2), (Dir::W, 3, 2)].iter() {
        assert_eq!(p.neighbor(*d), Position {col: *col, row: *row});
    }
}

*/
#![cfg_attr(not(test), no_std)]

use pluggable_interrupt_os::vga_buffer::{BUFFER_WIDTH, BUFFER_HEIGHT, plot, ColorCode, Color};
use pc_keyboard::{KeyCode, DecodedKey};

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
#[repr(u8)]
pub enum Dir {
    N, S, E, W
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

#[derive(Copy,Debug,Clone,Eq,PartialEq)]
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

    pub fn key(&mut self, key: DecodedKey) {
        let key = key2dir(key);
        if key.is_some() {
            self.last_key = key;
        }
    }
}

fn key2dir(key: DecodedKey) -> Option<Dir> {
    match key {
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
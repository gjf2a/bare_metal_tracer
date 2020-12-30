use crate::vga_buffer::{BUFFER_WIDTH, BUFFER_HEIGHT, plot, ColorCode, Color};
use pc_keyboard::{KeyCode, DecodedKey};
use crate::println;

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
#[repr(u8)]
pub enum Dir {
    N, S, E, W
}

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
#[repr(u8)]
enum Cell {
    Dot,
    Empty,
    Wall,
    PowerDot
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

#[derive(Debug,Clone,Eq,PartialEq)]
pub struct Pacman {
    cells: [[Cell; BUFFER_WIDTH]; BUFFER_HEIGHT],
    pacman: Position,
    pacman_char: char,
    ghosts: [Position; 4],
    dots_eaten: u32
}

const START: &'static str =
    "################################################################################
     #.........A............................................................A.......#
     #.#################.##.##.###.####.#.##############.##.##.##.##.################
     #.#################.##.##.###.####.#.##############.##.##.##.##.################
     #.#################.##.##.###.####.#.##############.##.##.##.##.################
     #.#################.##.##.###.####.#.##############.##.##.##.##.################
     ........O.........................................................O.............
     ###.####.#####.######.####.#.#.#######.#.####.####.#.######.#.####.###.###.##.##
     ###.####.#####.######.####.#.#.#######.#.####.####.#.######.#.####.###.###.##.##
     ###.####.#####.######.####.#.#.#######.#.####.####.#.######.#.####.###.###.##.##
     ###.####.#####.######.####.#.#.#######.#.####.####.#.######.#.####.###.###.##.##
     ###.####.#####.######.####.#.#.#######.#.####.####.#.######.#.####.###.###.##.##
     .......................................<........................................
     #####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.##
     #####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.##
     #####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.##
     #####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.##
     #####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.##
     #####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.##
     .........O...........................................................O..........
     ####.####.####.####.####.####.####.####.####.####.####.####.####.####.####.##.##
     ####.####.####.####.####.####.####.####.####.####.####.####.####.####.####.##.##
     ####.####.####.####.####.####.####.####.####.####.####.####.####.####.####.##.##
     #.........A............................................................A.......#
     ################################################################################";

impl Pacman {
    pub fn new() -> Self {
        let mut cells = [[Cell::Dot; BUFFER_WIDTH]; BUFFER_HEIGHT];
        let mut pacman = Position { col: 0, row: 0};
        let mut pacman_char = '>';
        let mut ghosts = [Position {col: 0, row: 0}; 4];
        let mut ghost = 0;
        for (row, row_chars) in START.split('\n').enumerate() {
            println!("row: {} len: {}", row, row_chars.len());
            for (col, chr) in row_chars.trim().chars().enumerate() {
                match chr {
                    '#' => cells[row][col] = Cell::Wall,
                    '.' => {},
                    'A' => {
                        ghosts[ghost] = Position {row: row as i16, col: col as i16};
                        ghost += 1;
                    },
                    'O' => cells[row][col] = Cell::PowerDot,
                    '>' |'<' | '^' | 'v' => {
                        pacman = Position {row: row as i16, col: col as i16};
                        pacman_char = chr;
                    },
                    _ =>  panic!("Unrecognized character: '{}'", chr)
                }
            }
        }
        assert_eq!(ghost, 4);
        Pacman {cells, pacman, pacman_char, ghosts, dots_eaten: 0}
    }

    fn draw(&self) {
        for (row, contents) in self.cells.iter().enumerate() {
            for (col, cell) in contents.iter().enumerate() {
                let p = Position {col: col as i16, row: row as i16};
                let (c, color) = if p == self.pacman {
                    (self.pacman_char, ColorCode::new(Color::Yellow, Color::Black))
                } else if self.ghosts.contains(&p) {
                    ('A', ColorCode::new(Color::Red, Color::Black))
                } else {
                    match cell {
                        Cell::Dot => ('.', ColorCode::new(Color::White, Color::Black)),
                        Cell::Empty => (' ', ColorCode::new(Color::Black, Color::Black)),
                        Cell::Wall => ('#', ColorCode::new(Color::Blue, Color::Black)),
                        Cell::PowerDot => ('O', ColorCode::new(Color::Green, Color::Black))
                    }
                };
                plot(col, row, c, color);
            }
        }
    }

    fn update_pacman_char(&mut self, dir: Dir) {
        self.pacman_char = match dir {
            Dir::N => 'v',
            Dir::S => '^',
            Dir::E => '<',
            Dir::W => '>'
        }
    }

    pub fn tick(&mut self) {
        self.draw();
    }

    pub fn key(&mut self, key: Option<DecodedKey>) {
        if let Some(dir) = key2dir(key) {
            let neighbor = self.pacman.neighbor(dir);
            if neighbor.is_legal() {
                let (row, col) = neighbor.row_col();
                if self.cells[row][col] != Cell::Wall {
                    self.pacman = neighbor;
                    self.update_pacman_char(dir);
                    match self.cells[row][col] {
                        Cell::Dot | Cell::PowerDot /*for now*/ => {
                            self.dots_eaten += 1;
                            self.cells[row][col] = Cell::Empty;
                        }
                        _ => {}
                    }
                }
            }
        }
        self.draw();
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

#[test_case]
fn test_neighbor_dir() {
    let p = Position {col: 4, row: 2};
    for (d, col, row) in [(Dir::N, 4, 1), (Dir::S, 4, 3), (Dir::E, 5, 2), (Dir::W, 3, 2)].iter() {
        assert_eq!(p.neighbor(*d), Position {col: *col, row: *row});
    }
}
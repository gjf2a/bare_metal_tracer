use crate::vga_buffer::{BUFFER_WIDTH, BUFFER_HEIGHT, plot, ColorCode, Color};
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

impl Sub for Position {
    type Output = Position;

    fn sub(self, rhs: Self) -> Self::Output {
        Position {col: self.col - rhs.col, row: self.row - rhs.row}
    }
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

const UPDATE_FREQUENCY: usize = 3;

#[derive(Debug,Clone,Eq,PartialEq)]
pub struct PacmanGame {
    cells: [[Cell; BUFFER_WIDTH]; BUFFER_HEIGHT],
    pacman: Pacman,
    ghosts: [Position; 4],
    dots_eaten: u32,
    countdown: usize,
    last_key: Option<Dir>
}

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
struct Pacman {
    pos: Position, dir: Dir, open: bool
}

impl Pacman {
    fn new(pos: Position, icon: char) -> Self {
        Pacman {pos, dir: Dir::from(icon), open: true}
    }

    fn tick(&mut self) {
        self.open = !self.open;
    }

    fn icon(&self) -> char {
        if self.open {
            self.dir.icon()
        } else {
            match self.dir {
                Dir::N | Dir::S => '|',
                Dir::E | Dir::W => '-'
            }
        }
    }
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

impl PacmanGame {
    pub fn new() -> Self {
        let mut game = PacmanGame {
            cells: [[Cell::Dot; BUFFER_WIDTH]; BUFFER_HEIGHT],
            pacman: Pacman::new(Position { col: 0, row: 0}, '>'),
            ghosts: [Position {col: 0, row: 0}; 4], dots_eaten: 0, countdown: UPDATE_FREQUENCY, last_key: None};
        let mut ghost = 0;
        for (row, row_chars) in START.split('\n').enumerate() {
            println!("row: {} len: {}", row, row_chars.len());
            for (col, icon) in row_chars.trim().chars().enumerate() {
                game.translate_icon(&mut ghost, row, col, icon);
            }
        }
        assert_eq!(ghost, 4);
        game
    }

    fn translate_icon(&mut self, ghost: &mut usize, row: usize, col: usize, icon: char) {
        match icon {
            '#' => self.cells[row][col] = Cell::Wall,
            '.' => {},
            'A' => {
                self.ghosts[*ghost] = Position {row: row as i16, col: col as i16};
                *ghost += 1;
            },
            'O' => self.cells[row][col] = Cell::PowerDot,
            '>' |'<' | '^' | 'v' => {
                self.pacman = Pacman::new(Position {row: row as i16, col: col as i16}, icon);
            },
            _ =>  panic!("Unrecognized character: '{}'", icon)
        }
    }

    fn cell(&self, p: Position) -> Cell {
        self.cells[p.row as usize][p.col as usize]
    }

    fn draw(&self) {
        for (row, contents) in self.cells.iter().enumerate() {
            for (col, cell) in contents.iter().enumerate() {
                let p = Position {col: col as i16, row: row as i16};
                let (c, color) = self.get_icon_color(p, cell);
                plot(col, row, c, color);
            }
        }
    }

    fn get_icon_color(&self, p: Position, cell: &Cell) -> (char, ColorCode) {
        let (icon, foreground) =
            if p == self.pacman.pos {
                (self.pacman.icon(), Color::Yellow)
            } else if self.ghosts.contains(&p) {
                ('A', Color::Red)
            } else {
                match cell {
                    Cell::Dot => ('.', Color::White),
                    Cell::Empty => (' ', Color::Black),
                    Cell::Wall => ('#', Color::Blue),
                    Cell::PowerDot => ('O', Color::Green)
                }
            };
        (icon, ColorCode::new(foreground, Color::Black))
    }

    fn update(&mut self) {
        self.resolve_move();
        self.last_key = None;
        self.pacman.tick();
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

    fn resolve_move(&mut self) {
        if let Some(dir) = self.last_key {
            let neighbor = self.pacman.pos.neighbor(dir);
            if neighbor.is_legal() {
                let (row, col) = neighbor.row_col();
                if self.cells[row][col] != Cell::Wall {
                    self.pacman.pos = neighbor;
                    self.pacman.dir = dir;
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

#[test_case]
fn first_few_moves() {
    let mut game = PacmanGame::new();
    let tests = [('w', 0, 0, 0), ('a', -1, 0, 1), ('a', -1, 0, 2), ('a', -1, 0, 3),
        ('a', -1, 0, 4), ('s', 0, 1, 5), ('s', 0, 1, 6), ('s', 0, 1, 7), ('s', 0, 1, 8),
        ('s', 0, 1, 9), ('s', 0, 1, 10), ('s', 0, 1, 11), ('s', 0, 0, 11), ('d', 1, 0, 12),
        ('d', 1, 0, 13), ('d', 1, 0, 14), ('d', 1, 0, 15), ('d', 1, 0, 16), ('d', 1, 0, 17),
        ('w', 0, -1, 18), ('w', 0, -1, 19), ('w', 0, -1, 20), ('w', 0, -1, 21), ('w', 0, -1, 22),
        ('w', 0, -1, 23), ('w', 0, -1, 24), ('w', 0, 0, 24)
    ];
    for (key, col_diff, row_diff, score) in tests.iter() {
        let was = game.pacman.pos;
        game.key(Some(DecodedKey::Unicode(*key)));
        game.update();
        let diff = game.pacman.pos - was;
        assert_eq!(diff.col, *col_diff);
        assert_eq!(diff.row, *row_diff);
        assert_eq!(game.dots_eaten, *score);
    }
}

#[test_case]
fn test_icons() {
    let game = PacmanGame::new();
    let tests = [
        (game.pacman.pos, Color::Yellow, '<'),
        (game.pacman.pos.neighbor(Dir::N), Color::Blue, '#'),
        (game.pacman.pos.neighbor(Dir::W), Color::White, '.'),
        (game.ghosts[0], Color::Red, 'A'),
        (Position {row: 6, col: 8}, Color::Green, 'O')
    ];

    for (pos, foreground, icon) in tests.iter() {
        assert_eq!(game.get_icon_color(*pos, &game.cell(*pos)),
                   (*icon, ColorCode::new(*foreground, Color::Black)));
    }
}
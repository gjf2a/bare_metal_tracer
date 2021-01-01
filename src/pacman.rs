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

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
struct Ghost {
    pos: Position, dir: Dir, color: Color
}

impl Ghost {
    fn on_my_left(&self, other: Position) -> bool {
        let offset = self.pos - other;
        match self.dir {
            Dir::N => offset.col > 0,
            Dir::S => offset.col < 0,
            Dir::E => offset.row > 0,
            Dir::W => offset.row < 0
        }
    }

    fn ahead_or_behind(&self, other: Position) -> bool {
        let offset = self.pos - other;
        match self.dir {
            Dir::N | Dir::S => offset.col == 0,
            Dir::E | Dir::W => offset.row == 0
        }
    }

    fn on_my_right(&self, other: Position) -> bool {
        !self.on_my_left(other) && !self.ahead_or_behind(other)
    }

    fn go(&mut self, ahead: Cell, left: Cell, right: Cell, pacman_pos: Position) {
        if left == Cell::Wall && ahead == Cell::Wall && right == Cell::Wall {
            self.dir = self.dir.reverse();
        } else if left != Cell::Wall && (self.on_my_left(pacman_pos) || ahead == Cell::Wall) {
            self.dir = self.dir.left();
        } else if right != Cell::Wall && (self.on_my_right(pacman_pos) || ahead == Cell::Wall) {
            self.dir = self.dir.right();
        }
        self.pos = self.pos.neighbor(self.dir);
    }

    fn icon(&self) -> (char, ColorCode) {
        ('A', ColorCode::new(self.color, Color::Black))
    }
}

const START: &'static str =
    "################################################################################
     #.........A............................................................A.......#
     #.#################.##.##.###.####.#.##############.##.##.##.##.################
     #.#################.##.##.###.####.#.##############.##.##.##.##.################
     #.#################.##.##.###.####.#.##############.##.##.##.##.################
     #.......O.........................................................O............#
     ###.####.#####.######.####.#.#.#######.#.####.####.#.######.#.####.###.###.##.##
     ###.####.#####.######.####.#.#.#######.#.####.####.#.######.#.####.###.###.##.##
     ###.####.#####.######.####.#.#.#######.#.####.####.#.######.#.####.###.###.##.##
     ###.####.#####.######.####.#.#.#######.#.####.####.#.######.#.####.###.###.##.##
     #......................................<.......................................#
     #####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.##
     #####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.##
     #####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.##
     #####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.##
     #####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.##
     #####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.#####.##
     #........O...........................................................O.........#
     ####.####.####.####.####.####.####.####.####.####.####.####.####.####.####.##.##
     ####.####.####.####.####.####.####.####.####.####.####.####.####.####.####.##.##
     ####.####.####.####.####.####.####.####.####.####.####.####.####.####.####.##.##
     #.........A............................................................A.......#
     ################################################################################";

const PACMAN_HEIGHT: usize = BUFFER_HEIGHT - 2;
const HEADER_SPACE: usize = BUFFER_HEIGHT - PACMAN_HEIGHT;

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
enum Status {
    NORMAL, OVER, EMPOWERED
}

#[derive(Debug,Clone,Eq,PartialEq)]
pub struct PacmanGame {
    cells: [[Cell; BUFFER_WIDTH]; PACMAN_HEIGHT],
    pacman: Pacman,
    ghosts: [Ghost; 4],
    status: Status,
    dots_eaten: u32,
    countdown: usize,
    last_key: Option<Dir>
}

const GHOST_STARTS: [(Dir, Color); 4] = [(Dir::E, Color::Red),  (Dir::W, Color::Pink), (Dir::E, Color::LightGreen), (Dir::W, Color::Cyan)];

impl PacmanGame {
    pub fn new() -> Self {
        let mut game = PacmanGame {
            cells: [[Cell::Dot; BUFFER_WIDTH]; PACMAN_HEIGHT],
            pacman: Pacman::new(Position { col: 0, row: 0}, '>'),
            ghosts: [Ghost { pos: Position {col: 0, row: 0}, dir: Dir::N, color: Color::Black }; 4],
            dots_eaten: 0, countdown: UPDATE_FREQUENCY, last_key: None, status: Status::NORMAL};
        game.reset();
        game
    }

    fn reset(&mut self) {
        let mut ghost = 0;
        for (row, row_chars) in START.split('\n').enumerate() {
            for (col, icon) in row_chars.trim().chars().enumerate() {
                self.translate_icon(&mut ghost, row, col, icon);
            }
        }
        assert_eq!(ghost, 4);
        self.status = Status::NORMAL;
        self.dots_eaten = 0;
        self.last_key = None;
    }

    fn translate_icon(&mut self, ghost: &mut usize, row: usize, col: usize, icon: char) {
        match icon {
            '#' => self.cells[row][col] = Cell::Wall,
            '.' => self.cells[row][col] = Cell::Dot,
            'A' => {
                let (dir, color) = GHOST_STARTS[*ghost];
                self.ghosts[*ghost] = Ghost {pos: Position {row: row as i16, col: col as i16}, dir, color};
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
        self.draw_header();
        self.draw_board();
    }

    fn draw_header(&self) {
        match self.status {
            Status::NORMAL => self.draw_normal_header(),
            Status::OVER => self.draw_game_over_header(),
            Status::EMPOWERED => self.draw_empowered_header()
        }
    }

    fn draw_normal_header(&self) {
        clear_row(1, Color::Black);
        let header_color = ColorCode::new(Color::White, Color::Black);
        let score_text = "Score:";
        clear_row(0, Color::Black);
        clear_row(1, Color::Black);
        plot_str(score_text, 0, 0, header_color);
        plot_num(self.dots_eaten as isize, score_text.len() + 1, 0, header_color);
    }

    fn draw_subheader(&self, subheader: &str) {
        plot_str(subheader, 0, 1, ColorCode::new(Color::LightRed, Color::Black));
    }

    fn draw_game_over_header(&self) {
        self.draw_normal_header();
        self.draw_subheader("Game over. Press S to restart.");
    }

    fn draw_empowered_header(&self) {
        self.draw_normal_header();
        self.draw_subheader("Powered up!");
    }

    fn draw_board(&self) {
        for (row, contents) in self.cells.iter().enumerate() {
            for (col, cell) in contents.iter().enumerate() {
                let p = Position {col: col as i16, row: row as i16};
                let (c, color) = self.get_icon_color(p, cell);
                plot(c, col, row + HEADER_SPACE, color);
            }
        }
    }

    fn get_icon_color(&self, p: Position, cell: &Cell) -> (char, ColorCode) {
        let (icon, foreground) =
            if p == self.pacman.pos {
                (match self.status {
                    Status::OVER => '*',
                    _ => self.pacman.icon()
                }, Color::Yellow)
            } else {
                for ghost in self.ghosts.iter() {
                    if ghost.pos == p {
                        return ghost.icon()
                    }
                }
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
        for g in 0..self.ghosts.len() {
            let (ahead, left, right) = self.ahead_left_right(self.ghosts[g].pos, self.ghosts[g].dir);
            self.ghosts[g].go(ahead, left, right, self.pacman.pos);
            if self.ghosts[g].pos == self.pacman.pos {
                self.status = Status::OVER;
            }
        }
    }

    fn ahead_left_right(&self, p: Position, dir: Dir) -> (Cell,Cell,Cell) {
        let ahead = self.cell(p.neighbor(dir));
        let left = self.cell(p.neighbor(dir.left()));
        let right = self.cell(p.neighbor(dir.right()));
        (ahead, left, right)
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
        match self.status {
            Status::OVER => {
                if let Some(decoded) = key {
                    match decoded {
                        DecodedKey::RawKey(KeyCode::S) | DecodedKey::Unicode('s') => self.reset(),
                        _ => {}
                    }
                }
            }
            _ => {
                let key = key2dir(key);
                if key.is_some() {
                    self.last_key = key;
                }
            }
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
        (game.ghosts[0].pos, Color::Red, 'A'),
        (Position {row: 5, col: 8}, Color::Green, 'O')
    ];

    for (pos, foreground, icon) in tests.iter() {
        assert_eq!(game.get_icon_color(*pos, &game.cell(*pos)),
                   (*icon, ColorCode::new(*foreground, Color::Black)));
    }
}

#[test_case]
fn test_ghost_ai() {
    let mut ghost = Ghost {
        pos: Position {col: 10, row: 15},
        dir: Dir::N,
        color: Color::Black
    };
    assert!(ghost.on_my_left(Position {col: 8, row: 15}));
    assert!(ghost.on_my_right(Position {col: 12, row: 15}));
    assert!(!ghost.on_my_right(Position {col: 8, row: 15}));
    assert!(!ghost.on_my_left(Position {col: 12, row: 15}));

    ghost.pos.col = 1;
    ghost.dir = Dir::W;
    ghost.go(Cell::Wall, Cell::Empty, Cell::Wall, Position {col: 2, row: 15});
    assert_eq!(ghost.dir, Dir::S);
    assert_eq!(ghost.pos, Position {col: 1, row: 16});
}

#[test_case]
fn test_above_left_right() {
    let game = PacmanGame::new();
    let tests = [
        (Dir::W, Cell::Wall, Cell::Dot, Cell::Wall),
        (Dir::N, Cell::Wall, Cell::Wall, Cell::Dot),
        (Dir::E, Cell::Dot, Cell::Wall, Cell::Dot),
        (Dir::S, Cell::Dot, Cell::Dot, Cell::Wall)
    ];

    for (dir, ahead, left, right) in tests.iter() {
        let (a, l, r) = game.ahead_left_right(Position {col: 1, row: 1}, *dir);
        assert_eq!(a, *ahead);
        assert_eq!(l, *left);
        assert_eq!(r, *right);
    }
}

#[test_case]
fn test_exit_screen_bug() {
    let mut game = PacmanGame::new();
    game.ghosts[1].pos = Position {row: 1, col: 1};
    game.ghosts[1].dir = Dir::W;
    let (ahead, left, right) = game.ahead_left_right(game.ghosts[1].pos, game.ghosts[1].dir);
    game.ghosts[1].go(ahead, left, right, game.pacman.pos);
    assert_eq!(game.ghosts[1].dir, Dir::S);
    assert_eq!(game.ghosts[1].pos, Position {row: 2, col: 1});
}
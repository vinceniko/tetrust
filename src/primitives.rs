use std::ops::{ Add, AddAssign };
use rand::{thread_rng, Rng};
use rand::distributions::{Distribution, Standard};

use quicksilver::{
    graphics::{Color as QSColor},
    input::{Key}
};

#[derive(Copy, Clone, Debug)]
pub struct Coord {
    pub x: i16,
    pub y: i16
}

impl Default for Coord {
    fn default() -> Self {
        Self{x:0, y:0}
    }
}

impl From<Direction> for Coord {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::Left => Coord{x: -1, y: 0},
            Direction::Right => Coord{x: 1, y: 0},
            Direction::Down => Coord{x: 0, y: 1},
            Direction::None => Coord{x: 0, y: 0},
        }
    }
}

impl Add for Coord {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}

impl AddAssign for Coord {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Coord {
    pub fn coord_to_pos(&self, width: i16) -> Pos {
        Pos (self.x + self.y * width)
    }

    pub fn rand_x_offset(x_range: (i16, i16), y: i16) -> Self {
        let mut rng = thread_rng();
        let i = rng.gen_range(x_range.0, x_range.1);

        Self {
            x: i.into(),
            y: y,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Pos(pub i16); // grid_index refers to the index in the Board grid array

impl Into<usize> for Pos {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl From<usize> for Pos {
    fn from(num: usize) -> Self {
        Self (num as i16)
    }
}

impl Pos {
    pub fn pos_to_coord(&self, width: i16) -> Coord {
        Coord {
            x: self.0 % width as i16,
            y: self.0 / width as i16,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Down,
    Left, 
    Right,
    None
}

impl From<Key> for Direction {
    fn from(key: Key) -> Self {
        match key {
            // move in a direction
            Key::Down => Direction::Down,
            Key::Left => Direction::Left,
            Key::Right => Direction::Right,
            _ => Direction::None
        }
    }
}

impl From<Coord> for Direction {
    fn from(coord: Coord) -> Self {
        match coord {
            Coord{x: 0, y: 1} => Direction::Down,
            Coord{x: -1, y: 0} => Direction::Left,
            Coord{x: 1, y: 0} => Direction::Right,
            _ => Direction::None,
        }
    }
}

impl From<Collision> for Direction {
    fn from(coll: Collision) -> Self {
        coll.into()
    }
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            _ => Direction::None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Rotation {
    CW,
    CCW,
    None,
}

// returns a random rotation to init a random Tetrinone
impl Distribution<Rotation> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Rotation {
        let i: i16 = rng.gen_range(0,3);
        match i {
            0 => Rotation::CW,
            1 => Rotation::CCW,
            _ => Rotation::None,
        }
    }
}

impl From<Key> for Rotation {
    fn from(key: Key) -> Self {
        match key {
            // move in a direction
            Key::Z => Rotation::CCW,
            Key::X => Rotation::CW,
            Key::Up => Rotation::CW,
            _ => Rotation::None
        }
    }
}

impl Rotation {
    pub fn to_dir(&self) -> Direction {
        match self {
            Rotation::CW => Direction::Right,
            Rotation::CCW => Direction::Left,
            _ => Direction::None
        }
    }
}

impl Into<Direction> for Rotation {
    fn into(self) -> Direction {
        match self {
            Rotation::CW => Direction::Right,
            Rotation::CCW => Direction::Left,
            _ => Direction::None
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Collision {
    Left,
    Right,
    Under,
    None,
}

impl Collision {
    pub fn to_dir(&self) -> Direction {
        match self {
            Collision::Left => Direction::Left,
            Collision::Right => Direction::Right,
            Collision::Under => Direction::Down,
            Collision::None => Direction::None,
        }
    }
}

impl From<Direction> for Collision {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::Left => Collision::Left,
            Direction::Right => Collision::Right,
            Direction::Down => Collision::Under,
            Direction::None => Collision::None,
        }
    }
}
    
const NUM_COLORS: usize = 8;

#[derive(Copy, Clone, Debug)]
pub enum Color {
    Black,
    Green,
    Yellow,
    Red,
    Blue,
    Pink,
    White,
    Aqua,
}

const COLORS: [Color; NUM_COLORS] = [Color::Black, Color::Green, Color::Yellow, Color::Red, Color::Blue, Color::Pink, Color::White, Color::Aqua];

impl Color {
    fn to_i(&self) -> usize {
        match self {
            Color::Black => 0,
            Color::Green => 1,
            Color::Yellow => 2,
            Color::Red => 3,
            Color::Blue => 4,
            Color::Pink => 5,
            Color::White => 6,
            Color::Aqua => 7,
        }
    }

    pub fn get_color(i: usize) -> Color {
        COLORS[(i + 1) % NUM_COLORS]
    }

    fn _next_color(i: usize) -> Color {
        Self::get_color(i)
    }

    pub fn next_color(&self) -> Color {
        Color::_next_color(self.to_i())
    }
}

impl Into<QSColor> for Color {
    fn into(self) -> QSColor {
        match self {
            Color::Black => QSColor::from_rgba(0, 0, 0, 1.0),
            Color::Green => QSColor::from_rgba(0, 255, 34, 1.0),
            Color::Yellow => QSColor::from_rgba(255, 255, 0, 1.0),
            Color::Red => QSColor::from_rgba(255, 0, 0, 1.0),
            Color::Blue => QSColor::from_rgba(0, 0, 255, 1.0),
            Color::Pink => QSColor::from_rgba(255, 0, 255, 1.0),
            Color::White => QSColor::from_rgba(255, 255, 255, 1.0),
            Color::Aqua => QSColor::from_rgba(0, 173, 254, 1.0),
        }
    }
}
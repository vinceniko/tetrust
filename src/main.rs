use ggez::{ conf, Context, ContextBuilder, GameResult };
use ggez:: graphics;
use ggez::event::{ self, EventHandler};
use ggez::event::{KeyCode, KeyMods};

use std::ops::{ Add, AddAssign };

use rand::{thread_rng, Rng};

use std::time::{Duration, Instant};

use nalgebra::{Vector2, Matrix2};

#[derive(Copy, Clone, Debug)]
struct Coord {
    x: i16,
    y: i16
}

impl Default for Coord {
    fn default() -> Self {
        Self{x:0, y:0}
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
    fn coord_to_pos(&self, width: i16) -> Pos {
        Pos (self.x + self.y * width)
    }

    fn rand_x_offset(x_range: (i16, i16), y: i16) -> Self {
        let mut rng = thread_rng();
        let i = rng.gen_range(x_range.0, x_range.1);

        Self {
            x: i.into(),
            y: y,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Pos(i16); // grid_index refers to the index in the Board grid array

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
    fn pos_to_coord(&self, width: i16) -> Coord {
        Coord {
            x: self.0 % width as i16,
            y: self.0 / width as i16,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Color {
    Black,
    Green,
    Yellow,
    Red,
    Blue,
    Pink,
    White,
    Aqua,
}

impl Into<graphics::Color> for Color {
    fn into(self) -> graphics::Color {
        match self {
            Color::Black => graphics::Color::from_rgba_u32(0x000000),
            Color::Green => graphics::Color::from_rgb(0, 255, 34),
            Color::Yellow => graphics::Color::from_rgb(255, 255, 0),
            Color::Red => graphics::Color::from_rgb(255, 0, 0),
            Color::Blue => graphics::Color::from_rgb(0, 0, 255),
            Color::Pink => graphics::Color::from_rgb(255, 0, 255),
            Color::White => graphics::Color::from_rgb(255, 255, 255),
            Color::Aqua => graphics::Color::from_rgb(0, 173, 254),

        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Bone {
    color: Color,
    coord: Coord,
}

impl Default for Bone {
    fn default() -> Self {
        Bone::new(Color::Black.into(), Coord{x: 0, y: 0})
    }
}

impl Bone {
    fn new(color: Color, coord: Coord) -> Self {
        Self {
            color,
            coord
        }
    }
}

#[derive(Debug, Clone)]
struct PlayingPiece {
    tetrinome: Tetrinome,
    offset: Coord,
}

impl PlayingPiece {
    fn rand_tetrinome() -> Tetrinome {
        let mut rng = thread_rng();
        let i = rng.gen_range(0, NUM_PIECES) as usize;
        
        unsafe {
            if let Some(pieces) = &PIECES {
                pieces[i].clone()
            } else {
                panic!("piece array not initialized!")
            }
        }
    }

    // new at random x in region: width - 4
    fn new(width: &i16) -> Self {
        Self {
            tetrinome: Self::rand_tetrinome(),
            offset: Coord::rand_x_offset((4, width-4), -1),
        }
    }

    // add offset
    fn shift(&self, offset: Coord) -> Coord {
        self.offset + offset
    }

    // replace offset
    fn shift_to(&mut self, new_offset: Coord) {
        self.offset = new_offset;
    }

    // set new offset based on adding offset
    fn shift_change(&mut self, offset: &Coord) {
        self.shift_to(self.shift(*offset))
    }

    fn get_offset_coords(&self) -> Vec<Coord> {
        self.tetrinome.bones.iter().map(|bone| bone.coord + self.offset ).collect()
    }

    // fn push_from(&mut self, dir: &Direction) {
    //     self.shift_change(&dir.opposite().into());
    // }
}

enum Collision {
    Left,
    Right,
    Under,
    None,
}

// impl Collision {
//     fn to_dir(&self) -> Direction {
//         match self {
//             Collision::Left => Direction::Left,
//             Collision::Right => Direction::Left,
//             Collision::Under => Direction::Down,
//             Collision::None => Direction::None,
//         }
//     }
// }

struct Block {
    color: Color,
    pos: Pos
}

#[derive(Debug, Clone)]
struct Blocks (Vec<Option<Bone>>);

impl Blocks {
    fn set_block(&mut self, new_pos: Pos, bone: Bone) {
        if new_pos.0 >= 0 { // make sure its on the grid
            let i: usize = new_pos.into(); // convert to index type
            self.0[i] = Some(bone);
        }
    }

    fn get_block(&self, pos: Pos) -> Option<Bone> {
        if pos.0 >= 0 { // make sure its on the grid
            let i: usize = pos.into(); // convert to index type
            return self.0[i]
        }
        None
    }

    fn clear(&mut self) {
        self.0 = vec![None; self.0.len()];
    }
}

impl From<Vec<Option<Bone>>> for Blocks {
    fn from(blocks: Vec<Option<Bone>>) -> Self {
        Self (
            blocks
        )
    }
}

#[derive(Debug, Clone)]
struct Grid {
    blocks: Blocks,
    curr_piece: PlayingPiece,
    width: i16,
    height: i16,
    size: i16, // total number of blocks
    block_size: i16
}

impl Grid {
    fn new(width: i16, height: i16, block_size: i16) -> Self {
        Self {
            blocks: vec![None; height as usize * width as usize].into(), // init to None (like null ptr)
            curr_piece: PlayingPiece::new(&width),
            width,
            height,
            size: width * height,
            block_size,
        }
    }

    // commit the piece after a downwards collision 
    fn commit_piece(&mut self) {
        for new_block in self.curr_piece.tetrinome.bones.iter_mut() {
            new_block.coord += self.curr_piece.offset; // add offset for each block
            let new_pos = new_block.coord.coord_to_pos(self.width); // convert into pos and then usize for indexing

            self.blocks.set_block(new_pos, *new_block);
        }
    }

    fn check_row_full(&self, row: &i16) -> bool {
        let start = (row * self.width) as usize;
        let end = start + self.width as usize; 
        for block in self.blocks.0[start..end].iter() {
            if let None = block {
                return false
            }
        }
        true
    }

    fn clear_row(&mut self, row: i16) {
        let start = (row * self.width) as usize;
        let end = start + self.width as usize; 
        for block in self.blocks.0[start..end].iter_mut() {
            *block = None;
        }
    }

    fn get_piece_rows(&self, piece: &PlayingPiece) -> Vec<i16> {
        piece.tetrinome.bones.iter().map(|bone| bone.coord.y).collect()
    }

    fn shift_row_down(&mut self, row: i16) {
        let mut start = (row * self.width) as usize;
        let end = start + self.width as usize; 
        for block in self.blocks.0.clone()[start..end].iter() {
            if let Some(_) = block {
                println!("{:?}", block);
                self.blocks.0[start] = None;
                self.blocks.0[start + self.width as usize] = *block;
            }
            start+=1;
        }
    }

    fn clear_row_if(&mut self) {
        let rows = self.get_piece_rows(&self.curr_piece);

        for row in rows.iter() {
            if self.check_row_full(row) {
                self.clear_row(*row);
                self.shift_row_down(*row-1);
            }
        }
    }

    fn check_collision(&self, piece: &PlayingPiece, dir: &Direction, rot: &Rotation) -> Collision {
        println!("{:?}", piece.offset);

        for coord in piece.get_offset_coords() {
            // out of bounds
            if coord.x < 0 {
                return Collision::Left
            } else if coord.x >= self.width {
                return Collision::Right
            }
            if coord.y >= self.height {
                return Collision::Under
            } else if let None = self.blocks.get_block(coord.coord_to_pos(self.width)) {
                // empty block
            } else {
                return match dir {
                    Direction::Down => Collision::Under,
                    Direction::Left => Collision::Left,
                    Direction::Right => Collision::Right,
                    Direction::None => match rot.to_dir() {
                        Direction::Left => Collision::Left,
                        Direction::Right => Collision::Right,
                        _ => Collision::None,
                    }
                }
            }
        }

        Collision::None
    }

    // move_if is the actually called helper, taking a direction and determining whether or not to move
    fn move_if(&mut self, dir: Direction, rot: Rotation) {
        let mut new_piece = self.curr_piece.clone();
        new_piece.shift_change(&dir.clone().into()); // translate new piece based on direction
        new_piece.tetrinome.rotate(&rot); // do rotation

        let col_dir = self.check_collision(&new_piece, &dir, &rot);
        match col_dir { // check collision for new piece
            Collision::Under => { 
                self.commit_piece(); 
                self.clear_row_if(); 
                self.curr_piece = PlayingPiece::new(&self.width); 
            }, // if collided underneath then commit
            Collision::Left | Collision::Right  => {
                if let Rotation::CCW | Rotation::CW = rot {
                    // self.curr_piece.push_from(&col_dir.to_dir().opposite());
                    // self.curr_piece.tetrinome.rotate(&rot);
                }
            }, // collided on the side, nothing happens
            Collision::None => self.curr_piece = new_piece, // no collision, then move
        }
    }

    fn draw_blocks(&self, ctx: &mut Context, bones: &Vec<Bone>, offset: &Coord) -> GameResult<()> {
        let mesh = &mut graphics::MeshBuilder::new(); // apply new rect meshes to this mesh, faster than drawing each individual rectangle

        for bone in bones.iter() {
            let rect = mesh.rectangle(
                    graphics::DrawMode::fill(), 
                    graphics::Rect::new(
                        ((bone.coord.x + offset.x) * self.block_size).into(),
                        ((bone.coord.y + offset.y) * self.block_size).into(),
                        self.block_size.into(),
                        self.block_size.into(),
                    ), 
                    bone.color.into(),
                ).build(ctx)?;
            graphics::draw(ctx, &rect, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
        }
        Ok(())
    }

    fn draw_grid(&mut self, ctx: &mut Context) -> GameResult<()> {
        let blocks = &self.blocks.0; 
        let bones: Vec<Bone> = blocks.iter().map(|block| { // pull out all bones from Option<Bone>
                if let Some(bone) = block {
                    Ok(*bone)
                } else {
                    Err(())
                }
            }
        )
        .filter_map(Result::ok)
        .collect();
        self.draw_blocks(ctx, &bones, &Coord{x:0, y:0})?;

        Ok(())
    }

    fn draw_curr_piece(&self, ctx: &mut Context) -> GameResult<()> {
        self.draw_blocks(ctx, &self.curr_piece.tetrinome.bones, &self.curr_piece.offset)?;
        Ok(())
    }

    fn update(&mut self) {
        
    }
}

// pixels
#[derive(Debug)]
struct Display {
    width: i16,
    height: i16,
}

impl Display {
    fn new(width: i16, height: i16) -> Self {
        Display {
            width: width,
            height: height
        }
    }
}

#[derive(Debug)]
struct Timing {
    updates_per_sec: u32,
    millis_per_update: u32,
    last_update: Instant,
    fall_update: Instant,
}

impl Timing {
    fn new(updates_per_sec: u32, millis_per_update: u32, last_update: Instant, fall_update: Instant) -> Self {
        Timing {
            updates_per_sec,
            millis_per_update,
            last_update,
            fall_update,
        }
    }
}

#[derive(Debug, Clone)]
struct Tetrinome {
    kind: PieceKind,
    bones: Vec<Bone>,
    pivot: Option<Bone>,
}

impl Tetrinome {
    // from_layout instantiates a new tetrinome using the provided layout
    fn from_layout(layout: String, color: Color, kind: PieceKind) -> Self{
        let width = layout.find('\n').unwrap() as i16 + 1; // width in units not indices
    
        let mut pivot = None;
        
        let mut bones: Vec<Bone> = Vec::new();
        for (i, c) in layout.chars().enumerate() {
            if c == 'x' || c == 'o' {
                let bone = Bone::new(color, Pos::from(i).pos_to_coord(width)); 
                if c == 'o' {
                    pivot = Some(bone);
                }

                bones.push(bone);
            }
        }
        
        Tetrinome {
            bones,
            pivot,
            kind
        }
    }


    fn from_piece(kind: PieceKind) -> Tetrinome {
        match kind {
            PieceKind::I => Tetrinome::from_layout(
                vec![
                    "----",
                    "xoxx",
                    "----",
                    "----",
                ].join("\n"),
                Color::Green,
                kind,
            ),
            PieceKind::L => Tetrinome::from_layout(
                vec![
                    "--x-",
                    "xxo-",
                    "----",
                    "----",
                ].join("\n"),
                Color::Yellow,
                kind,
            ),
            PieceKind::J => Tetrinome::from_layout(
                vec![
                    "x---",
                    "xox-",
                    "----",
                    "----",
                ].join("\n"),
                Color::Red,
                kind,
            ),
            PieceKind::T => Tetrinome::from_layout(
                vec![
                    "--x-",
                    "-xox",
                    "----",
                    "----"
                ].join("\n"),
                Color::Blue,
                kind,
            ),
            PieceKind::Z => Tetrinome::from_layout(
                vec![
                    "xx--",
                    "-ox-",
                    "----",
                    "----",
                ].join("\n"),
                Color::Pink,
                kind,
            ),
            PieceKind::S => Tetrinome::from_layout(
                vec![
                    "--xx",
                    "-xo-",
                    "----",
                    "----",
                ].join("\n"),
                Color::White,
                kind,
            ),
            PieceKind::O => Tetrinome::from_layout(
                vec![
                    "-xx-",
                    "-xx-",
                    "----",
                    "----",
                ].join("\n"),
                Color::Aqua,
                kind,
            ),
        }
    }

    fn rotate(&mut self, rot: &Rotation) {
        if let Some(pivot_bone) = self.pivot { // if the tetrinome has a pivot
            for bone in self.bones.iter_mut() {
                if let Rotation::None = rot { 
                } else { // rotation not nothing
                    let rot_cw_matrix: Matrix2<i16>;
                    if let Rotation::CW = rot {
                        rot_cw_matrix = Matrix2::new(0, -1, 
                                                    1, 0);
                    } else {
                        rot_cw_matrix = Matrix2::new(0, 1, 
                                                    -1, 0);
                    }

                    let coord_vec = Vector2::new(bone.coord.x, bone.coord.y);
                        let pivot_vec = Vector2::new(pivot_bone.coord.x, pivot_bone.coord.y);

                        let pivot_offset = coord_vec - pivot_vec;
                        
                        let new_pivot_offset = rot_cw_matrix * pivot_offset;

                        let new_coord = pivot_vec + new_pivot_offset;

                        bone.coord = Coord{x: new_coord[0], y: new_coord[1]};
                }
            }
        }
    }
}

// number of tetries piece kinds
const NUM_PIECES: usize = 7;

#[derive(Debug, Clone)]
enum PieceKind {
    L,
    J,
    I,
    T,
    Z,
    S,
    O,
}

#[derive(Debug)]
struct Game {
    grid: Grid,
    display: Display,
    timing: Timing,
}

impl Game {
    fn new(_ctx: &mut Context, grid: Grid, display: Display, timing: Timing) -> Self {
        Game {
            grid,
            display,
            timing,
        }
    }
}

impl EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if Instant::now() - self.timing.last_update >= Duration::from_millis(self.timing.millis_per_update.into()) {
            if Instant::now() - self.timing.fall_update >= Duration::from_millis((FALL_RATE).into()) { // gravity
                self.grid.move_if(Direction::Down, Rotation::None);

                let now = Instant::now();
                self.timing.fall_update = now;
                self.timing.last_update = now;
            }

            self.timing.last_update = Instant::now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::Black.into());

        self.grid.draw_curr_piece(ctx)?;
        self.grid.draw_grid(ctx)?;
        
        graphics::present(ctx)?;

        ggez::timer::yield_now();

        Ok(())
    }

    /// key_down_event gets fired when a key gets pressed.
    fn key_down_event (
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        self.grid.move_if(keycode.into(), keycode.into());
        if let KeyCode::C = keycode {
            self.grid.blocks.clear();
        }
    }
}

#[derive(Debug,Clone)]
enum Direction {
    Down,
    Left, 
    Right,
    None
}

impl From<KeyCode> for Direction {
    fn from(key: KeyCode) -> Self {
        match key {
            // move in a direction
            KeyCode::Down => Direction::Down,
            KeyCode::Left => Direction::Left,
            KeyCode::Right => Direction::Right,
            _ => Direction::None
        }
    }
}

impl Into<Coord> for Direction {
    fn into(self) -> Coord {
        match self {
            Direction::Down => Coord{x:0, y:1} ,
            Direction::Left => Coord{x: -1, y: 0},
            Direction::Right => Coord{x: 1, y: 0},
            Direction::None => Coord::default(),
        }
    }
}

// impl Direction {
//     fn opposite(&self) -> Self {
//         match self {
//             Direction::Left => Direction::Right,
//             Direction::Right => Direction::Left,
//             _ => Direction::None,
//         }
//     }
// }

enum Rotation {
    CW,
    CCW,
    None,
}

impl From<KeyCode> for Rotation {
    fn from(key: KeyCode) -> Self {
        match key {
            // move in a direction
            KeyCode::Z => Rotation::CCW,
            KeyCode::X => Rotation::CW,
            _ => Rotation::None
        }
    }
}

impl Rotation {
    fn to_dir(&self) -> Direction {
        match self {
            Rotation::CW => Direction::Right,
            Rotation::CCW => Direction::Left,
            _ => Direction::None
        }
    }
}

const GRID_WIDTH: i16 = 10;
const GRID_HEIGHT: i16 = 20;
const PIXEL_SIZE: i16 = 64;
// Here we're defining how many quickly we want our game to update.
const UPDATES_PER_SEC: u32 = 16;
// And we get the milliseconds of delay that this update rate corresponds to.
const MILLIS_PER_UPDATE: u32 = (1.0 / UPDATES_PER_SEC as f64 * 1000.0) as u32;
const FALL_RATE: u32 = MILLIS_PER_UPDATE * 10;
const DISPLAY_WIDTH: i16 = GRID_WIDTH * PIXEL_SIZE;
const DISPLAY_HEIGHT: i16 = GRID_HEIGHT * PIXEL_SIZE;

static mut PIECES: Option<[Tetrinome; NUM_PIECES]> = None;

fn main() ->GameResult<()> {
    let pieces: [Tetrinome; NUM_PIECES] = [
        Tetrinome::from_piece(PieceKind::L),
        Tetrinome::from_piece(PieceKind::J),
        Tetrinome::from_piece(PieceKind::I),
        Tetrinome::from_piece(PieceKind::T),
        Tetrinome::from_piece(PieceKind::Z),
        Tetrinome::from_piece(PieceKind::S),
        Tetrinome::from_piece(PieceKind::O),
    ];
    unsafe {
        PIECES = Some(pieces);
    }
    
    let grid = Grid::new(GRID_WIDTH, GRID_HEIGHT, PIXEL_SIZE);
    let display = Display::new(DISPLAY_WIDTH, DISPLAY_HEIGHT);
    let timing = Timing::new(UPDATES_PER_SEC, MILLIS_PER_UPDATE, Instant::now(), Instant::now());

    // Make a Context. vsync enabled by default
    let (ctx, events_loop) = &mut ContextBuilder::new("Tetrust", "vinceniko")
        .window_mode(conf::WindowMode::default().dimensions(display.width.into(), display.height.into()))
        .build()?;

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let game = &mut Game::new(ctx, grid, display, timing);

   event::run(ctx, events_loop, game)
}
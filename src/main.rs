#![feature(checked_duration_since)] // non-panic'ing version for instant delay checking
#![feature(duration_float)] // used to determine the number of frames given a frame time and total duration of an animation

use ggez::{ conf, Context, ContextBuilder, GameResult };
use ggez::graphics;
use ggez::event::{ self, EventHandler, KeyCode, KeyMods};

use std::ops::{ Add, AddAssign };

use rand::{thread_rng, Rng};
use rand::distributions::{Distribution, Standard};

use std::time::{Duration, Instant};

use nalgebra::{Vector2, Matrix2};

mod animation;
use animation::{FrameTimer, Animatable, FrameState};

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
            Direction::Down => Coord{x: 0, y: 1} ,
            Direction::Left => Coord{x: -1, y: 0},
            Direction::Right => Coord{x: 1, y: 0},
            Direction::None => Coord::default(),
        }
    }
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            _ => Direction::None,
        }
    }
}

enum Rotation {
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

#[derive(Debug)]
enum Collision {
    Left,
    Right,
    Under,
    None,
}

impl Collision {
    fn to_dir(&self) -> Direction {
        match self {
            Collision::Left => Direction::Left,
            Collision::Right => Direction::Right,
            Collision::Under => Direction::Down,
            Collision::None => Direction::None,
        }
    }
}
    
const NUM_COLORS: usize = 8;

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

    fn get_color(i: usize) -> Color {
        COLORS[(i + 1) % NUM_COLORS]
    }

    fn _next_color(i: usize) -> Color {
        Self::get_color(i)
    }

    fn next_color(&self) -> Color {
        Color::_next_color(self.to_i())
    }
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
struct Block {
    bone: Bone,
    frame_timer: Option<FrameTimer>,
}

impl From<Bone> for Block {
    fn from(some_bone: Bone) -> Self {
        Self {
            bone: some_bone,
            frame_timer: None,
        }
    }
}

impl Animatable for Bone {
    fn animate(&mut self, state: &FrameState) {
        if let FrameState::Ready = state { 
            self.color = self.color.next_color()
        }
    }
}

#[derive(Clone)]
struct Blocks {
    data: Vec<Option<Block>>,
    rows_full: Vec<i16>,
}

impl Blocks {
    fn new(len: usize) -> Self {
        Self {
            data: vec![None; len],
            rows_full: Vec::default(),
        }
    }

    fn set_block(&mut self, new_pos: Pos, bone: Bone) {
        if new_pos.0 >= 0 { // make sure its on the grid
            let i: usize = new_pos.into(); // convert to index type
            self.data[i] = Some(bone.into());
        }
    }

    fn get_block(&self, pos: Pos) -> Option<Block> {
        if pos.0 >= 0 { // make sure its on the grid
            let i: usize = pos.into(); // convert to index type
            return self.data[i].clone()
        }
        None
    }

    // clears the entire grid
    fn clear(&mut self) {
        self.data = vec![None.into(); GRID_SIZE as usize];
    }

    // returns whether the row is full
    fn row_full(&self, row: &i16) -> bool {
        let start = (row * GRID_WIDTH) as usize;
        let end = start + GRID_WIDTH as usize; 
        for some_block in self.data[start..end].iter() {
            if let None = some_block {
                println!("not clear {}", row);
                return false
            }
        }
        true
    }

    // replaces each block in the row with None
    fn clear_row(&mut self, row: &i16) {
        let start = (row * GRID_WIDTH) as usize;
        let end = start + GRID_WIDTH as usize; 
        for some_block in self.data[start..end].iter_mut() {
            if let None = some_block {
            } else {
                *some_block = None;
            }
        }
    }

    fn add_row_to_clear(&mut self, row: &i16) {
        self.rows_full.push(*row);
    }

    // returns whether the row is ready to be cleared if all the animations in the row are done
    fn row_ready(&mut self, row: &i16) -> bool {
        let start = (row * GRID_WIDTH) as usize;
        let end = start + GRID_WIDTH as usize;
        
        self.data[start..end].iter_mut().filter_map(|some_block| {
            if let Some(block) = some_block {
                if let Some(frame_timer) = &mut block.frame_timer {
                    let frame_state = frame_timer.get_state();
                    return Some(frame_state)                    
                }
            }
            return None
        })
        .all(|frame_state| { if let FrameState::Done = frame_state { return true } return false })
    }

    // initializes the FrameTimer which begins the clearing countdown
    fn start_clear(&mut self, row: &i16) {
        let start = (row * GRID_WIDTH) as usize;
        let end = start + GRID_WIDTH as usize;
        
        let mut i = 0;
        for some_block in self.data[start..end].iter_mut() {
            if let Some(block) = some_block {
                if let None = &mut block.frame_timer {
                    let frame_duration = Duration::from_millis(50);
                    let total_anim_time = Duration::from_millis(1000); // to give the perception that the animation doesn't stop before clearing, the blocks on the left have a longer total duration and finish when the blocks on the right finish
                    let n_frames = total_anim_time.as_secs_f64() / frame_duration.as_secs_f64();
                    block.bone.color = Color::get_color(i as usize);
                    block.frame_timer = Some(FrameTimer::equal_sized(n_frames as usize, frame_duration, Duration::default())); // wave effect
                    i += 1;
                }
            }
        }

        self.add_row_to_clear(row);
    }
    
    fn finish_clear(&mut self) {
        let ready_rows: Vec<i16> = self.rows_full.clone().into_iter().filter(|row| self.row_ready(row) ).collect();
        println!("Full Rows: {:?}", self.rows_full);
        println!("Ready Rows: {:?}", ready_rows);

        // clear the ready rows
        for ready_row in ready_rows.iter() {
            self.clear_row(ready_row);
        }
        for ready_row in ready_rows.iter() {
            for upper_row in (0..*ready_row).rev() {
                println!("tried {}", upper_row);
                if self.drop_row_down(&upper_row) == 0 {
                    println!("{}", "returned");
                    break; // preliminary break if empty row found
                }
            }
            self.rows_full.remove(0); // dequeu from front
        }
    }

    // returns the rows the piece inhabits
    fn get_piece_rows(&self, piece: &Tetrinome) -> Vec<i16> {
        let mut ys: Vec<i16> = piece.bones.iter().map(|bone| bone.coord.y).collect();
        ys.sort();
        ys.dedup();
        println!("Piece Rows: {:?}", ys);
        ys.into_iter().collect()
    }

    // drops the given row down
    fn drop_row_down(&mut self, row: &i16) -> i16 {
        let mut start = (row * GRID_WIDTH) as usize;
        let end = start + GRID_WIDTH as usize;
        let mut count = 0;
        for block in self.data.clone()[start..end].iter_mut() {
            if let Some(block) = block {
                block.bone.coord.y += 1; // coord for drawing
                println!("Dropped: {:?}", block);
                self.data[start] = None.into(); // old spot
                self.data[start + GRID_WIDTH as usize] = Some(block.clone()); // new spot has clone
                count+=1;
            }
            start+=1;
        }
        // dropping down the rows affects the rows about to be cleared as well so add to each full row above the cleared row
        for full_row in self.rows_full.iter_mut() {
            if row >= full_row {
                *full_row+=1;
            }
        }
        count
    }

    fn check_collision(&self, piece: &Tetrinome, dir: &Direction, rot: &Rotation) -> Collision {
        for coord in piece.get_coords() {
            // out of bounds
            if coord.x < 0 {
                return Collision::Left
            } else if coord.x >= GRID_WIDTH {
                return Collision::Right
            }
            if coord.y >= GRID_HEIGHT {
                return Collision::Under
            } else if let None = self.get_block(coord.coord_to_pos(GRID_WIDTH)) {
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
}

#[derive(Clone)]
struct Grid {
    blocks: Blocks,
    curr_piece: Tetrinome,
}

impl Grid {
    fn new() -> Self {
        Self {
            blocks: Blocks::new(GRID_WIDTH as usize * GRID_HEIGHT as usize), // init to None (like null ptr)
            curr_piece: Tetrinome::new(&GRID_WIDTH),
        }
    }

    // commit the piece after a downwards collision 
    fn commit_piece(&mut self) {
        for new_block in self.curr_piece.bones.iter_mut() {
            let new_pos = new_block.coord.coord_to_pos(GRID_WIDTH); // convert into pos and then usize for indexing

            self.blocks.set_block(new_pos, *new_block);
        }
    }

    fn clear_row_if(&mut self) {
        let rows = self.blocks.get_piece_rows(&self.curr_piece); // in asc order

        // iterate from top to bottom checking for full rows, once found clear it, and iterate from bottom up to drop blocks down
        for row in 0..=rows[rows.len()-1] {
            if self.blocks.row_full(&row) {
                self.blocks.start_clear(&row);
            }
        }
    }

    // move_if is the actually called helper, taking a direction and determining whether or not to move
    fn move_if(&mut self, dir: Direction, rot: Rotation) {
        let mut new_piece = self.curr_piece.clone();
        new_piece.trans_change(&dir.clone().into()); // translate new piece based on direction
        new_piece.rotate(&rot); // do rotation

        let col_dir = self.blocks.check_collision(&new_piece, &dir, &rot);
        match col_dir { // check collision for new piece
            Collision::Under => { 
                self.commit_piece(); 
                self.clear_row_if(); 
                self.curr_piece = Tetrinome::new(&GRID_WIDTH); 
            }, // if collided underneath then commit
            Collision::Left | Collision::Right  => {
                if let Rotation::CCW | Rotation::CW = rot {
                    let new_dir = &col_dir.to_dir().opposite();
                    println!("widths: {:?}, {:?}", new_piece.get_width(), new_piece.get_width() / 2);
                    for _ in 0..new_piece.get_width()/2 {
                        new_piece.trans_change(&new_dir.clone().into());
                    }
                    let new_col = self.blocks.check_collision(&new_piece, &new_dir, &Rotation::None);
                    if let Collision::None = new_col {
                        println!("{}", true);
                        self.curr_piece = new_piece;
                    } else {
                        println!("{:?}, {:?}", new_dir, new_col);
                    }
                }
            }, // collided on the side, nothing happens
            Collision::None => self.curr_piece = new_piece, // no collision, then move
        }
    }

    fn draw_bones(&self, ctx: &mut Context, bones: &[Bone]) -> GameResult<()> { // bones is a slice of either a vec or an array
        let mesh = &mut graphics::MeshBuilder::new(); // apply new rect meshes to this mesh, faster than drawing each individual rectangle

        for bone in bones.iter() {
            let rect = mesh.rectangle(
                    graphics::DrawMode::fill(), 
                    graphics::Rect::new(
                        ((bone.coord.x) * get_pixel_size()).into(),
                        ((bone.coord.y) * get_pixel_size()).into(),
                        get_pixel_size().into(),
                        get_pixel_size().into(),
                    ), 
                    bone.color.into(),
                ).build(ctx)?;
            graphics::draw(ctx, &rect, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
        }
        Ok(())
    }

    fn draw_grid(&mut self, ctx: &mut Context) -> GameResult<()> {
        let blocks = &mut self.blocks.data; 
        let bones: Vec<Bone> = blocks.iter_mut().filter_map(|block| { // pull out all bones from Option<Bone>
                if let Some(block) = block {
                    if let Some(frame_timer) = &mut block.frame_timer {  // if animatable
                        block.bone.animate(&frame_timer.state());
                    }
                    Some(block.bone)
                } else {
                    None
                }
            }
        )
        .collect();

        self.draw_bones(ctx, &bones)?;

        Ok(())
    }

    fn draw_curr_piece(&self, ctx: &mut Context) -> GameResult<()> {
        self.draw_bones(ctx, &self.curr_piece.bones)?;
        Ok(())
    }
}

#[derive(Debug)]
struct Timing {
    last_update: Instant,
    fall_update: Instant,
}

impl Timing {
    fn new(last_update: Instant, fall_update: Instant) -> Self {
        Timing {
            last_update,
            fall_update,
        }
    }
}

// number of tetries piece kinds
const NUM_PIECES: usize = 7;

const TETRINOME_SIZE: usize = 4;

static mut PIECES: Option<[Tetrinome; NUM_PIECES]> = None;

#[derive(Debug, Clone)]
struct Tetrinome {
    kind: PieceKind,
    bones: [Bone; TETRINOME_SIZE],
    pivot: Option<usize>,
}

impl Tetrinome {
    fn new(width: &i16) -> Self {
        let mut new_piece = rand::random::<Self>();
        new_piece.trans_change(&Coord::rand_x_offset((TETRINOME_SIZE as i16, width-TETRINOME_SIZE as i16), 0)); // translate to random x in the middle of the grid
        new_piece
    }

    // add offset
    fn shift(&self, offset: Coord) -> Vec<Coord> {
        self.bones.iter().map(|bone| bone.coord + offset ).collect()
    }

    // replace offset
    fn trans_to(&mut self, new_coords: Vec<Coord>) {
        self.bones.iter_mut().zip(new_coords).map(|(bone, new_coord)| bone.coord = new_coord ).collect()
    }

    // set new offset based on adding offset
    fn trans_change(&mut self, offset: &Coord) {
        self.trans_to(self.shift(*offset));
    }

    fn get_coords(&self) -> Vec<Coord> {
        self.bones.iter().map(|bone| bone.coord ).collect()
    }

    // from_layout instantiates a new tetrinome using the provided layout
    fn from_layout(layout: String, color: Color, kind: PieceKind) -> Self {
        let width = layout.find('\n').unwrap() as i16 + 1; // width in units not indices
    
        let mut pivot = None;
        
        let mut bones: [Bone; TETRINOME_SIZE] = [Bone::default(); TETRINOME_SIZE];
        let mut bone_i: usize = 0;
        for (i, c) in layout.chars().enumerate() {
            if c == 'x' || c == 'o' {
                let bone = Bone::new(color, Pos::from(i).pos_to_coord(width)); 
                bones[bone_i] = bone;
                
                if c == 'o' {
                    pivot = Some(bone_i);
                }
                bone_i+=1;
            }
        }
        
        Tetrinome {
            bones,
            pivot,
            kind
        }
    }

    fn get_width(&self) -> i16 {
        let xs = self.bones.iter().map(|bone| bone.coord.x );
        xs.clone().max().unwrap() - xs.min().unwrap() + 1 // TODO: is the clone necessary? moved value xs where first clone
    }

    fn from_piece(kind: PieceKind) -> Self {
        match kind {
            PieceKind::I => Tetrinome::from_layout(
                [
                    "----",
                    "xoxx",
                    "----",
                    "----",
                ].join("\n"),
                Color::Green,
                kind,
            ),
            PieceKind::L => Tetrinome::from_layout(
                [
                    "--x-",
                    "xox-",
                    "----",
                    "----",
                ].join("\n"),
                Color::Yellow,
                kind,
            ),
            PieceKind::J => Tetrinome::from_layout(
                [
                    "x---",
                    "xox-",
                    "----",
                    "----",
                ].join("\n"),
                Color::Red,
                kind,
            ),
            PieceKind::T => Tetrinome::from_layout(
                [
                    "--x-",
                    "-xox",
                    "----",
                    "----"
                ].join("\n"),
                Color::Blue,
                kind,
            ),
            PieceKind::Z => Tetrinome::from_layout(
                [
                    "xx--",
                    "-ox-",
                    "----",
                    "----",
                ].join("\n"),
                Color::Pink,
                kind,
            ),
            PieceKind::S => Tetrinome::from_layout(
                [
                    "--xx",
                    "-xo-",
                    "----",
                    "----",
                ].join("\n"),
                Color::White,
                kind,
            ),
            PieceKind::O => Tetrinome::from_layout(
                [
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
        if let Some(pivot_i) = self.pivot { // if the tetrinome has a pivot
            let pivot = self.bones[pivot_i];
            let pivot_vec = Vector2::new(pivot.coord.x, pivot.coord.y);
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

                    let pivot_offset = coord_vec - pivot_vec;
                    
                    let new_pivot_offset = rot_cw_matrix * pivot_offset;

                    let new_coord = pivot_vec + new_pivot_offset;

                    bone.coord = Coord{x: new_coord[0], y: new_coord[1]};
                }
            }
        }
    }
}

// returns a random tetrinome with a random 1 step rotation in either direction but not translated
impl Distribution<Tetrinome> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Tetrinome {
        let i = rng.gen_range(0, NUM_PIECES) as usize;
        
        unsafe {
            if let Some(pieces) = &PIECES {
                let mut new_piece = pieces[i].clone();
                new_piece.rotate(&rand::random::<Rotation>());
                new_piece
            } else {
                panic!("piece array not initialized!")
            }
        }
    }
}

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

struct Game {
    grid: Grid,
    timing: Timing,
}

impl Game {
    fn new(_ctx: &mut Context, grid: Grid, timing: Timing) -> Self {
        Game {
            grid,
            timing,
        }
    }
}

impl EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if Instant::now() - self.timing.last_update >= Duration::from_millis(MILLIS_PER_UPDATE.into()) {
            self.grid.blocks.finish_clear(); // checks whether there are lines to clear

            if Instant::now() - self.timing.fall_update >= Duration::from_millis((FALL_RATE).into()) { // gravity
                self.grid.move_if(Direction::Down, Rotation::None);

                self.timing.fall_update = Instant::now();
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
        if let KeyCode::Q = keycode {
            self.grid.blocks.clear();
        }
    }
}

const GRID_WIDTH: i16 = 10;
const GRID_HEIGHT: i16 = 20;
const GRID_SIZE: i16 = GRID_WIDTH * GRID_HEIGHT;
static mut PIXEL_SIZE: i16 = 0;

// Here we're defining how many quickly we want our game to update.
const UPDATES_PER_SEC: u32 = 16;
// And we get the milliseconds of delay that this update rate corresponds to.
const MILLIS_PER_UPDATE: u32 = (1.0 / UPDATES_PER_SEC as f64 * 1000.0) as u32;
const FALL_RATE: u32 = MILLIS_PER_UPDATE * 10;

fn get_pixel_size() -> i16 {
    unsafe {
        PIXEL_SIZE
    }
}

fn main() ->GameResult<()> {
    let pieces: [Tetrinome; NUM_PIECES] = [
        Tetrinome::from_piece(PieceKind::I),
        Tetrinome::from_piece(PieceKind::O),
        Tetrinome::from_piece(PieceKind::L),
        Tetrinome::from_piece(PieceKind::T),
        Tetrinome::from_piece(PieceKind::Z),
        Tetrinome::from_piece(PieceKind::S),
        Tetrinome::from_piece(PieceKind::J),
    ];
    unsafe {
        PIECES = Some(pieces);
    }
    
    let grid = Grid::new();
    let timing = Timing::new(Instant::now(), Instant::now());

    // determine pixel size based on display height
    unsafe {
        let display_height = event::EventsLoop::new().get_primary_monitor().get_dimensions().height;
        PIXEL_SIZE = (display_height * 0.9) as i16 / GRID_HEIGHT;
    }

    // Make a Context. vsync enabled by default
    let (ctx, events_loop) = &mut ContextBuilder::new("Tetrust", "vinceniko")
        .window_setup(conf::WindowSetup::default().title("Tetrust"))
        .window_mode(conf::WindowMode::default().dimensions((GRID_WIDTH * get_pixel_size()).into(), (GRID_HEIGHT * get_pixel_size()).into()))
        .build()?;

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let game = &mut Game::new(ctx, grid, timing);

    event::run(ctx, events_loop, game)
}
use ggez::{ conf, Context, ContextBuilder, GameResult };
use ggez:: graphics;
use ggez::event::{ self, EventHandler};
use std::ops::{ Add, AddAssign };

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
            y: self.x + other.y
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
    fn coord_to_pos(&self, grid: &Grid) -> Pos {
        Pos (self.x + self.y * grid.width)
    }
}

#[derive(Copy, Clone, Debug)]
struct Pos(i16); // grid_index refers to the index in the Board grid array

impl Pos {
    fn pos_to_coord_grid(&self, grid: &Grid) -> Coord {
        Coord {
            x: self.0 % grid.width as i16,
            y: self.0 / grid.width as i16,
        }
    }

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
}

impl Into<graphics::Color> for Color {
    fn into(self) -> graphics::Color {
        match self {
            Color::Black => graphics::Color::from_rgba_u32(0x000000),
            Color::Green => graphics::Color::from_rgb(0, 255, 34),
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
struct Grid {
    blocks: Vec<Option<Bone>>,
    width: i16,
    height: i16,
    size: i16, // total number of blocks
    block_size: i16,
    offset: Coord,
}

impl Grid {
    fn new(width: i16, height: i16, block_size: i16) -> Self {
        Self {
            blocks: vec![None; height as usize * width as usize], // init to None (like null ptr)
            width,
            height,
            size: width * height,
            block_size,
            offset: Coord{x: 0, y: 0},
        }
    }

    fn commit_piece(&mut self, piece: Tetrinome) {
        let grid = self.clone();
        for mut bone in piece.bones.into_iter() {
            bone.coord += self.offset; // BROKEN
            self.blocks[bone.coord.coord_to_pos(&grid).0 as usize] = Some(bone);
            println!("{:?}", bone);
        }
        self.offset = Coord::default();
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
}

impl Timing {
    fn new(updates_per_sec: u32, millis_per_update: u32) -> Self {
        Timing {
            updates_per_sec,
            millis_per_update
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
        layout.chars().enumerate().for_each(|elem| {
                    if elem.1 == 'x' || elem.1 == 'o' {
                        let bone = Bone::new(color, Pos(elem.0 as i16).pos_to_coord(width)); 
                        if elem.1 == 'o' {
                            pivot = Some(bone);
                        }

                        bones.push(bone);
                    }
                }
            );
        
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
                    "----"
                ].join("\n"),
                Color::Green,
                kind,
            ),
            PieceKind::L => Tetrinome::from_layout(
                vec![
                    "--x-",
                    "xxo-",
                    "----",
                    "----"
                ].join("\n"),
                Color::Green,
                kind,
            ),
            PieceKind::J => Tetrinome::from_layout(
                vec![
                    "----",
                    "x---",
                    "xox-",
                    "----"
                ].join("\n"),
                Color::Green,
                kind,
            ),
            PieceKind::T => Tetrinome::from_layout(
                vec![
                    "--x-",
                    "-xox",
                    "----",
                    "----"
                ].join("\n"),
                Color::Green,
                kind,
            ),
            PieceKind::Z => Tetrinome::from_layout(
                vec![
                    "----",
                    "xx--",
                    "-ox-",
                    "----"
                ].join("\n"),
                Color::Green,
                kind,
            ),
            PieceKind::S => Tetrinome::from_layout(
                vec![
                    "----",
                    "--xx",
                    "-xo-",
                    "----"
                ].join("\n"),
                Color::Green,
                kind,
            ),
            PieceKind::O => Tetrinome::from_layout(
                vec![
                    "----",
                    "-xx-",
                    "-xx-",
                    "----"
                ].join("\n"),
                Color::Green,
                kind,
            ),
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

#[derive(Debug)]
struct Game {
    grid: Grid,
    display: Display,
    timing: Timing,
    pieces: [Tetrinome; 7],
}

impl Game {
    fn new(ctx: &mut Context, grid: Grid, display: Display, timing: Timing, pieces: [Tetrinome; 7]) -> Self {
        Game {
            grid,
            display,
            timing,
            pieces,
        }
    }

    fn draw_grid(&mut self, ctx: &mut Context) -> GameResult<()> {
        let blocks = &self.grid.blocks; 
        for (i, block) in blocks.into_iter().enumerate() {
            let the_bone: Bone;
            if let Some(bone) = block {
                the_bone = *bone;
            } else {
                the_bone = Bone::new(Color::Black.into(), Pos::pos_to_coord_grid(&Pos(i as i16), &self.grid));
            }
            let rect = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(), 
                    graphics::Rect::new(
                        (the_bone.coord.x * self.grid.block_size) as f32,
                        (the_bone.coord.y * self.grid.block_size) as f32,
                        self.grid.block_size as f32,
                        self.grid.block_size as f32,
                    ), 
                    the_bone.color.into(),
                )?;
            graphics::draw(ctx, &rect, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
        }

        Ok(())
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let piece = &self.pieces[3];
        self.grid.commit_piece(piece.clone());
        let piece = &self.pieces[4];
        self.grid.offset += Coord{x: 4, y:4};
        self.grid.commit_piece(piece.clone());

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::Black.into());

        self.draw_grid(ctx)?;
        
        graphics::present(ctx)
    }
}

const GRID_WIDTH: i16 = 10;
const GRID_HEIGHT: i16 = 20;
const PIXEL_SIZE: i16 = 32;
// Here we're defining how many quickly we want our game to update.
const UPDATES_PER_SEC: u32 = 8;
// And we get the milliseconds of delay that this update rate corresponds to.
const MILLIS_PER_UPDATE: u32 = (1.0 / UPDATES_PER_SEC as f64 * 1000.0) as u32;

fn main() ->GameResult<()> {
    let pieces: [Tetrinome; 7] = [
        Tetrinome::from_piece(PieceKind::L),
        Tetrinome::from_piece(PieceKind::J),
        Tetrinome::from_piece(PieceKind::I),
        Tetrinome::from_piece(PieceKind::T),
        Tetrinome::from_piece(PieceKind::Z),
        Tetrinome::from_piece(PieceKind::S),
        Tetrinome::from_piece(PieceKind::O),
    ];

    for bone in &pieces[0].bones {
        println!("{:?}", bone.coord);
    }
    println!("{:?}, {:?}", pieces[0].pivot, pieces[0].kind);    
    println!("{}", "\n");

    for piece in pieces.into_iter() {
        println!("{:?}", piece)
    }

    let grid = Grid::new(GRID_WIDTH, GRID_HEIGHT, PIXEL_SIZE);
    let display = Display::new(GRID_WIDTH * PIXEL_SIZE, GRID_HEIGHT * PIXEL_SIZE);
    let timing = Timing::new(UPDATES_PER_SEC, MILLIS_PER_UPDATE);

    // Make a Context. vsync enabled by default
    let (ctx, events_loop) = &mut ContextBuilder::new("Tetrust", "vinceniko")
        .window_mode(conf::WindowMode::default().dimensions(display.width as f32, display.height as f32))
        .build()?;

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let game = &mut Game::new(ctx, grid, display, timing, pieces);

   event::run(ctx, events_loop, game)
}
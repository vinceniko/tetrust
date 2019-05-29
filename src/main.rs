// use ggez::{ Context, ContextBuilder, GameResult };
use ggez:: graphics;
// use ggez::event::{ self, EventHandler};

#[derive(Copy, Clone, Debug)]
struct Coord {
    x: i16,
    y: i16
}

impl Coord {
    fn coord_to_pos(&self, grid: &Grid) -> Pos {
        Pos (self.x * grid.width + self.y * grid.height)
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

enum Color {
    Black,
    Green,
}

impl Color {
    fn convert(&self) -> graphics::Color {
        match self {
            Color::Black => graphics::Color::from_rgba_u32(0x000000),
            Color::Green => graphics::Color::from_rgba_u32(0x22FF00)
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Bone {
    color: graphics::Color,
    coord: Coord,
}

impl Default for Bone {
    fn default() -> Self {
        Bone::new(Color::Black.convert(), Coord{x: 0, y: 0})
    }
}

impl Bone {
    fn new(color: graphics::Color, coord: Coord) -> Self {
        Bone {
            color,
            coord
        }
    }
}

#[derive(Debug)]
struct Grid {
    blocks: Vec<Option<Bone>>,
    width: i16,
    height: i16,
    size: i16, // total number of blocks
}

impl Grid {
    fn new(width: i16, height: i16) -> Self {
        Grid {
            blocks: vec![None; height as usize * width as usize], // init to None (like null ptr)
            width,
            height,
            size: width * height,
        }
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

#[derive(Debug)]
struct Game {
    grid: Grid,
    display: Display,
    timing: Timing,
}

impl Game {
    fn new(grid: Grid, display: Display, timing: Timing) -> Self {
        Game {
            grid,
            display,
            timing,
        }
    }
}

#[derive(Debug)]
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
                        let bone = Bone::new(color.convert(), Pos(elem.0 as i16).pos_to_coord(width)); 
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

#[derive(Debug)]
enum PieceKind {
    L,
    J,
    I,
    T,
    Z,
    S,
    O,
}

fn main() {
    const GRID_WIDTH: i16 = 10;
    const GRID_HEIGHT: i16 = 20;
    const PIXEL_SIZE: i16 = 32;
    // Here we're defining how many quickly we want our game to update.
    const UPDATES_PER_SEC: u32 = 8;
    // And we get the milliseconds of delay that this update rate corresponds to.
    const MILLIS_PER_UPDATE: u32 = (1.0 / UPDATES_PER_SEC as f64 * 1000.0) as u32;

    let game = Game::new(
        Grid::new(GRID_WIDTH, GRID_HEIGHT),
        Display::new(GRID_WIDTH * PIXEL_SIZE, GRID_HEIGHT * PIXEL_SIZE),
        Timing::new(UPDATES_PER_SEC, MILLIS_PER_UPDATE)
    );

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
}
mod loader;
mod sprites;

use loader::{load_assets, Assets};
use sprites::{blit, Sprite, SpriteRef, Sprites};

/// The screen width is constant (units are in pixels)
pub const SCREEN_WIDTH: usize = 224;
/// The screen height is constant (units are in pixels)
pub const SCREEN_HEIGHT: usize = 256;

// Invader positioning
const START: Point = Point::new(24, 60);
const GRID: Point = Point::new(16, 16);
const ROWS: usize = 5;
const COLS: usize = 11;

#[derive(Debug)]
pub struct World {
    invaders: Invaders,
    lasers: Vec<Laser>,
    shields: Vec<Shield>,
    player: Player,
    cannons: Vec<Cannon>,
    score: u32,
    assets: Assets,
    screen: Vec<u8>,
}

/// A tiny position vector
#[derive(Debug, Default, Eq, PartialEq)]
struct Point {
    x: usize,
    y: usize,
}

/// A formation of invaders.
#[derive(Debug)]
struct Invaders {
    grid: Vec<Vec<Option<Invader>>>,
    stepper: Stepper,
    bounds: Bounds,
}

/// Everything you ever wanted to know about Invaders
#[derive(Debug)]
struct Invader {
    sprite: SpriteRef,
    pos: Point,
    score: u32,
}

/// The stepper will linerly walk through the 2D vector of invaders, updating their state along the
/// way.
#[derive(Debug)]
struct Stepper {
    row: usize,
    col: usize,
}

/// Creates a boundary around the live invaders.
///
/// Used for collision detection and minor optimizations.
#[derive(Debug)]
struct Bounds {
    left: usize,
    right: usize,
    bottom: usize,
}

/// The player entity.
#[derive(Debug)]
struct Player {
    sprite: SpriteRef,
    pos: Point,
}

/// The shield entity.
#[derive(Debug)]
struct Shield {
    // Shield sprite is not referenced because we want to deform it when it gets shot
    sprite: Sprite,
    pos: Point,
}

/// The laser entity.
#[derive(Debug)]
struct Laser {
    sprite: SpriteRef,
    pos: Point,
}

/// The cannon entity.
#[derive(Debug)]
struct Cannon {
    sprite: SpriteRef,
    pos: Point,
}

impl World {
    /// Create a new simple-invaders `World`.
    pub fn new() -> World {
        // Load assets first
        let assets = load_assets();

        let invaders = Invaders {
            grid: make_invader_grid(&assets),
            stepper: Stepper::default(),
            bounds: Bounds::default(),
        };
        let player = Player {
            sprite: SpriteRef::new(&assets, "player1"),
            pos: Point::new(80, 216),
        };
        let shields = (0..4)
            .map(|i| Shield {
                sprite: Sprite::new(&assets, "shield"),
                pos: Point::new(i * 45 + 32, 192),
            })
            .collect();

        // Create a screen with the correct size
        let mut screen = Vec::new();
        screen.resize_with(SCREEN_WIDTH * SCREEN_HEIGHT * 4, Default::default);

        World {
            invaders,
            lasers: Vec::new(),
            shields,
            player,
            cannons: Vec::new(),
            score: 0,
            assets,
            screen,
        }
    }

    /// Update the internal state.
    pub fn update(&mut self) {
        // Find the next invader
        let mut invader = None;
        while let None = invader {
            let (col, row) = self.invaders.stepper.incr();
            invader = self.invaders.grid[row][col].as_mut();
        }
        let invader = invader.unwrap();

        // Animate the invader
        let assets = self.assets.sprites();
        let (pixels, frame) = match invader.sprite.frame().as_ref() {
            "blipjoy1" => (assets.get("blipjoy2").unwrap().2.clone(), "blipjoy2".into()),
            "blipjoy2" => (assets.get("blipjoy1").unwrap().2.clone(), "blipjoy1".into()),
            "ferris1" => (assets.get("ferris2").unwrap().2.clone(), "ferris2".into()),
            "ferris2" => (assets.get("ferris1").unwrap().2.clone(), "ferris1".into()),
            _ => unreachable!(),
        };

        invader.sprite.update_pixels(pixels);
        invader.sprite.update_frame(frame);
    }

    /// Draw the internal state to the screen.
    pub fn draw(&mut self) -> &[u8] {
        // Clear the screen
        self.clear();

        // Draw the invaders
        for row in &self.invaders.grid {
            for col in row {
                if let Some(invader) = col {
                    blit(&mut self.screen, &invader.pos, &invader.sprite);
                }
            }
        }

        &self.screen
    }

    /// Clear the screen
    fn clear(&mut self) {
        for (i, byte) in self.screen.iter_mut().enumerate() {
            *byte = if i % 4 == 3 { 255 } else { 0 };
        }
    }
}

impl Default for World {
    fn default() -> Self {
        World::new()
    }
}

impl Point {
    const fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Mul for Point {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y)
    }
}

impl Stepper {
    fn incr(&mut self) -> (usize, usize) {
        self.col += 1;
        if self.col >= COLS {
            self.col = 0;
            if self.row == 0 {
                self.row = ROWS - 1;
            } else {
                self.row -= 1;
            }
        }

        (self.col, self.row)
    }
}

impl Default for Stepper {
    fn default() -> Self {
        Self {
            row: 0,
            col: COLS - 1,
        }
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            left: START.x,
            right: START.x + COLS * GRID.x,
            bottom: START.y + ROWS * GRID.y,
        }
    }
}

/// Create a grid of invaders.
fn make_invader_grid(assets: &Assets) -> Vec<Vec<Option<Invader>>> {
    const BLIPJOY_OFFSET: Point = Point::new(3, 4);
    const FERRIS_OFFSET: Point = Point::new(3, 5);

    (0..1)
        .map(|y| {
            (0..COLS)
                .map(|x| {
                    Some(Invader {
                        sprite: SpriteRef::new(assets, "blipjoy1"),
                        pos: START + BLIPJOY_OFFSET + Point::new(x, y) * GRID,
                        score: 10,
                    })
                })
                .collect()
        })
        .chain((1..3).map(|y| {
            (0..COLS)
                .map(|x| {
                    Some(Invader {
                        sprite: SpriteRef::new(assets, "ferris1"),
                        pos: START + FERRIS_OFFSET + Point::new(x, y) * GRID,
                        score: 10,
                    })
                })
                .collect()
        }))
        .chain((3..5).map(|y| {
            (0..COLS)
                .map(|x| {
                    Some(Invader {
                        // TODO: Need a third invader
                        sprite: SpriteRef::new(assets, "blipjoy1"),
                        pos: START + BLIPJOY_OFFSET + Point::new(x, y) * GRID,
                        score: 10,
                    })
                })
                .collect()
        }))
        .collect()
}

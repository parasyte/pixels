//! A simple Space Invaders clone to demonstrate `pixels`.
//!
//! This doesn't use anything fancy like a game engine, so you may not want to build a game like
//! this in practice. That said, the game is fully functional, and it should not be too difficult
//! to understand the code.

use std::env;
use std::time::Duration;

pub use controls::{Controls, Direction};
use loader::{load_assets, Assets};
use sprites::{blit, rect, Animation, Frame, Sprite, SpriteRef};

mod controls;
mod loader;
mod sprites;

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
    bullets: Vec<Bullet>,
    score: u32,
    assets: Assets,
    screen: Vec<u8>,
    dt: Duration,
    gameover: bool,
    debug: bool,
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
    direction: Direction,
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
    last_update: usize,
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
struct Bullet {
    sprite: SpriteRef,
    pos: Point,
}

impl World {
    /// Create a new simple-invaders `World`.
    pub fn new() -> World {
        use Frame::*;

        // Load assets first
        let assets = load_assets();

        // TODO: Create invaders one-at-a-time
        let invaders = Invaders {
            grid: make_invader_grid(&assets),
            stepper: Stepper::default(),
            bounds: Bounds::default(),
        };
        let player = Player {
            sprite: SpriteRef::new(&assets, Player1, Duration::from_millis(100)),
            pos: Point::new(80, 216),
            last_update: 0,
        };
        let shields = (0..4)
            .map(|i| Shield {
                sprite: Sprite::new(&assets, Shield1),
                pos: Point::new(i * 45 + 32, 192),
            })
            .collect();

        // Create a screen with the correct size
        let mut screen = Vec::new();
        screen.resize_with(SCREEN_WIDTH * SCREEN_HEIGHT * 4, Default::default);

        // Enable debug mode with `DEBUG=true` environment variable
        let debug = env::var("DEBUG")
            .unwrap_or("false".to_string())
            .parse()
            .unwrap_or(false);

        World {
            invaders,
            lasers: Vec::new(),
            shields,
            player,
            bullets: Vec::new(),
            score: 0,
            assets,
            screen,
            dt: Duration::default(),
            gameover: false,
            debug,
        }
    }

    /// Update the internal state.
    ///
    /// # Arguments
    ///
    /// * `dt`: The time delta since last update.
    /// * `controls`: The player inputs.
    pub fn update(&mut self, dt: Duration, controls: &Controls) {
        if self.gameover {
            // TODO: Add a game over screen
            return;
        }

        let one_frame = Duration::new(0, 16_666_667);

        // Advance the timer by the delta time
        self.dt += dt;

        // Step the invaders one by one
        while self.dt >= one_frame {
            self.dt -= one_frame;
            self.step_invaders();
        }

        // Handle player movement and animation
        self.step_player(controls, dt);

        // TODO: Handle lasers and bullets
        // Movements can be multiplied by the delta-time frame count, instead of looping
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

        // Draw the shields
        for shield in &self.shields {
            blit(&mut self.screen, &shield.pos, &shield.sprite);
        }

        // Draw the player
        blit(&mut self.screen, &self.player.pos, &self.player.sprite);

        if self.debug {
            // Draw invaders bounding box
            let p1 = Point::new(self.invaders.bounds.left, START.y);
            let p2 = Point::new(self.invaders.bounds.right, self.invaders.bounds.bottom);
            let red = [255, 0, 0, 255];
            rect(&mut self.screen, &p1, &p2, &red);
        }

        &self.screen
    }

    fn step_invaders(&mut self) {
        // Find the next invader
        let mut invader = None;
        while let None = invader {
            let (col, row) = self.invaders.stepper.incr();
            invader = self.invaders.grid[row][col].as_mut();
        }
        let invader = invader.unwrap();

        // Move the invader
        let stepper = &self.invaders.stepper;
        let mut bounds = &mut self.invaders.bounds;

        // TODO: Cleanup and remove dupliacte code
        match invader.direction {
            Direction::Left => {
                if bounds.left >= 2 {
                    invader.pos.x -= 2;

                    // Adjust the invaders bounding box
                    // FIXME: This only works if the corner invader is alive
                    if stepper.col == 10 && stepper.row == 0 {
                        bounds.left -= 2;
                        bounds.right -= 2;
                    }
                } else {
                    invader.direction = Direction::Right;
                    invader.pos.x += 2;
                    invader.pos.y += 8;

                    // Adjust the invaders bounding box
                    // FIXME: This only works if the corner invader is alive
                    if stepper.col == 0 && stepper.row == 4 {
                        if invader.pos.y >= self.player.pos.y {
                            self.gameover = true;
                        }

                        bounds.right += 2;
                        bounds.bottom += 8;
                    } else if stepper.col == 10 && stepper.row == 0 {
                        bounds.left += 2;
                    }
                }
            },
            Direction::Right => {
                if bounds.right + 2 <= SCREEN_WIDTH {
                    invader.pos.x += 2;

                    // Adjust the invaders bounding box
                    // FIXME: This only works if the corner invader is alive
                    if stepper.col == 10 && stepper.row == 0 {
                        bounds.left += 2;
                        bounds.right += 2;
                    }
                } else {
                    // TODO: Stop moving down at some point!
                    invader.direction = Direction::Left;
                    invader.pos.x -= 2;
                    invader.pos.y += 8;

                    // Adjust the invaders bounding box
                    // FIXME: This only works if the corner invader is alive
                    if stepper.col == 0 && stepper.row == 4 {
                        // When the lowest invader reaches `player.pos.y`, it's game over!
                        if invader.pos.y >= self.player.pos.y {
                            self.gameover = true;
                        }

                        bounds.left -= 2;
                        bounds.bottom += 8;
                    } else if stepper.col == 10 && stepper.row == 0 {
                        bounds.right -= 2;
                    }
                }
            },
            _ => unreachable!(),
        }

        // Animate the invader
        invader.sprite.step_frame(&self.assets);
    }

    fn step_player(&mut self, controls: &Controls, dt: Duration) {
        match controls.direction {
            Direction::Left => {
                if self.player.pos.x > 0 {
                    self.player.pos.x -= 1;
                    self.player.sprite.animate(&self.assets, dt);
                }
            }

            Direction::Right => {
                if self.player.pos.x < SCREEN_WIDTH - 16 {
                    self.player.pos.x += 1;
                    self.player.sprite.animate(&self.assets, dt);
                }
            }
            _ => (),
        }
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
    use Frame::*;

    const BLIPJOY_OFFSET: Point = Point::new(3, 4);
    const FERRIS_OFFSET: Point = Point::new(3, 5);
    const CTHULHU_OFFSET: Point = Point::new(1, 3);

    (0..1)
        .map(|y| {
            (0..COLS)
                .map(|x| {
                    Some(Invader {
                        sprite: SpriteRef::new(assets, Blipjoy1, Duration::default()),
                        pos: START + BLIPJOY_OFFSET + Point::new(x, y) * GRID,
                        direction: Direction::Right,
                        score: 10,
                    })
                })
                .collect()
        })
        .chain((1..3).map(|y| {
            (0..COLS)
                .map(|x| {
                    Some(Invader {
                        sprite: SpriteRef::new(assets, Ferris1, Duration::default()),
                        pos: START + FERRIS_OFFSET + Point::new(x, y) * GRID,
                        direction: Direction::Right,
                        score: 10,
                    })
                })
                .collect()
        }))
        .chain((3..5).map(|y| {
            (0..COLS)
                .map(|x| {
                    Some(Invader {
                        sprite: SpriteRef::new(assets, Cthulhu1, Duration::default()),
                        pos: START + CTHULHU_OFFSET + Point::new(x, y) * GRID,
                        direction: Direction::Right,
                        score: 10,
                    })
                })
                .collect()
        }))
        .collect()
}

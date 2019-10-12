//! A simple Space Invaders clone to demonstrate `pixels`.
//!
//! This doesn't use anything fancy like a game engine, so you may not want to build a game like
//! this in practice. That said, the game is fully functional, and it should not be too difficult
//! to understand the code.

use std::env;
use std::time::Duration;
use rand_core::{RngCore, OsRng};

pub use controls::{Controls, Direction};
use loader::{load_assets, Assets};
use sprites::{blit, line, Animation, Frame, Sprite, SpriteRef};

mod controls;
mod loader;
mod sprites;

/// The screen width is constant (units are in pixels)
pub const SCREEN_WIDTH: usize = 224;
/// The screen height is constant (units are in pixels)
pub const SCREEN_HEIGHT: usize = 256;

// Invader positioning
const START: Point = Point::new(24, 64);
const GRID: Point = Point::new(16, 16);
const ROWS: usize = 5;
const COLS: usize = 11;

#[derive(Debug)]
pub struct World {
    invaders: Invaders,
    lasers: Vec<Laser>,
    shields: Vec<Shield>,
    player: Player,
    bullet: Option<Bullet>,
    score: u32,
    assets: Assets,
    screen: Vec<u8>,
    dt: Duration,
    gameover: bool,
    random: OsRng,
    debug: bool,
}

/// A tiny position vector.
#[derive(Debug, Default, Eq, PartialEq)]
struct Point {
    x: usize,
    y: usize,
}

/// A fleet of invaders.
#[derive(Debug)]
struct Invaders {
    grid: Vec<Vec<Option<Invader>>>,
    stepper: Point,
    direction: Direction,
    descend: bool,
    bounds: Bounds,
}

/// Everything you ever wanted to know about Invaders.
#[derive(Debug)]
struct Invader {
    sprite: SpriteRef,
    pos: Point,
    score: u32,
}

/// Creates a boundary around the live invaders.
///
/// Used for collision detection and minor optimizations.
#[derive(Debug)]
struct Bounds {
    left_col: usize,
    right_col: usize,
    px: usize,
}

/// The player entity.
#[derive(Debug)]
struct Player {
    sprite: SpriteRef,
    pos: Point,
    dt: usize,
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
    dt: usize,
}

/// The cannon entity.
#[derive(Debug)]
struct Bullet {
    sprite: SpriteRef,
    pos: Point,
    dt: usize,
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
            stepper: Point::new(COLS - 1, 0),
            direction: Direction::Right,
            descend: false,
            bounds: Bounds::default(),
        };
        let player = Player {
            sprite: SpriteRef::new(&assets, Player1, Duration::from_millis(100)),
            pos: Point::new(80, 216),
            dt: 0,
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
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        World {
            invaders,
            lasers: Vec::new(),
            shields,
            player,
            bullet: None,
            score: 0,
            assets,
            screen,
            dt: Duration::default(),
            gameover: false,
            random: OsRng,
            debug,
        }
    }

    /// Update the internal state.
    ///
    /// # Arguments
    ///
    /// * `dt`: The time delta since last update.
    /// * `controls`: The player inputs.
    pub fn update(&mut self, dt: &Duration, controls: &Controls) {
        if self.gameover {
            // TODO: Add a game over screen
            return;
        }

        let one_frame = Duration::new(0, 16_666_667);

        // Advance the timer by the delta time
        self.dt += *dt;

        // Step the invaders one by one
        while self.dt >= one_frame {
            self.dt -= one_frame;
            self.step_invaders();
        }

        // Handle player movement and animation
        self.step_player(controls, dt);

        // Handle bullet movement
        if let Some(bullet) = &mut self.bullet {
            let velocity = update_dt(&mut bullet.dt, dt) * 4;

            if bullet.pos.y > velocity {
                bullet.pos.y -= velocity;
                bullet.sprite.animate(&self.assets, dt);
            } else {
                self.bullet = None;
            }
        }

        // Handle laser movement
        let mut destroy = Vec::new();
        for (i, laser) in self.lasers.iter_mut().enumerate() {
            let velocity = update_dt(&mut laser.dt, dt) * 2;

            if laser.pos.y < self.player.pos.y {
                laser.pos.y += velocity;
                laser.sprite.animate(&self.assets, dt);
            } else {
                destroy.push(i);
            }
        }

        // Destroy dead lasers
        for i in destroy.iter().rev() {
            self.lasers.remove(*i);
        }
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

        // Draw the bullet
        if let Some(bullet) = &self.bullet {
            blit(&mut self.screen, &bullet.pos, &bullet.sprite);
        }

        // Draw lasers
        for laser in self.lasers.iter() {
            blit(&mut self.screen, &laser.pos, &laser.sprite);
        }

        if self.debug {
            // Draw invaders bounding box
            let (left, right) = self.invaders.get_bounds();
            let red = [255, 0, 0, 255];

            let p1 = Point::new(left, START.y);
            let p2 = Point::new(left, self.player.pos.y);
            line(&mut self.screen, &p1, &p2, red);

            let p1 = Point::new(right, START.y);
            let p2 = Point::new(right, self.player.pos.y);
            line(&mut self.screen, &p1, &p2, red);
        }

        &self.screen
    }

    fn step_invaders(&mut self) {
        let (left, right) = self.invaders.get_bounds();
        let (invader, is_leader) =
            next_invader(&mut self.invaders.grid, &mut self.invaders.stepper);

        // The leader controls the fleet
        if is_leader {
            // The leader first commands the fleet to stop descending
            self.invaders.descend = false;

            // Then the leader redirects the fleet when they reach the boundaries
            match self.invaders.direction {
                Direction::Left => {
                    if left < 2 {
                        self.invaders.bounds.px += 2;
                        self.invaders.descend = true;
                        self.invaders.direction = Direction::Right;
                    } else {
                        self.invaders.bounds.px -= 2;
                    }
                }
                Direction::Right => {
                    if right > SCREEN_WIDTH - 2 {
                        self.invaders.bounds.px -= 2;
                        self.invaders.descend = true;
                        self.invaders.direction = Direction::Left;
                    } else {
                        self.invaders.bounds.px += 2;
                    }
                }
                _ => unreachable!(),
            }
        }

        // Every invader in the fleet moves 2px per frame
        match self.invaders.direction {
            Direction::Left => invader.pos.x -= 2,
            Direction::Right => invader.pos.x += 2,
            _ => unreachable!(),
        }

        // And they descend 8px on command
        if self.invaders.descend {
            invader.pos.y += 8;

            // One of the end scenarios
            if invader.pos.y + 8 >= self.player.pos.y {
                self.gameover = true;
            }
        }

        // Animate the invader
        invader.sprite.step_frame(&self.assets);

        // They also shoot lasers at random with a 1:50 chance
        let r = self.random.next_u32() as usize;
        let chance = r % 50;
        if self.lasers.len() < 3 && chance == 0 {
            // Pick a random column to begin searching for an invader that can fire a laser
            let col = r / 50 % COLS;
            let invader = self.invaders.get_closest_invader(col);

            let laser = Laser {
                sprite: SpriteRef::new(&self.assets, Frame::Laser1, Duration::from_millis(16)),
                pos: Point::new(invader.pos.x + 4, invader.pos.y + 10),
                dt: 0,
            };
            self.lasers.push(laser);
        }
    }

    fn step_player(&mut self, controls: &Controls, dt: &Duration) {
        let frames = update_dt(&mut self.player.dt, dt);

        match controls.direction {
            Direction::Left => {
                if self.player.pos.x >= frames {
                    self.player.pos.x -= frames;
                    self.player.sprite.animate(&self.assets, dt);
                }
            }

            Direction::Right => {
                if self.player.pos.x < SCREEN_WIDTH - 15 - frames {
                    self.player.pos.x += frames;
                    self.player.sprite.animate(&self.assets, dt);
                }
            }
            _ => (),
        }

        if controls.fire && self.bullet.is_none() {
            self.bullet = Some(Bullet {
                sprite: SpriteRef::new(&self.assets, Frame::Bullet1, Duration::from_millis(32)),
                pos: Point::new(self.player.pos.x + 7, self.player.pos.y),
                dt: 0,
            });
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

impl Invaders {
    fn get_bounds(&self) -> (usize, usize) {
        let width = (self.bounds.right_col - self.bounds.left_col + 1) * GRID.x;

        let left = self.bounds.px;
        let right = left + width;

        (left, right)
    }

    fn get_closest_invader(&self, mut col: usize) -> &Invader {
        let mut row = ROWS - 1;
        loop {
            if self.grid[row][col].is_some() {
                return self.grid[row][col].as_ref().unwrap();
            }

            if row == 0 {
                row = ROWS - 1;
                col += 1;
                if col == COLS {
                    col = 0;
                }
            } else {
                row -= 1;
            }
        }
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            left_col: 0,
            right_col: COLS - 1,
            px: START.x,
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
                        score: 10,
                    })
                })
                .collect()
        }))
        .collect()
}

fn next_invader<'a>(
    invaders: &'a mut Vec<Vec<Option<Invader>>>,
    stepper: &mut Point,
) -> (&'a mut Invader, bool) {
    let mut is_leader = false;

    loop {
        // Iterate through the entire grid
        stepper.x += 1;
        if stepper.x >= COLS {
            stepper.x = 0;
            if stepper.y == 0 {
                stepper.y = ROWS - 1;

                // After a full cycle, the next invader will be the leader
                is_leader = true;
            } else {
                stepper.y -= 1;
            }
        }

        if invaders[stepper.y][stepper.x].is_some() {
            return (invaders[stepper.y][stepper.x].as_mut().unwrap(), is_leader);
        }
    }
}

fn update_dt(dest_dt: &mut usize, dt: &Duration) -> usize {
    *dest_dt += dt.subsec_nanos() as usize;
    let frames = *dest_dt / 16_666_667;
    *dest_dt -= frames * 16_666_667;

    frames
}

//! A simple Space Invaders clone to demonstrate `pixels`.
//!
//! This doesn't use anything fancy like a game engine, so you may not want to build a game like
//! this in practice. That said, the game is fully functional, and it should not be too difficult
//! to understand the code.

#![no_std]
#![deny(clippy::all)]
#![forbid(unsafe_code)]

extern crate alloc;
use alloc::vec::Vec;

use crate::collision::Collision;
pub use crate::controls::{Controls, Direction};
use crate::geo::Point;
use crate::loader::{load_assets, Assets};
use crate::player::Player;
use crate::shield::Shield;
use crate::sprites::{blit, Animation, Drawable, Frame, SpriteRef};
use core::time::Duration;
use randomize::PCG32;

mod collision;
mod controls;
mod debug;
mod geo;
mod loader;
mod player;
mod shield;
mod sprites;

/// The screen width is constant (units are in pixels)
pub const WIDTH: usize = 224;
/// The screen height is constant (units are in pixels)
pub const HEIGHT: usize = 256;

// Fixed time step (240 fps)
pub const FPS: usize = 240;
pub const TIME_STEP: Duration = Duration::from_nanos(1_000_000_000 / FPS as u64);
// Internally, the game advances at 60 fps
const ONE_FRAME: Duration = Duration::from_nanos(1_000_000_000 / 60);

// Invader positioning
const START: Point = Point::new(24, 64);
const GRID: Point = Point::new(16, 16);
const ROWS: usize = 5;
const COLS: usize = 11;

// Player positioning
const PLAYER_START: Point = Point::new(80, 216);

// Projectile positioning
const LASER_OFFSET: Point = Point::new(4, 10);
const BULLET_OFFSET: Point = Point::new(7, 0);

#[derive(Debug)]
pub struct World {
    invaders: Invaders,
    lasers: Vec<Laser>,
    shields: Vec<Shield>,
    player: Player,
    bullet: Option<Bullet>,
    collision: Collision,
    _score: u32,
    assets: Assets,
    dt: Duration,
    gameover: bool,
    prng: PCG32,
    debug: bool,
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
    _score: u32,
}

/// Creates a boundary around the live invaders.
///
/// Used for collision detection and minor optimizations.
#[derive(Debug)]
struct Bounds {
    pos: Point,
    left_col: usize,
    right_col: usize,
    top_row: usize,
    bottom_row: usize,
}

/// The laser entity.
#[derive(Debug)]
struct Laser {
    sprite: SpriteRef,
    pos: Point,
    dt: Duration,
}

/// The cannon entity.
#[derive(Debug)]
struct Bullet {
    sprite: SpriteRef,
    pos: Point,
    dt: Duration,
}

trait DeltaTime {
    fn update(&mut self) -> usize;

    fn update_dt(dest_dt: &mut Duration, step: Duration) -> usize {
        *dest_dt += TIME_STEP;
        let frames = dest_dt.as_nanos() / step.as_nanos();
        *dest_dt -= Duration::from_nanos((frames * step.as_nanos()) as u64);

        frames as usize
    }
}

impl DeltaTime for Player {
    fn update(&mut self) -> usize {
        Self::update_dt(&mut self.dt, ONE_FRAME)
    }
}

impl DeltaTime for Laser {
    fn update(&mut self) -> usize {
        Self::update_dt(&mut self.dt, ONE_FRAME)
    }
}

impl DeltaTime for Bullet {
    fn update(&mut self) -> usize {
        Self::update_dt(&mut self.dt, TIME_STEP)
    }
}

impl World {
    /// Create a new simple-invaders `World`.
    ///
    /// # Arguments
    ///
    /// * `debug` - Enable debug visualizations.
    /// * `seed` - Inputs for the pseudorandom number generator.
    ///
    /// # Example
    ///
    /// ```
    /// use byteorder::{ByteOrder, NativeEndian};
    /// use getrandom::getrandom;
    /// use simple_invaders::World;
    ///
    /// // Create a seed for the PRNG
    /// let mut seed = [0_u8; 16];
    /// getrandom(&mut seed).expect("failed to getrandom");
    /// let seed = (
    ///     NativeEndian::read_u64(&seed[0..8]),
    ///     NativeEndian::read_u64(&seed[8..16]),
    /// );
    ///
    /// let world = World::new(seed, false);
    /// ```
    pub fn new(seed: (u64, u64), debug: bool) -> World {
        // Load assets first
        let assets = load_assets();

        // TODO: Create invaders one-at-a-time
        let invaders = Invaders::new(&assets);
        let lasers = Vec::new();
        let shields = (0..4)
            .map(|i| Shield::new(&assets, Point::new(i * 45 + 32, 192)))
            .collect();
        let player = Player::new(&assets);
        let bullet = None;
        let collision = Collision::default();
        let _score = 0;

        let dt = Duration::default();
        let gameover = false;
        let prng = PCG32::seed(seed.0, seed.1);

        World {
            invaders,
            lasers,
            shields,
            player,
            bullet,
            collision,
            _score,
            assets,
            dt,
            gameover,
            prng,
            debug,
        }
    }

    /// Update the internal state.
    ///
    /// # Arguments
    ///
    /// * `dt`: The time delta since last update.
    /// * `controls`: The player inputs.
    pub fn update(&mut self, controls: &Controls) {
        if self.gameover {
            // TODO: Add a game over screen
            return;
        }

        // Advance the timer by the delta time
        self.dt += TIME_STEP;

        // Clear the collision details
        self.collision.clear();

        // Step the invaders one by one
        while self.dt >= ONE_FRAME {
            self.dt -= ONE_FRAME;
            self.step_invaders();
        }

        // Handle player movement and animation
        self.step_player(controls);

        if let Some(bullet) = &mut self.bullet {
            // Handle bullet movement
            let velocity = bullet.update();

            if bullet.pos.y > velocity {
                bullet.pos.y -= velocity;
                bullet.sprite.animate(&self.assets);

                // Handle collisions
                if self
                    .collision
                    .bullet_to_invader(&mut self.bullet, &mut self.invaders)
                {
                    // One of the end scenarios
                    self.gameover = self.invaders.shrink_bounds();
                } else {
                    self.collision
                        .bullet_to_shield(&mut self.bullet, &mut self.shields);
                }
            } else {
                self.bullet = None;
            }
        }

        // Handle laser movement
        let mut destroy = Vec::new();
        for (i, laser) in self.lasers.iter_mut().enumerate() {
            let velocity = laser.update() * 2;

            if laser.pos.y < self.player.pos.y {
                laser.pos.y += velocity;
                laser.sprite.animate(&self.assets);

                // Handle collisions
                if self.collision.laser_to_player(laser, &self.player) {
                    // One of the end scenarios
                    self.gameover = true;

                    destroy.push(i);
                } else if self.collision.laser_to_bullet(laser, &mut self.bullet)
                    || self.collision.laser_to_shield(laser, &mut self.shields)
                {
                    destroy.push(i);
                }
            } else {
                destroy.push(i);
            }
        }

        // Destroy dead lasers
        for &i in destroy.iter().rev() {
            self.lasers.remove(i);
        }
    }

    /// Draw the internal state to the screen.
    ///
    /// Calling this method more than once without an `update` call between is a no-op.
    pub fn draw(&mut self, screen: &mut [u8]) {
        // Clear the screen
        clear(screen);

        // Draw the invaders
        for row in &self.invaders.grid {
            for invader in row.iter().flatten() {
                blit(screen, &invader.pos, &invader.sprite);
            }
        }

        // Draw the shields
        for shield in &self.shields {
            blit(screen, &shield.pos, &shield.sprite);
        }

        // Draw the player
        blit(screen, &self.player.pos, &self.player.sprite);

        // Draw the bullet
        if let Some(bullet) = &self.bullet {
            blit(screen, &bullet.pos, &bullet.sprite);
        }

        // Draw lasers
        for laser in self.lasers.iter() {
            blit(screen, &laser.pos, &laser.sprite);
        }

        // Draw debug information
        if self.debug {
            debug::draw_invaders(screen, &self.invaders, &self.collision);
            debug::draw_bullet(screen, self.bullet.as_ref());
            debug::draw_lasers(screen, &self.lasers);
            debug::draw_player(screen, &self.player, &self.collision);
            debug::draw_shields(screen, &self.shields, &self.collision);
        }
    }

    fn step_invaders(&mut self) {
        let (_, right, _, left) = self.invaders.get_bounds();
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
                        self.invaders.bounds.pos.x += 2;
                        self.invaders.bounds.pos.y += 8;
                        self.invaders.descend = true;
                        self.invaders.direction = Direction::Right;
                    } else {
                        self.invaders.bounds.pos.x -= 2;
                    }
                }
                Direction::Right => {
                    if right > WIDTH - 2 {
                        self.invaders.bounds.pos.x -= 2;
                        self.invaders.bounds.pos.y += 8;
                        self.invaders.descend = true;
                        self.invaders.direction = Direction::Left;
                    } else {
                        self.invaders.bounds.pos.x += 2;
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
        let r = self.prng.next_u32() as usize;
        let chance = r % 50;
        if self.lasers.len() < 3 && chance == 0 {
            // Pick a random column to begin searching for an invader that can fire a laser
            let col = r / 50 % COLS;
            let invader = self.invaders.get_closest_invader(col);

            let laser = Laser {
                sprite: SpriteRef::new(&self.assets, Frame::Laser1, Duration::from_millis(16)),
                pos: invader.pos + LASER_OFFSET,
                dt: Duration::default(),
            };
            self.lasers.push(laser);
        }
    }

    fn step_player(&mut self, controls: &Controls) {
        let frames = self.player.update();
        let width = self.player.sprite.width();

        match controls.direction {
            Direction::Left => {
                if self.player.pos.x > width {
                    self.player.pos.x -= frames;
                    self.player.sprite.animate(&self.assets);
                }
            }

            Direction::Right => {
                if self.player.pos.x < WIDTH - width * 2 {
                    self.player.pos.x += frames;
                    self.player.sprite.animate(&self.assets);
                }
            }
            _ => (),
        }

        if controls.fire && self.bullet.is_none() {
            self.bullet = Some(Bullet {
                sprite: SpriteRef::new(&self.assets, Frame::Bullet1, Duration::from_millis(32)),
                pos: self.player.pos + BULLET_OFFSET,
                dt: Duration::default(),
            });
        }
    }

    pub fn reset_game(&mut self) {
        // Recreate the alien
        self.invaders = Invaders::new(&self.assets);

        // Empty laser
        self.lasers.clear();

        // Recreate the shield
        self.shields = (0..4)
            .map(|i| Shield::new(&self.assets, Point::new(i * 45 + 32, 192)))
            .collect();

        // Reset player position
        self.player.pos = PLAYER_START;

        // Remove bullet
        self.bullet = None;

        // Reset collision state
        self.collision.clear();

        // Reset game score
        self._score = 0;

        // Set gameover to false
        self.gameover = false;
    }
}

/// Create a default `World` with a static PRNG seed.
impl Default for World {
    fn default() -> Self {
        let seed = (6_364_136_223_846_793_005, 1);

        World::new(seed, false)
    }
}

impl Invaders {
    // New
    pub fn new(assets: &Assets) -> Self {
        let grid = make_invader_grid(assets);
        let stepper = Point::new(COLS - 1, 0);
        let direction = Direction::Right;
        let descend = false;
        let bounds = Bounds::default();

        Invaders {
            grid,
            stepper,
            direction,
            descend,
            bounds,
        }
    }
    /// Compute the bounding box for the Invader fleet.
    ///
    /// # Returns
    ///
    /// Tuple of `(top, right, bottom, left)`, e.g. in CSS clockwise order.
    fn get_bounds(&self) -> (usize, usize, usize, usize) {
        let width = (self.bounds.right_col - self.bounds.left_col + 1) * GRID.x;
        let height = (self.bounds.bottom_row - self.bounds.top_row + 1) * GRID.y;

        let top = self.bounds.pos.y;
        let bottom = top + height;
        let left = self.bounds.pos.x;
        let right = left + width;

        (top, right, bottom, left)
    }

    /// Resize the bounds to fit the live invaders.
    ///
    /// # Returns
    ///
    /// `true` when all invaders have been destroyed.
    fn shrink_bounds(&mut self) -> bool {
        let mut top = ROWS;
        let mut right = 0;
        let mut bottom = 0;
        let mut left = COLS;

        // Scan through the entire grid
        for (y, row) in self.grid.iter().enumerate() {
            row.iter()
                .enumerate()
                .filter(|(_, col)| col.is_some())
                .for_each(|(x, _)| {
                    top = top.min(y);
                    bottom = bottom.max(y);
                    left = left.min(x);
                    right = right.max(x);
                });
        }

        if top > bottom || left > right {
            // No more invaders left alive
            return true;
        }

        // Adjust the bounding box position
        self.bounds.pos.x += (left - self.bounds.left_col) * GRID.x;
        self.bounds.pos.y += (top - self.bounds.top_row) * GRID.y;

        // Adjust the bounding box columns and rows
        self.bounds.left_col = left;
        self.bounds.right_col = right;
        self.bounds.top_row = top;
        self.bounds.bottom_row = bottom;

        // No more changes
        false
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
            pos: START,
            left_col: 0,
            right_col: COLS - 1,
            top_row: 0,
            bottom_row: ROWS - 1,
        }
    }
}

/// Clear the screen
fn clear(screen: &mut [u8]) {
    for (i, byte) in screen.iter_mut().enumerate() {
        *byte = if i % 4 == 3 { 255 } else { 0 };
    }
}

/// Create a grid of invaders.
fn make_invader_grid(assets: &Assets) -> Vec<Vec<Option<Invader>>> {
    use Frame::*;

    const BLIPJOY_OFFSET: Point = Point::new(3, 4);
    const FERRIS_OFFSET: Point = Point::new(2, 5);
    const CTHULHU_OFFSET: Point = Point::new(1, 3);

    (0..1)
        .map(|y| {
            (0..COLS)
                .map(|x| {
                    Some(Invader {
                        sprite: SpriteRef::new(assets, Blipjoy1, Duration::default()),
                        pos: START + BLIPJOY_OFFSET + Point::new(x, y) * GRID,
                        _score: 10,
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
                        _score: 10,
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
                        _score: 10,
                    })
                })
                .collect()
        }))
        .collect()
}

fn next_invader<'a>(
    invaders: &'a mut [Vec<Option<Invader>>],
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

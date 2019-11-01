//! A simple Space Invaders clone to demonstrate `pixels`.
//!
//! This doesn't use anything fancy like a game engine, so you may not want to build a game like
//! this in practice. That said, the game is fully functional, and it should not be too difficult
//! to understand the code.

#![deny(clippy::all)]

use std::time::Duration;

use crate::collision::Collision;
pub use crate::controls::{Controls, Direction};
use crate::geo::Point;
use crate::loader::{load_assets, Assets};
use crate::particles::Particle;
use crate::sprites::{blit, line, Animation, Drawable, Frame, Sprite, SpriteRef};
use arrayvec::ArrayVec;
use rand_core::{OsRng, RngCore};

mod collision;
mod controls;
mod debug;
mod geo;
mod loader;
mod particles;
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

// Player positioning
const PLAYER_START: Point = Point::new(80, 216);

// Projectile positioning
const LASER_OFFSET: Point = Point::new(4, 10);
const BULLET_OFFSET: Point = Point::new(7, 0);

#[derive(Debug)]
pub struct World {
    invaders: Option<Invaders>,
    lasers: Vec<Laser>,
    shields: Vec<Shield>,
    player: Option<Player>,
    bullet: Option<Bullet>,
    particles: Vec<Particle>,
    collision: Collision,
    score: u32,
    assets: Assets,
    dt: Duration,
    gameover: bool,
    prng: OsRng,
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
    score: u32,
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
    pub fn new(debug: bool) -> World {
        use Frame::*;

        // Load assets first
        let assets = load_assets();

        // TODO: Create invaders one-at-a-time
        let invaders = Some(Invaders {
            grid: make_invader_grid(&assets),
            stepper: Point::new(COLS - 1, 0),
            direction: Direction::Right,
            descend: false,
            bounds: Bounds::default(),
        });

        let lasers = Vec::with_capacity(3);
        let shields = (0..4)
            .map(|i| Shield {
                sprite: Sprite::new(&assets, Shield1),
                pos: Point::new(i * 45 + 32, 192),
            })
            .collect();

        let player = Some(Player {
            sprite: SpriteRef::new(&assets, Player1, Duration::from_millis(100)),
            pos: PLAYER_START,
            dt: 0,
        });

        let bullet = None;
        let particles = Vec::with_capacity(1024);
        let mut collision = Collision::default();
        collision.pixel_mask = Vec::with_capacity(SCREEN_WIDTH * SCREEN_HEIGHT * 4);
        collision
            .pixel_mask
            .resize_with(SCREEN_WIDTH * SCREEN_HEIGHT * 4, Default::default);
        let score = 0;

        let dt = Duration::default();
        let gameover = false;
        let prng = OsRng;

        World {
            invaders,
            lasers,
            shields,
            player,
            bullet,
            particles,
            collision,
            score,
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
    pub fn update(&mut self, dt: &Duration, controls: &Controls) {
        let one_frame = Duration::new(0, 16_666_667);

        // Advance the timer by the delta time
        self.dt += *dt;

        // Clear the collision details
        self.collision.clear();

        // Simulate particles
        let destroy = particles::update(&mut self.particles, dt, &self.collision);
        for &i in destroy.iter().rev() {
            self.particles.remove(i);
        }

        if !self.gameover {
            // Step the invaders one by one
            if self.invaders.is_some() {
                while self.dt >= one_frame {
                    self.dt -= one_frame;
                    self.step_invaders();
                }
            }

            // Handle player movement and animation
            if self.player.is_some() {
                self.step_player(controls, dt);
            }
        }

        if let Some(bullet) = &mut self.bullet {
            // Handle bullet movement
            let velocity = update_dt(&mut bullet.dt, dt) * 4;

            if bullet.pos.y > velocity {
                bullet.pos.y -= velocity;
                bullet.sprite.animate(&self.assets, dt);

                // Handle collisions
                if self.invaders.is_some() {
                    if let Some(mut particles) = self.collision.bullet_to_invader(
                        &mut self.bullet,
                        &mut self.invaders.as_mut().unwrap(),
                        &mut self.prng,
                    ) {
                        // Add particles to the world
                        for particle in particles.drain(..) {
                            self.particles.push(particle);
                        }

                        // One of the end scenarios
                        if self.invaders.as_mut().unwrap().shrink_bounds() {
                            self.gameover = true;
                            self.invaders = None;
                        }
                    }
                }
                if self.bullet.is_some() {
                    self.collision
                        .bullet_to_shield(&mut self.bullet, &mut self.shields);
                }
            } else {
                self.bullet = None;
            }
        }

        // Handle laser movement
        let mut destroy = ArrayVec::<[_; 3]>::new();
        for (i, laser) in self.lasers.iter_mut().enumerate() {
            let velocity = update_dt(&mut laser.dt, dt) * 2;

            if laser.pos.y < PLAYER_START.y {
                laser.pos.y += velocity;
                laser.sprite.animate(&self.assets, dt);

                // Handle collisions
                if self.player.is_some() {
                    if let Some(mut particles) = self.collision.laser_to_player(
                        laser,
                        &self.player.as_ref().unwrap(),
                        &mut self.prng,
                    ) {
                        // Add particles to the world
                        for particle in particles.drain(..) {
                            self.particles.push(particle);
                        }

                        // One of the end scenarios
                        self.gameover = true;
                        self.player = None;

                        destroy.push(i);
                    } else if let Some(mut particles) =
                        self.collision
                            .laser_to_bullet(laser, &mut self.bullet, &mut self.prng)
                    {
                        // Laser and bullet obliterate each other

                        // Add particles to the world
                        for particle in particles.drain(..) {
                            self.particles.push(particle);
                        }

                        destroy.push(i);
                    } else if self.collision.laser_to_shield(laser, &mut self.shields) {
                        // TODO
                        destroy.push(i);
                    }
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

        // Draw the ground
        {
            // TODO: Draw cracks where lasers hit
            let p1 = Point::new(0, PLAYER_START.y + 17);
            let p2 = Point::new(SCREEN_WIDTH, PLAYER_START.y + 17);

            line(screen, &p1, &p2, [255, 255, 255, 255]);
        }

        // Draw the invaders
        if self.invaders.is_some() {
            for row in &self.invaders.as_ref().unwrap().grid {
                for col in row {
                    if let Some(invader) = col {
                        blit(screen, &invader.pos, &invader.sprite);
                    }
                }
            }
        }

        // Draw the shields
        for shield in &self.shields {
            blit(screen, &shield.pos, &shield.sprite);
        }

        // Draw the player
        if self.player.is_some() {
            let player = self.player.as_ref().unwrap();
            blit(screen, &player.pos, &player.sprite);
        }

        // Draw the bullet
        if let Some(bullet) = &self.bullet {
            blit(screen, &bullet.pos, &bullet.sprite);
        }

        // Draw lasers
        for laser in self.lasers.iter() {
            blit(screen, &laser.pos, &laser.sprite);
        }

        // Copy screen to the backbuffer for particle simulation
        self.collision.pixel_mask.copy_from_slice(screen);

        // Draw particles
        particles::draw(screen, &self.particles);

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
        let invaders = self.invaders.as_mut().unwrap();
        let (_, right, _, left) = invaders.get_bounds();
        let (invader, is_leader) = next_invader(&mut invaders.grid, &mut invaders.stepper);

        // The leader controls the fleet
        if is_leader {
            // The leader first commands the fleet to stop descending
            invaders.descend = false;

            // Then the leader redirects the fleet when they reach the boundaries
            match invaders.direction {
                Direction::Left => {
                    if left < 2 {
                        invaders.bounds.pos.x += 2;
                        invaders.bounds.pos.y += 8;
                        invaders.descend = true;
                        invaders.direction = Direction::Right;
                    } else {
                        invaders.bounds.pos.x -= 2;
                    }
                }
                Direction::Right => {
                    if right > SCREEN_WIDTH - 2 {
                        invaders.bounds.pos.x -= 2;
                        invaders.bounds.pos.y += 8;
                        invaders.descend = true;
                        invaders.direction = Direction::Left;
                    } else {
                        invaders.bounds.pos.x += 2;
                    }
                }
                _ => unreachable!(),
            }
        }

        // Every invader in the fleet moves 2px per frame
        match invaders.direction {
            Direction::Left => invader.pos.x -= 2,
            Direction::Right => invader.pos.x += 2,
            _ => unreachable!(),
        }

        // And they descend 8px on command
        if invaders.descend {
            invader.pos.y += 8;

            // One of the end scenarios
            if self.player.is_some() && invader.pos.y + 8 >= self.player.as_ref().unwrap().pos.y {
                self.gameover = true;
                self.player = None;

                // TODO: Explosion!
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
            let invader = invaders.get_closest_invader(col);

            let laser = Laser {
                sprite: SpriteRef::new(&self.assets, Frame::Laser1, Duration::from_millis(16)),
                pos: invader.pos + LASER_OFFSET,
                dt: 0,
            };
            self.lasers.push(laser);
        }
    }

    fn step_player(&mut self, controls: &Controls, dt: &Duration) {
        let player = self.player.as_mut().unwrap();
        let frames = update_dt(&mut player.dt, dt);
        let width = player.sprite.width();

        match controls.direction {
            Direction::Left => {
                if player.pos.x > width {
                    player.pos.x -= frames;
                    player.sprite.animate(&self.assets, dt);
                }
            }

            Direction::Right => {
                if player.pos.x < SCREEN_WIDTH - width * 2 {
                    player.pos.x += frames;
                    player.sprite.animate(&self.assets, dt);
                }
            }
            _ => (),
        }

        if controls.fire && self.bullet.is_none() {
            self.bullet = Some(Bullet {
                sprite: SpriteRef::new(&self.assets, Frame::Bullet1, Duration::from_millis(32)),
                pos: player.pos + BULLET_OFFSET,
                dt: 0,
            });
        }
    }
}

impl Default for World {
    fn default() -> Self {
        World::new(false)
    }
}

impl Invaders {
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
            for (x, col) in row.iter().enumerate() {
                if col.is_some() {
                    // Build a boundary box of invaders in the grid
                    if top > y {
                        top = y;
                    }
                    if bottom < y {
                        bottom = y;
                    }
                    if left > x {
                        left = x;
                    }
                    if right < x {
                        right = x;
                    }
                }
            }
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

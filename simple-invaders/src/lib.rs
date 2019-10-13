//! A simple Space Invaders clone to demonstrate `pixels`.
//!
//! This doesn't use anything fancy like a game engine, so you may not want to build a game like
//! this in practice. That said, the game is fully functional, and it should not be too difficult
//! to understand the code.

use rand_core::{OsRng, RngCore};
use std::time::Duration;

pub use crate::controls::{Controls, Direction};
use crate::geo::{Point, Rect};
use crate::loader::{load_assets, Assets};
use crate::sprites::{blit, rect, Animation, Drawable, Frame, Sprite, SpriteRef};

mod controls;
mod geo;
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
    score: u32,
    assets: Assets,
    screen: Vec<u8>,
    dt: Duration,
    gameover: bool,
    random: OsRng,
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

/// Store information about collisions (for debug mode).
#[derive(Debug, Default)]
struct Collision {
    bullet_details: Vec<BulletDetail>,
    laser_details: Vec<LaserDetail>,
}

/// Information regarding collisions between bullets and invaders, lasers, or shields.
#[derive(Debug, Eq, PartialEq)]
enum BulletDetail {
    /// A grid position (col, row) for an invader.
    Invader(usize, usize),
    /// A shield index.
    Shield(usize),
    /// A laser index.
    Laser(usize),
}

/// Information regarding collisions between lasers and shields or the player.
#[derive(Debug, Eq, PartialEq)]
enum LaserDetail {
    /// A laser index and shield index pair.
    Shield(usize, usize),
    /// A laser index and the player.
    Player(usize),
}

impl World {
    /// Create a new simple-invaders `World`.
    pub fn new(debug: bool) -> World {
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
        let lasers = Vec::new();
        let shields = (0..4)
            .map(|i| Shield {
                sprite: Sprite::new(&assets, Shield1),
                pos: Point::new(i * 45 + 32, 192),
            })
            .collect();
        let player = Player {
            sprite: SpriteRef::new(&assets, Player1, Duration::from_millis(100)),
            pos: PLAYER_START,
            dt: 0,
        };
        let bullet = None;
        let collision = Collision::default();
        let score = 0;

        // Create a screen with the correct size
        let mut screen = Vec::new();
        screen.resize_with(SCREEN_WIDTH * SCREEN_HEIGHT * 4, Default::default);

        let dt = Duration::default();
        let gameover = false;
        let random = OsRng;

        World {
            invaders,
            lasers,
            shields,
            player,
            bullet,
            collision,
            score,
            assets,
            screen,
            dt,
            gameover,
            random,
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

        // Clear the collision details
        self.collision.bullet_details.clear();
        self.collision.laser_details.clear();

        // Step the invaders one by one
        while self.dt >= one_frame {
            self.dt -= one_frame;
            self.step_invaders();
        }

        // Handle player movement and animation
        self.step_player(controls, dt);

        if let Some(bullet) = &mut self.bullet {
            // Handle bullet movement
            let velocity = update_dt(&mut bullet.dt, dt) * 4;

            if bullet.pos.y > velocity {
                bullet.pos.y -= velocity;
                bullet.sprite.animate(&self.assets, dt);

                // Handle bullet collisions with invaders

                // Broad phase collision detection
                let (top, right, bottom, left) = self.invaders.get_bounds();
                let invaders_rect = Rect::new(&Point::new(left, top), &Point::new(right, bottom));
                let bullet_rect = Rect::from_drawable(&bullet.pos, &bullet.sprite);
                if bullet_rect.intersects(&invaders_rect) {
                    // Narrow phase collision detection
                    let corners = [
                        // Upper left corner of bullet
                        (bullet_rect.p1.x, bullet_rect.p1.y),
                        // Upper right corner of bullet
                        (bullet_rect.p1.x, bullet_rect.p2.y),
                        // Lower left corner of bullet
                        (bullet_rect.p2.x, bullet_rect.p1.y),
                        // Lower right corner of bullet
                        (bullet_rect.p2.x, bullet_rect.p2.y),
                    ];

                    for (x, y) in corners.iter() {
                        let col = (x - left) / GRID.x + self.invaders.bounds.left_col;
                        let row = (y - top) / GRID.y + self.invaders.bounds.top_row;

                        if col < COLS && row < ROWS && self.invaders.grid[row][col].is_some() {
                            let detail = BulletDetail::Invader(col, row);
                            self.collision.push_bullet_detail(detail);
                        }
                    }

                    // If any collision candidate is a hit, kill the bullet and invader
                    for detail in self.collision.bullet_details.iter() {
                        if let BulletDetail::Invader(x, y) = *detail {
                            let invader = self.invaders.grid[y][x].as_ref().unwrap();
                            let invader_rect = Rect::from_drawable(&invader.pos, &invader.sprite);
                            if bullet_rect.intersects(&invader_rect) {
                                // TODO: Explosion! Score!
                                self.invaders.grid[y][x] = None;
                                self.bullet = None;
                            }
                        }
                    }
                }
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

                // Handler laser collisions with player

                let laser_rect = Rect::from_drawable(&laser.pos, &laser.sprite);
                let player_rect = Rect::from_drawable(&self.player.pos, &self.player.sprite);
                if laser_rect.intersects(&player_rect) {
                    // One of the end scenarios
                    self.gameover = true;

                    self.collision.laser_details.push(LaserDetail::Player(i));
                }
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
            // Colors
            let red = [255, 0, 0, 255];
            let green = [0, 255, 0, 255];
            let blue = [0, 0, 255, 255];
            let yellow = [255, 255, 0, 255];

            // Draw invaders bounding box
            {
                let (top, right, bottom, left) = self.invaders.get_bounds();
                let p1 = Point::new(left, top);
                let p2 = Point::new(right, bottom);

                rect(&mut self.screen, &p1, &p2, blue);
            }

            // Draw bounding boxes for each invader
            for (y, row) in self.invaders.grid.iter().enumerate() {
                for (x, col) in row.iter().enumerate() {
                    let detail = BulletDetail::Invader(x, y);
                    if let Some(invader) = col {
                        let p1 = invader.pos;
                        let p2 = p1 + Point::new(invader.sprite.width(), invader.sprite.height());

                        // Select color based on proximity to bullet
                        let color = if self.collision.bullet_details.contains(&detail) {
                            yellow
                        } else {
                            green
                        };

                        rect(&mut self.screen, &p1, &p2, color);
                    } else if self.collision.bullet_details.contains(&detail) {
                        let p1 = self.invaders.bounds.pos + Point::new(x, y) * GRID;
                        let p2 = p1 + GRID;

                        rect(&mut self.screen, &p1, &p2, red);
                    }
                }
            }

            // Draw bounding box for bullet
            if let Some(bullet) = &self.bullet {
                let p1 = bullet.pos;
                let p2 = p1 + Point::new(bullet.sprite.width(), bullet.sprite.height());

                rect(&mut self.screen, &p1, &p2, green);
            }

            // Draw bounding box for lasers
            for (i, laser) in self.lasers.iter().enumerate() {
                let p1 = laser.pos;
                let p2 = p1 + Point::new(laser.sprite.width(), laser.sprite.height());

                // Select color based on collision
                let detail = LaserDetail::Player(i);
                let color = if self.collision.laser_details.contains(&detail) {
                    red
                } else {
                    green
                };

                rect(&mut self.screen, &p1, &p2, color);
            }

            // Draw bounding box for player
            {
                let p1 = self.player.pos;
                let p2 = p1 + Point::new(self.player.sprite.width(), self.player.sprite.height());
                let color = if self.collision.laser_details.is_empty() {
                    green
                } else {
                    red
                };

                rect(&mut self.screen, &p1, &p2, color);
            }
        }

        &self.screen
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
                    if right > SCREEN_WIDTH - 2 {
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
        let r = self.random.next_u32() as usize;
        let chance = r % 50;
        if self.lasers.len() < 3 && chance == 0 {
            // Pick a random column to begin searching for an invader that can fire a laser
            let col = r / 50 % COLS;
            let invader = self.invaders.get_closest_invader(col);

            let laser = Laser {
                sprite: SpriteRef::new(&self.assets, Frame::Laser1, Duration::from_millis(16)),
                pos: invader.pos + LASER_OFFSET,
                dt: 0,
            };
            self.lasers.push(laser);
        }
    }

    fn step_player(&mut self, controls: &Controls, dt: &Duration) {
        let frames = update_dt(&mut self.player.dt, dt);
        let width = self.player.sprite.width();

        match controls.direction {
            Direction::Left => {
                if self.player.pos.x > width {
                    self.player.pos.x -= frames;
                    self.player.sprite.animate(&self.assets, dt);
                }
            }

            Direction::Right => {
                if self.player.pos.x < SCREEN_WIDTH - width * 2 {
                    self.player.pos.x += frames;
                    self.player.sprite.animate(&self.assets, dt);
                }
            }
            _ => (),
        }

        if controls.fire && self.bullet.is_none() {
            self.bullet = Some(Bullet {
                sprite: SpriteRef::new(&self.assets, Frame::Bullet1, Duration::from_millis(32)),
                pos: self.player.pos + BULLET_OFFSET,
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

impl Collision {
    fn push_bullet_detail(&mut self, detail: BulletDetail) {
        if !self.bullet_details.contains(&detail) {
            self.bullet_details.push(detail);
        }
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

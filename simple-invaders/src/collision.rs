//! Collision detection primitives.

use std::collections::HashSet;

use crate::geo::{convex_hull, Point, Rect, Vec2D};
use crate::particles::{drawable_to_particles, Particle};
use crate::sprites::Drawable;
use crate::{
    Bullet, Invaders, Laser, Player, Shield, COLS, GRID, ROWS, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use arrayvec::ArrayVec;
use line_drawing::Bresenham;
use rand_core::RngCore;

/// Store information about collisions (for debug mode).
#[derive(Debug, Default)]
pub(crate) struct Collision {
    pub(crate) bullet_details: HashSet<BulletDetail>,
    pub(crate) laser_details: HashSet<LaserDetail>,
    pub(crate) pixel_mask: Vec<u8>,
}

/// Information regarding collisions between bullets and invaders, lasers, or shields.
#[derive(Debug, Eq, Hash, PartialEq)]
pub(crate) enum BulletDetail {
    /// A grid position (col, row) for an invader.
    Invader(usize, usize),
    /// A shield index.
    Shield(usize),
    /// Collided with a laser.
    Laser,
}

/// Information regarding collisions between lasers and shields or the player.
#[derive(Debug, Eq, Hash, PartialEq)]
pub(crate) enum LaserDetail {
    /// A shield index.
    Shield(usize),
    /// Collided with the player.
    Player,
}

impl Collision {
    /// Clear the collision details.
    pub(crate) fn clear(&mut self) {
        self.bullet_details.clear();
        self.laser_details.clear();
    }

    /// Handle collisions between bullets and invaders.
    pub(crate) fn bullet_to_invader<R>(
        &mut self,
        bullet: &mut Option<Bullet>,
        invaders: &mut Invaders,
        prng: &mut R,
    ) -> Option<ArrayVec<[Particle; 1024]>>
    where
        R: RngCore,
    {
        // Broad phase collision detection
        let (top, right, bottom, left) = invaders.get_bounds();
        let invaders_rect = Rect::new(Point::new(left, top), Point::new(right, bottom));
        let bullet_rect = {
            let bullet = bullet.as_ref().unwrap();
            Rect::from_drawable(bullet.pos, &bullet.sprite)
        };
        if bullet_rect.intersects(invaders_rect) {
            // Narrow phase collision detection
            let corners = [
                (bullet_rect.p1.x, bullet_rect.p1.y),
                (bullet_rect.p1.x, bullet_rect.p2.y),
                (bullet_rect.p2.x, bullet_rect.p1.y),
                (bullet_rect.p2.x, bullet_rect.p2.y),
            ];

            for (x, y) in corners.iter() {
                let col = (x - left) / GRID.x + invaders.bounds.left_col;
                let row = (y - top) / GRID.y + invaders.bounds.top_row;

                if col < COLS && row < ROWS && invaders.grid[row][col].is_some() {
                    let detail = BulletDetail::Invader(col, row);
                    self.bullet_details.insert(detail);
                }
            }

            // If any collision candidate is a hit, kill the bullet and invader
            for detail in self.bullet_details.iter() {
                if let BulletDetail::Invader(x, y) = *detail {
                    let invader = invaders.grid[y][x].as_ref().unwrap();
                    let invader_rect = Rect::from_drawable(invader.pos, &invader.sprite);
                    if bullet_rect.intersects(invader_rect) {
                        // TODO: Score!

                        // Create a spectacular explosion!
                        let mut particles = {
                            let bullet = bullet.as_ref().unwrap();
                            let force = 4.0;
                            let center = Vec2D::from(bullet.pos) - Vec2D::from(invader.pos)
                                + Vec2D::new(0.9, 1.9);

                            drawable_to_particles(
                                prng,
                                invader.pos,
                                &invader.sprite,
                                invader.sprite.rect(),
                                force,
                                center,
                            )
                        };
                        let mut bullet_particles = {
                            let bullet = bullet.as_ref().unwrap();
                            let force = 4.0;
                            let center = Vec2D::new(0.9, 4.1);

                            drawable_to_particles(
                                prng,
                                bullet.pos,
                                &bullet.sprite,
                                bullet.sprite.rect(),
                                force,
                                center,
                            )
                        };
                        for particle in bullet_particles.drain(..) {
                            particles.push(particle);
                        }

                        // Destroy invader
                        invaders.grid[y][x] = None;

                        // Destroy bullet
                        *bullet = None;

                        return Some(particles);
                    }
                }
            }
        }

        None
    }

    /// Handle collisions between bullets and shields.
    pub(crate) fn bullet_to_shield(&mut self, bullet: &mut Option<Bullet>, shields: &mut [Shield]) {
        if bullet.is_some() {
            let shield_rects = create_shield_rects(shields);
            let bullet_rect = {
                let bullet = bullet.as_ref().unwrap();
                Rect::from_drawable(bullet.pos, &bullet.sprite)
            };

            for (i, &shield_rect) in shield_rects.iter().enumerate() {
                // broad phase collision detection
                if bullet_rect.intersects(shield_rect) {
                    // TODO: Narrow phase (per-pixel) collision detection
                    // TODO: Break shield

                    // TODO: Explosion!
                    let detail = BulletDetail::Shield(i);
                    self.bullet_details.insert(detail);

                    // Destroy bullet
                    *bullet = None;
                }
            }
        }
    }

    /// Handle collisions between lasers and the player.
    pub(crate) fn laser_to_player<R>(
        &mut self,
        laser: &Laser,
        player: &Player,
        prng: &mut R,
    ) -> Option<ArrayVec<[Particle; 1024]>>
    where
        R: RngCore,
    {
        let laser_rect = Rect::from_drawable(laser.pos, &laser.sprite);
        let player_rect = Rect::from_drawable(player.pos, &player.sprite);
        if laser_rect.intersects(player_rect) {
            self.laser_details.insert(LaserDetail::Player);

            // Create a spectacular explosion!
            let mut particles = {
                let force = 8.0;
                let center =
                    Vec2D::from(laser.pos) - Vec2D::from(player.pos) + Vec2D::new(2.5, 3.5);

                drawable_to_particles(
                    prng,
                    player.pos,
                    &player.sprite,
                    player.sprite.rect(),
                    force,
                    center,
                )
            };
            let mut bullet_particles = {
                let force = 8.0;
                let center = Vec2D::new(2.5, -0.5);

                drawable_to_particles(
                    prng,
                    laser.pos,
                    &laser.sprite,
                    laser.sprite.rect(),
                    force,
                    center,
                )
            };
            for particle in bullet_particles.drain(..) {
                particles.push(particle);
            }

            Some(particles)
        } else {
            None
        }
    }

    /// Handle collisions between lasers and bullets.
    pub(crate) fn laser_to_bullet(&mut self, laser: &Laser, bullet: &mut Option<Bullet>) -> bool {
        let mut destroy = false;
        if bullet.is_some() {
            let laser_rect = Rect::from_drawable(laser.pos, &laser.sprite);

            if let Some(bullet) = &bullet {
                let bullet_rect = Rect::from_drawable(bullet.pos, &bullet.sprite);
                if bullet_rect.intersects(laser_rect) {
                    // TODO: Explosion!
                    let detail = BulletDetail::Laser;
                    self.bullet_details.insert(detail);

                    // Destroy laser and bullet
                    destroy = true;
                }
            }

            if destroy {
                *bullet = None;
            }
        }

        destroy
    }

    /// Handle collisions between lasers and shields.
    pub(crate) fn laser_to_shield(&mut self, laser: &Laser, shields: &mut [Shield]) -> bool {
        let laser_rect = Rect::from_drawable(laser.pos, &laser.sprite);
        let shield_rects = create_shield_rects(shields);
        let mut destroy = false;

        for (i, &shield_rect) in shield_rects.iter().enumerate() {
            // broad phase collision detection
            if laser_rect.intersects(shield_rect) {
                // TODO: Narrow phase (per-pixel) collision detection
                // TODO: Break shield

                // TODO: Explosion!
                let detail = LaserDetail::Shield(i);
                self.laser_details.insert(detail);

                // Destroy laser
                destroy = true;
            }
        }

        destroy
    }

    /// Trace a ray between the line segment formed by `start, end`, looking for collisions with
    /// the collision mask. When a hit is detected, return the position of the hit and a new
    /// velocity vector representing how the ray will proceed after bounding.
    ///
    /// In the case of no hits, returns `None`.
    ///
    /// # Arguments
    ///
    /// * `start` - Particle's current position.
    /// * `end` - Particle's predicted position (must be `start + velocity`)
    /// * `velocity` - Particle's vector of motion.
    pub(crate) fn trace(
        &self,
        start: Vec2D,
        end: Vec2D,
        velocity: Vec2D,
    ) -> Option<(Vec2D, Vec2D)> {
        let p1 = (start.x.round() as i32, start.y.round() as i32);
        let p2 = (end.x.round() as i32, end.y.round() as i32);
        let stride = SCREEN_WIDTH * 4;

        let mut hit = start;

        // Trace the particle's trajectory, checking each pixel in the collision mask along the way
        for (x, y) in Bresenham::new(p1, p2) {
            let x = x as usize;
            let y = y as usize;
            let index = x * 4 + y * stride;

            // Only checking the red channel, that's all we really need
            if x > 0
                && y > 0
                && x < SCREEN_WIDTH - 1
                && y < SCREEN_HEIGHT - 1
                && self.pixel_mask[index] > 0
            {
                // TODO: Split this into its own function!

                // A 5x5 grid with four points surrounding each pixel center needs 60 points max
                let mut points = ArrayVec::<[_; 64]>::new();

                // Create a list of vertices representing neighboring pixels.
                for v in y - 2..=y + 2 {
                    for u in x - 2..=x + 2 {
                        let index = u * 4 + v * stride;

                        // Only checking the red channel, again
                        if self.pixel_mask[index] > 0 {
                            let s = u as f32;
                            let t = v as f32;

                            // Top and left sides of the pixel
                            points.push(Vec2D::new(s, t - 0.5));
                            points.push(Vec2D::new(s - 0.5, t));

                            // Inspect neighboring pixels to determine whether we need to also add
                            // the bottom and right sides of the pixel. This de-dupes overlapping
                            // points.
                            if u == x + 2 || self.pixel_mask[index + 4] == 0 {
                                // Right side
                                points.push(Vec2D::new(s + 0.5, t));
                            }
                            if v == y + 2 || self.pixel_mask[index + stride] == 0 {
                                // Bottom side
                                points.push(Vec2D::new(s, t + 0.5));
                            }
                        }
                    }
                }

                // Compute the convex hull of the set of points.
                let hull = convex_hull(&points);

                // For each line segment in the convex hull, compute the intersection between the
                // line segment and the particle trajectory, keeping only the line segment that
                // intersects closest to the particle's current position. In other words, find
                // which slope the particle collides with.
                let mut closest = end;
                let mut slope = Vec2D::default();
                for (&p1, &p2) in hull.iter().zip(hull.iter().skip(1)) {
                    // The cross product between two line segments can tell use whether they
                    // intersect and where. This is adapted from "Intersection of two lines in
                    // three-space" by Ronald Goldman, published in Graphics Gems, page 304.

                    // First we take the cross product between the velocity vector and the
                    // difference between the two points on the hull.
                    let magnitude = p2 - p1;
                    let cross = velocity.cross(magnitude);

                    if cross.abs() < std::f32::EPSILON {
                        // Line segments are colinear or parallel
                        continue;
                    }

                    // Interpolate the velocity vector toward the intersection
                    let t = (p1 - start).cross(magnitude) / cross;
                    let candidate = velocity.scale(t);

                    // Record the closest intersecting line segment
                    if candidate.len_sq() < closest.len_sq() {
                        closest = candidate;
                        slope = magnitude;
                    }
                }

                // We now have a slope along the particle's trajectory. All that is left to do is
                // reflecting the velocity around the slope's angle.

                // Compute the angles of the velocity and slope vectors.
                let theta = velocity.y.atan2(velocity.x);
                let alpha = slope.y.atan2(slope.x);

                // Reflect theta around alpha.
                // https://en.wikipedia.org/wiki/List_of_trigonometric_identities#Reflections
                let theta_prime = alpha * 2.0 - theta;

                // Update velocity and apply friction.
                let magnitude = velocity.len();
                let velocity =
                    Vec2D::new(theta_prime.cos() * magnitude, theta_prime.sin() * magnitude);
                let velocity = velocity * Vec2D::new(0.8, 0.8);

                return Some((hit, velocity));
            }

            // Defer the hit location by 1 pixel. A fudge factor to prevent particles from getting
            // stuck inside solids.
            // TODO: I would like to instead walk the ray from the hit point through the updated
            // velocity until the particle stops colliding. This will prevent "unpredictable"
            // movements in the collision mask from capturing particles.
            hit.x = x as f32;
            hit.y = y as f32;
        }

        None
    }

    // TODO: Detect collisions between a `Drawable` and the internal pixel mask.
}

fn create_shield_rects(shields: &[Shield]) -> [Rect; 4] {
    [
        Rect::from_drawable(shields[0].pos, &shields[0].sprite),
        Rect::from_drawable(shields[1].pos, &shields[1].sprite),
        Rect::from_drawable(shields[2].pos, &shields[2].sprite),
        Rect::from_drawable(shields[3].pos, &shields[3].sprite),
    ]
}

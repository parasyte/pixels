//! Collision detection primitives.

use crate::geo::{Point, Rect};
use crate::{Bullet, Invaders, Laser, Player, Shield, COLS, GRID, ROWS};
use alloc::collections::BTreeSet;

/// Store information about collisions (for debug mode).
#[derive(Debug, Default)]
pub(crate) struct Collision {
    pub(crate) bullet_details: BTreeSet<BulletDetail>,
    pub(crate) laser_details: BTreeSet<LaserDetail>,
}

/// Information regarding collisions between bullets and invaders, lasers, or shields.
#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub(crate) enum BulletDetail {
    /// A grid position (col, row) for an invader.
    Invader(usize, usize),
    /// A shield index.
    Shield(usize),
    /// Collided with a laser.
    Laser,
}

/// Information regarding collisions between lasers and shields or the player.
#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
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
    pub(crate) fn bullet_to_invader(
        &mut self,
        bullet: &mut Option<Bullet>,
        invaders: &mut Invaders,
    ) -> bool {
        // Broad phase collision detection
        let (top, right, bottom, left) = invaders.get_bounds();
        let invaders_rect = Rect::new(&Point::new(left, top), &Point::new(right, bottom));
        let bullet_rect = {
            let bullet = bullet.as_ref().unwrap();
            Rect::from_drawable(&bullet.pos, &bullet.sprite)
        };
        if bullet_rect.intersects(&invaders_rect) {
            // Narrow phase collision detection
            let corners = [
                (bullet_rect.p1.x, bullet_rect.p1.y),
                (bullet_rect.p1.x, bullet_rect.p2.y),
                (bullet_rect.p2.x, bullet_rect.p1.y),
                (bullet_rect.p2.x, bullet_rect.p2.y),
            ];

            for (x, y) in corners.iter() {
                let col = x.saturating_sub(left) / GRID.x + invaders.bounds.left_col;
                let row = y.saturating_sub(top) / GRID.y + invaders.bounds.top_row;

                if col < COLS && row < ROWS && invaders.grid[row][col].is_some() {
                    let detail = BulletDetail::Invader(col, row);
                    self.bullet_details.insert(detail);
                }
            }

            // If any collision candidate is a hit, kill the bullet and invader
            for detail in self.bullet_details.iter() {
                if let BulletDetail::Invader(x, y) = *detail {
                    let invader = invaders.grid[y][x].as_ref().unwrap();
                    let invader_rect = Rect::from_drawable(&invader.pos, &invader.sprite);
                    if bullet_rect.intersects(&invader_rect) {
                        // TODO: Explosion! Score!
                        invaders.grid[y][x] = None;

                        // Destroy bullet
                        *bullet = None;

                        return true;
                    }
                }
            }
        }

        false
    }

    /// Handle collisions between bullets and shields.
    pub(crate) fn bullet_to_shield(&mut self, bullet: &mut Option<Bullet>, shields: &mut [Shield]) {
        if bullet.is_some() {
            let shield_rects = create_shield_rects(shields);
            let bullet_rect = {
                let bullet = bullet.as_ref().unwrap();
                Rect::from_drawable(&bullet.pos, &bullet.sprite)
            };

            for (i, shield_rect) in shield_rects.iter().enumerate() {
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
    pub(crate) fn laser_to_player(&mut self, laser: &Laser, player: &Player) -> bool {
        let laser_rect = Rect::from_drawable(&laser.pos, &laser.sprite);
        let player_rect = Rect::from_drawable(&player.pos, &player.sprite);
        if laser_rect.intersects(&player_rect) {
            self.laser_details.insert(LaserDetail::Player);
            true
        } else {
            false
        }
    }

    /// Handle collisions between lasers and bullets.
    pub(crate) fn laser_to_bullet(&mut self, laser: &Laser, bullet: &mut Option<Bullet>) -> bool {
        let mut destroy = false;
        if bullet.is_some() {
            let laser_rect = Rect::from_drawable(&laser.pos, &laser.sprite);

            if let Some(bullet) = &bullet {
                let bullet_rect = Rect::from_drawable(&bullet.pos, &bullet.sprite);
                if bullet_rect.intersects(&laser_rect) {
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
        let laser_rect = Rect::from_drawable(&laser.pos, &laser.sprite);
        let shield_rects = create_shield_rects(shields);
        let mut destroy = false;

        for (i, shield_rect) in shield_rects.iter().enumerate() {
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
}

fn create_shield_rects(shields: &[Shield]) -> [Rect; 4] {
    [
        Rect::from_drawable(&shields[0].pos, &shields[0].sprite),
        Rect::from_drawable(&shields[1].pos, &shields[1].sprite),
        Rect::from_drawable(&shields[2].pos, &shields[2].sprite),
        Rect::from_drawable(&shields[3].pos, &shields[3].sprite),
    ]
}

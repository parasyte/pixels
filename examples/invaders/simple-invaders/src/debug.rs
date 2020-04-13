use crate::collision::{BulletDetail, Collision, LaserDetail};
use crate::geo::Point;
use crate::sprites::{rect, Drawable};
use crate::{Bullet, Invaders, Laser, Player, Shield, GRID};

// Colors
const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const YELLOW: [u8; 4] = [255, 255, 0, 255];

/// Draw bounding boxes for the invader fleet and each invader.
pub(crate) fn draw_invaders(screen: &mut [u8], invaders: &Invaders, collision: &Collision) {
    // Draw invaders bounding box
    {
        let (top, right, bottom, left) = invaders.get_bounds();
        let p1 = Point::new(left, top);
        let p2 = Point::new(right, bottom);

        rect(screen, &p1, &p2, BLUE);
    }

    // Draw bounding boxes for each invader
    for (y, row) in invaders.grid.iter().enumerate() {
        for (x, col) in row.iter().enumerate() {
            let detail = BulletDetail::Invader(x, y);
            if let Some(invader) = col {
                let p1 = invader.pos;
                let p2 = p1 + Point::new(invader.sprite.width(), invader.sprite.height());

                // Select color based on proximity to bullet
                let color = if collision.bullet_details.contains(&detail) {
                    YELLOW
                } else {
                    GREEN
                };

                rect(screen, &p1, &p2, color);
            } else if collision.bullet_details.contains(&detail) {
                let x = x - invaders.bounds.left_col;
                let y = y - invaders.bounds.top_row;
                let p1 = invaders.bounds.pos + Point::new(x, y) * GRID;
                let p2 = p1 + GRID;

                rect(screen, &p1, &p2, RED);
            }
        }
    }
}

/// Draw bounding box for bullet.
pub(crate) fn draw_bullet(screen: &mut [u8], bullet: Option<&Bullet>) {
    if let Some(bullet) = bullet {
        let p1 = bullet.pos;
        let p2 = p1 + Point::new(bullet.sprite.width(), bullet.sprite.height());

        rect(screen, &p1, &p2, GREEN);
    }
}

/// Draw bounding box for lasers.
pub(crate) fn draw_lasers(screen: &mut [u8], lasers: &[Laser]) {
    for laser in lasers {
        let p1 = laser.pos;
        let p2 = p1 + Point::new(laser.sprite.width(), laser.sprite.height());

        rect(screen, &p1, &p2, GREEN);
    }
}

/// Draw bounding box for player.
pub(crate) fn draw_player(screen: &mut [u8], player: &Player, collision: &Collision) {
    let p1 = player.pos;
    let p2 = p1 + Point::new(player.sprite.width(), player.sprite.height());

    // Select color based on collisions
    let detail = LaserDetail::Player;
    let color = if collision.laser_details.contains(&detail) {
        RED
    } else {
        GREEN
    };

    rect(screen, &p1, &p2, color);
}

/// Draw bounding boxes for shields.
pub(crate) fn draw_shields(screen: &mut [u8], shields: &[Shield], collision: &Collision) {
    for (i, shield) in shields.iter().enumerate() {
        let p1 = shield.pos;
        let p2 = p1 + Point::new(shield.sprite.width(), shield.sprite.height());

        // Select color based on collisions
        let laser_detail = LaserDetail::Shield(i);
        let bullet_detail = BulletDetail::Shield(i);
        let color = if collision.laser_details.contains(&laser_detail)
            || collision.bullet_details.contains(&bullet_detail)
        {
            RED
        } else {
            GREEN
        };

        rect(screen, &p1, &p2, color);
    }
}

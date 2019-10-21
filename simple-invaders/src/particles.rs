//! Particle simulation primitives.

use crate::collision::Collision;
use crate::geo::{Point, Rect, Vec2D};
use crate::sprites::Drawable;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use arrayvec::ArrayVec;
use rand_core::RngCore;
use std::time::Duration;

/// Particles a 1x1 pixels that fly around all crazy like.
#[derive(Debug)]
pub(crate) struct Particle {
    /// Position in the simulation, relative to upper-left corner. (For physics).
    pos: Vec2D,
    /// Absolute position (for drawing).
    abs_pos: Point,
    /// Direction and magnitude of motion.
    velocity: Vec2D,
    /// This is how long the particle remains alive at full brightness. It will countdown to zero
    /// then start fading.
    alive: Duration,
    /// This is how long the particle remains visible while fading. It is an absolute duration,
    /// not a countdown; see `dt`.
    fade: Duration,
    /// The delta time for the fade counter. When the particle is no longer "alive", this will
    /// countdown to zero then the particle will die.
    dt: Duration,
}

/// Run particle simulation.
pub(crate) fn update(
    particles: &mut [Particle],
    dt: &Duration,
    collision: &Collision,
) -> ArrayVec<[usize; 1024]> {
    // TODO:
    // - [x] Move particles.
    // - [x] Apply gravity.
    // - [x] Apply friction.
    // - [ ] Detect collisions.
    // - [ ] Apply collision reaction.
    // - [ ] Particle decay.
    // - [ ] Particle fade.
    // - [ ] Particle death.
    // - [ ] Scale by `dt`.

    let mut destroy = ArrayVec::new();

    for (i, particle) in particles.iter_mut().enumerate() {
        // Apply gravity
        particle.velocity.y += 0.20;

        // Apply damping / friction
        particle.velocity.x *= 0.985;
        particle.velocity.y *= 0.985;

        // Apply velocity
        let prediction = particle.pos + particle.velocity;

        // Ensure the position is within view. Destroys particles that are on the screen's border.
        if prediction.x >= 1.0
            && prediction.x < (SCREEN_WIDTH - 1) as f32
            && prediction.y >= 1.0
            && prediction.y < (SCREEN_HEIGHT - 1) as f32
        {
            // Apply collision detection and update particle state
            // TODO: Apply collision detection multiple times until the particle stops bouncing
            if let Some((pos, velocity)) =
                collision.trace(particle.pos, prediction, particle.velocity)
            {
                // TODO
                particle.pos = pos;
                particle.velocity = velocity;
            } else {
                // Update position
                particle.pos = prediction;
            }

            // Convert to absolute position
            particle.abs_pos = Point::from(particle.pos);
        } else {
            destroy.push(i);
        }
    }

    destroy
}

/// Draw particles.
///
/// # Panics
///
/// Asserts that the particle's absolute position is within the screen.
pub(crate) fn draw(screen: &mut [u8], particles: &[Particle]) {
    for particle in particles {
        assert!(particle.abs_pos.x <= SCREEN_WIDTH);
        assert!(particle.abs_pos.y <= SCREEN_HEIGHT);

        // Generate a shade of gray based on the particle lifetime and fade
        let shade = if particle.alive > Duration::new(0, 0) {
            255
        } else {
            let dt = particle.dt.subsec_nanos() as f32;
            let fade = particle.fade.subsec_nanos() as f32;

            ((dt / fade).min(1.0) * 255.0) as u8
        };
        let color = [shade, shade, shade, 255];
        let i = particle.abs_pos.x * 4 + particle.abs_pos.y * SCREEN_WIDTH * 4;

        screen[i..i + 4].copy_from_slice(&color);
    }
}

/// Create particles from a `Drawable`.
///
/// The particles are copied from a sprite, pixel-by-pixel. Forces are applied independently to
/// each particle, based on the `force` vector and size/position of the `other` rectangle.
///
/// # Arguments
///
/// * `prng` - A PRNG for providing some variance to emitted particles.
/// * `pos` - The screen position for the `Drawable`.
/// * `drawable` - The sprite that is being copied.
/// * `src` - A rectangle subset of the sprite to copy.
/// * `force` - An impulse force applied to all particles.
/// * `center` - Center of mass for impulse `force`.
///
/// # Panics
///
/// The `center` should be offset by 0.5 on each axis to prevent dividing by zero. This function
/// panics if `center.x.fract() == 0.0 || center.y.fract() == 0.0`.
///
/// It also asserts that the `src` rectangle is fully contained within the `drawable`.
pub(crate) fn drawable_to_particles<D, R>(
    prng: &mut R,
    pos: &Point,
    drawable: &D,
    src: &Rect,
    force: f32,
    center: &Vec2D,
) -> ArrayVec<[Particle; 1024]>
where
    D: Drawable,
    R: RngCore,
{
    let width = drawable.width();
    let height = drawable.height();
    assert!(src.p1.x < width && src.p2.x <= width && src.p1.x < src.p2.x);
    assert!(src.p1.y < height && src.p2.y <= height && src.p1.y < src.p2.y);
    assert!(center.x.fract().abs() > std::f32::EPSILON);
    assert!(center.y.fract().abs() > std::f32::EPSILON);

    // The "extreme" is the longest side of the sprite multiplied by the square root of 2 with some
    // fudge factor. In other words, it's the longest vector length expected between the center of
    // mass and any other pixel. This value is used to approximate how much influence the force has
    // on each particle.
    let extreme = if width > height { width } else { height } as f32 * 1.28;

    let mut particles = ArrayVec::new();
    let pixels = drawable.pixels();

    for y in src.p1.y..src.p2.y {
        for x in src.p1.x..src.p2.x {
            let i = x * 4 + y * width * 4;

            // Only checking the red channel, that's all we really need
            if pixels[i] > 0 {
                // Initialize velocity using force and center of mass
                let mut velocity = Vec2D::new(x as f32, y as f32) - *center;
                let scale = (extreme - velocity.len()) / extreme;
                velocity.normalize();
                velocity.scale(scale * force);

                // Add some random variance [-0.5, 0.5) to the velocity
                let rx = prng.next_u32() as f32 / std::u32::MAX as f32 - 0.5;
                let ry = prng.next_u32() as f32 / std::u32::MAX as f32 - 0.5;
                velocity += Vec2D::new(rx, ry);

                let abs_pos = *pos + Point::new(x, y);

                particles.push(Particle {
                    pos: Vec2D::from(abs_pos),
                    abs_pos,
                    velocity,
                    alive: Duration::new(2, 0),
                    fade: Duration::new(5, 0),
                    dt: Duration::default(),
                });
            }
        }
    }

    particles
}

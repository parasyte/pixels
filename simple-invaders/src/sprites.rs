use std::rc::Rc;
use std::time::Duration;

use crate::loader::Assets;
use crate::{Point, SCREEN_WIDTH};

// This is the type stored in the `Assets` hash map
pub(crate) type CachedSprite = (usize, usize, Rc<Vec<u8>>);

/// Frame identifier for managing animations.
#[derive(Debug, Eq, Hash, PartialEq)]
pub(crate) enum Frame {
    Blipjoy1,
    Blipjoy2,

    Ferris1,
    Ferris2,

    Cthulhu1,
    Cthulhu2,

    Player1,
    Player2,

    Shield1,
    // Laser1,
    // Laser2,
}

/// Sprites can be drawn and procedurally generated.
///
/// A `Sprite` owns its pixel data, and cannot be animated. Use a `SpriteRef` if you need
/// animations.
#[derive(Debug)]
pub(crate) struct Sprite {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

/// SpriteRefs can be drawn and animated.
///
/// They reference their pixel data (instead of owning it).
#[derive(Debug)]
pub(crate) struct SpriteRef {
    width: usize,
    height: usize,
    pixels: Rc<Vec<u8>>,
    frame: Frame,
    duration: Duration,
    dt: Duration,
}

/// Drawables can be blitted to the pixel buffer and animated.
pub(crate) trait Drawable {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn pixels(&self) -> &[u8];
}

pub(crate) trait Animation {
    fn animate(&mut self, assets: &Assets, dt: Duration);
}

impl Sprite {
    pub(crate) fn new(assets: &Assets, frame: Frame) -> Sprite {
        let (width, height, pixels) = assets.sprites().get(&frame).unwrap();

        Sprite {
            width: *width,
            height: *height,
            pixels: pixels.to_vec(),
        }
    }
}

impl SpriteRef {
    pub(crate) fn new(assets: &Assets, frame: Frame, duration: Duration) -> SpriteRef {
        let (width, height, pixels) = assets.sprites().get(&frame).unwrap();

        SpriteRef {
            width: *width,
            height: *height,
            pixels: pixels.clone(),
            frame,
            duration,
            dt: Duration::default(),
        }
    }

    pub(crate) fn step_frame(&mut self, assets: &Assets) {
        use Frame::*;

        let assets = assets.sprites();
        let (pixels, frame) = match self.frame {
            Blipjoy1 => (assets.get(&Blipjoy2).unwrap().2.clone(), Blipjoy2),
            Blipjoy2 => (assets.get(&Blipjoy1).unwrap().2.clone(), Blipjoy1),

            Ferris1 => (assets.get(&Ferris2).unwrap().2.clone(), Ferris2),
            Ferris2 => (assets.get(&Ferris1).unwrap().2.clone(), Ferris1),

            Cthulhu1 => (assets.get(&Cthulhu2).unwrap().2.clone(), Cthulhu2),
            Cthulhu2 => (assets.get(&Cthulhu1).unwrap().2.clone(), Cthulhu1),

            Player1 => (assets.get(&Player2).unwrap().2.clone(), Player2),
            Player2 => (assets.get(&Player1).unwrap().2.clone(), Player1),

            // This should not happen, but here we are!
            Shield1 => (assets.get(&Shield1).unwrap().2.clone(), Shield1),
            // Laser1 => (assets.get(&Laser2).unwrap().2.clone(), Laser2),
            // Laser2 => (assets.get(&Laser1).unwrap().2.clone(), Laser1),
        };

        self.pixels = pixels;
        self.frame = frame;
    }
}

impl Drawable for Sprite {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

impl Drawable for SpriteRef {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

impl Animation for SpriteRef {
    fn animate(&mut self, assets: &Assets, dt: Duration) {
        if self.duration.subsec_nanos() == 0 {
            self.step_frame(assets);
        } else {
            self.dt += dt;

            while self.dt >= self.duration {
                self.dt -= self.duration;
                self.step_frame(assets);
            }
        }
    }
}

/// Blit a drawable to the pixel buffer.
pub(crate) fn blit<S>(screen: &mut [u8], dest: &Point, sprite: &S)
where
    S: Drawable,
{
    let pixels = sprite.pixels();
    let width = sprite.width() * 4;

    let mut s = 0;
    for y in 0..sprite.height() {
        let i = dest.x * 4 + dest.y * SCREEN_WIDTH * 4 + y * SCREEN_WIDTH * 4;
        screen[i..i + width].copy_from_slice(&pixels[s..s + width]);
        s += width;
    }
}

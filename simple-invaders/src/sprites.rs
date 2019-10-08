use std::rc::Rc;

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

/// Sprites can be drawn and animated.
#[derive(Debug)]
pub(crate) struct Sprite {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
    frame: Frame,
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
}

/// Drawables can be blitted to the pixel buffer and animated.
pub(crate) trait Drawable {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn pixels(&self) -> &[u8];
    fn update_pixels(&mut self, pixels: Rc<Vec<u8>>);
    fn frame(&self) -> &Frame;
    fn update_frame(&mut self, frame: Frame);

    fn animate(&mut self, assets: &Assets) {
        use Frame::*;

        let assets = assets.sprites();
        let (pixels, frame) = match self.frame() {
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

        self.update_pixels(pixels);
        self.update_frame(frame);
    }
}

impl Sprite {
    pub(crate) fn new(assets: &Assets, frame: Frame) -> Sprite {
        let cached_sprite = assets.sprites().get(&frame).unwrap();
        Sprite {
            width: cached_sprite.0,
            height: cached_sprite.1,
            pixels: cached_sprite.2.to_vec(),
            frame,
        }
    }
}

impl SpriteRef {
    pub(crate) fn new(assets: &Assets, frame: Frame) -> SpriteRef {
        let cached_sprite = assets.sprites().get(&frame).unwrap();
        SpriteRef {
            width: cached_sprite.0,
            height: cached_sprite.1,
            pixels: cached_sprite.2.clone(),
            frame,
        }
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

    fn update_pixels(&mut self, pixels: Rc<Vec<u8>>) {
        self.pixels = pixels.to_vec();
    }

    fn frame(&self) -> &Frame {
        &self.frame
    }

    fn update_frame(&mut self, frame: Frame) {
        self.frame = frame;
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

    fn update_pixels(&mut self, pixels: Rc<Vec<u8>>) {
        self.pixels = pixels;
    }

    fn frame(&self) -> &Frame {
        &self.frame
    }

    fn update_frame(&mut self, frame: Frame) {
        self.frame = frame;
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

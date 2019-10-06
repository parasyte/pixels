use std::rc::Rc;

use crate::loader::Assets;
use crate::{Point, SCREEN_WIDTH};

// This is the type stored in the `Assets` hash map
pub(crate) type CachedSprite = (usize, usize, Rc<Vec<u8>>);

/// Sprites can be drawn and animated.
#[derive(Debug)]
pub(crate) struct Sprite {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
    frame: String,
}

/// SpriteRefs can be drawn and animated.
///
/// They reference their pixel data (instead of owning it).
#[derive(Debug)]
pub(crate) struct SpriteRef {
    width: usize,
    height: usize,
    pixels: Rc<Vec<u8>>,
    frame: String,
}

pub(crate) trait Sprites {
    fn new(assets: &Assets, name: &str) -> Self;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn pixels(&self) -> &[u8];
    fn update_pixels(&mut self, pixels: Rc<Vec<u8>>);
    fn frame(&self) -> &str;
    fn update_frame(&mut self, frame: &str);
}

impl Sprites for Sprite {
    fn new(assets: &Assets, name: &str) -> Sprite {
        let cached_sprite = assets.sprites().get(name).unwrap();
        Sprite {
            width: cached_sprite.0,
            height: cached_sprite.1,
            pixels: cached_sprite.2.to_vec(),
            frame: name.into(),
        }
    }

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
        self.pixels.copy_from_slice(&pixels);
    }

    fn frame(&self) -> &str {
        &self.frame
    }

    fn update_frame(&mut self, frame: &str) {
        self.frame = frame.into();
    }
}

impl Sprites for SpriteRef {
    fn new(assets: &Assets, name: &str) -> SpriteRef {
        let cached_sprite = assets.sprites().get(name).unwrap();
        SpriteRef {
            width: cached_sprite.0,
            height: cached_sprite.1,
            pixels: cached_sprite.2.clone(),
            frame: name.into(),
        }
    }

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

    fn frame(&self) -> &str {
        &self.frame
    }

    fn update_frame(&mut self, frame: &str) {
        self.frame = frame.into();
    }
}

pub(crate) fn blit<S>(screen: &mut [u8], dest: &Point, sprite: &S)
where
    S: Sprites,
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

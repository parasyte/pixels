use crate::loader::Assets;
use crate::TIME_STEP;
use crate::{Point, HEIGHT, WIDTH};
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::time::Duration;

// This is the type stored in the `Assets` hash map
pub(crate) type CachedSprite = (usize, usize, Rc<[u8]>);

/// Frame identifier for managing animations.
#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
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

    Bullet1,
    Bullet2,
    Bullet3,
    Bullet4,
    Bullet5,

    Laser1,
    Laser2,
    Laser3,
    Laser4,
    Laser5,
    Laser6,
    Laser7,
    Laser8,
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
    pixels: Rc<[u8]>,
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
    fn animate(&mut self, assets: &Assets);
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
            pixels: Rc::clone(pixels),
            frame,
            duration,
            dt: Duration::default(),
        }
    }

    pub(crate) fn step_frame(&mut self, assets: &Assets) {
        use Frame::*;

        let assets = assets.sprites();
        let (pixels, frame) = match self.frame {
            Blipjoy1 => (Rc::clone(&assets.get(&Blipjoy2).unwrap().2), Blipjoy2),
            Blipjoy2 => (Rc::clone(&assets.get(&Blipjoy1).unwrap().2), Blipjoy1),

            Ferris1 => (Rc::clone(&assets.get(&Ferris2).unwrap().2), Ferris2),
            Ferris2 => (Rc::clone(&assets.get(&Ferris1).unwrap().2), Ferris1),

            Cthulhu1 => (Rc::clone(&assets.get(&Cthulhu2).unwrap().2), Cthulhu2),
            Cthulhu2 => (Rc::clone(&assets.get(&Cthulhu1).unwrap().2), Cthulhu1),

            Player1 => (Rc::clone(&assets.get(&Player2).unwrap().2), Player2),
            Player2 => (Rc::clone(&assets.get(&Player1).unwrap().2), Player1),

            Bullet1 => (Rc::clone(&assets.get(&Bullet2).unwrap().2), Bullet2),
            Bullet2 => (Rc::clone(&assets.get(&Bullet3).unwrap().2), Bullet3),
            Bullet3 => (Rc::clone(&assets.get(&Bullet4).unwrap().2), Bullet4),
            Bullet4 => (Rc::clone(&assets.get(&Bullet5).unwrap().2), Bullet5),
            Bullet5 => (Rc::clone(&assets.get(&Bullet1).unwrap().2), Bullet1),

            Laser1 => (Rc::clone(&assets.get(&Laser2).unwrap().2), Laser2),
            Laser2 => (Rc::clone(&assets.get(&Laser3).unwrap().2), Laser3),
            Laser3 => (Rc::clone(&assets.get(&Laser4).unwrap().2), Laser4),
            Laser4 => (Rc::clone(&assets.get(&Laser5).unwrap().2), Laser5),
            Laser5 => (Rc::clone(&assets.get(&Laser6).unwrap().2), Laser6),
            Laser6 => (Rc::clone(&assets.get(&Laser7).unwrap().2), Laser7),
            Laser7 => (Rc::clone(&assets.get(&Laser8).unwrap().2), Laser8),
            Laser8 => (Rc::clone(&assets.get(&Laser1).unwrap().2), Laser1),

            _ => unreachable!(),
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
    fn animate(&mut self, assets: &Assets) {
        if self.duration.subsec_nanos() == 0 {
            self.step_frame(assets);
        } else {
            self.dt += TIME_STEP;

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
    assert!(dest.x + sprite.width() <= WIDTH);
    assert!(dest.y + sprite.height() <= HEIGHT);

    let pixels = sprite.pixels();
    let width = sprite.width() * 4;

    let mut s = 0;
    for y in 0..sprite.height() {
        let i = dest.x * 4 + dest.y * WIDTH * 4 + y * WIDTH * 4;

        // Merge pixels from sprite into screen
        let zipped = screen[i..i + width].iter_mut().zip(&pixels[s..s + width]);
        for (left, &right) in zipped {
            if right > 0 {
                *left = right;
            }
        }

        s += width;
    }
}

/// Draw a line to the pixel buffer using Bresenham's algorithm.
pub(crate) fn line(screen: &mut [u8], p1: &Point, p2: &Point, color: [u8; 4]) -> Option<()> {
    let p1 = (p1.x as isize, p1.y as isize);
    let p2 = (p2.x as isize, p2.y as isize);
    let clip_max = (WIDTH as isize - 1, HEIGHT as isize - 1);
    for (x, y) in clipline::Clipline::new((p1, p2), ((0, 0), clip_max))? {
        let (x, y) = (x as usize, y as usize);
        let i = x * 4 + y * WIDTH * 4;

        screen[i..i + 4].copy_from_slice(&color);
    }
    Some(())
}

/// Draw a rectangle to the pixel buffer using two points in opposite corners.
pub(crate) fn rect(screen: &mut [u8], p1: &Point, p2: &Point, color: [u8; 4]) {
    let p2 = Point::new(p2.x - 1, p2.y - 1);
    let p3 = Point::new(p1.x, p2.y);
    let p4 = Point::new(p2.x, p1.y);

    line(screen, p1, &p3, color);
    line(screen, &p3, &p2, color);
    line(screen, &p2, &p4, color);
    line(screen, &p4, p1, color);
}

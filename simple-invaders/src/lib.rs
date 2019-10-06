use std::collections::HashMap;
use std::io::Cursor;
use std::rc::Rc;

type CachedSprite = (usize, usize, Rc<Vec<u8>>);

// Invader positioning
const START: Point = Point::new(24, 60);
const GRID: Point = Point::new(16, 16);

// Screen handling
pub const SCREEN_WIDTH: usize = 224;
pub const SCREEN_HEIGHT: usize = 256;

#[derive(Debug)]
pub struct World {
    invaders: Invaders,
    lasers: Vec<Laser>,
    shields: Vec<Shield>,
    player: Player,
    cannons: Vec<Cannon>,
    score: u32,
    assets: Assets,
    screen: Vec<u8>,
}

/// A list of assets loaded into memory.
#[derive(Debug)]
struct Assets {
    // sounds: TODO
    sprites: HashMap<String, CachedSprite>,
}

/// A tiny position vector
#[derive(Debug, Default, Eq, PartialEq)]
struct Point {
    x: usize,
    y: usize,
}

/// A collection of invaders.
#[derive(Debug)]
struct Invaders {
    grid: Vec<Vec<Option<Invader>>>,
    stepper: Stepper,
    bounds: Bounds,
}

/// Everything you ever wanted to know about Invaders
#[derive(Debug)]
struct Invader {
    sprite: SpriteRef,
    pos: Point,
    score: u32,
}

/// The stepper will linerly walk through the 2D vector of invaders, updating their state along the
/// way.
#[derive(Debug)]
struct Stepper {
    row: usize,
    col: usize,
}

/// Creates a boundary around the live invaders.
///
/// Used for collision detection and minor optimizations.
#[derive(Debug)]
struct Bounds {
    left: usize,
    right: usize,
    bottom: usize,
}

/// The player entity.
#[derive(Debug)]
struct Player {
    sprite: SpriteRef,
    pos: Point,
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
}

/// The cannon entity.
#[derive(Debug)]
struct Cannon {
    sprite: SpriteRef,
    pos: Point,
}

/// Sprites can be drawn and animated.
#[derive(Debug)]
struct Sprite {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
    frame: String,
}

/// SpriteRefs can be drawn and animated.
///
/// They reference their pixel data (instead of owning it).
#[derive(Debug)]
struct SpriteRef {
    width: usize,
    height: usize,
    pixels: Rc<Vec<u8>>,
    frame: String,
}

trait Sprites {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn pixels(&self) -> &[u8];
    fn frame(&self) -> &str;
}

impl World {
    pub fn new() -> World {
        // Load assets first
        let assets = load_assets();

        let invaders = Invaders {
            grid: make_invader_grid(&assets),
            stepper: Stepper::default(),
            bounds: Bounds::default(),
        };
        let player = Player {
            sprite: SpriteRef::new(&assets, "player1"),
            pos: Point::new(80, 216),
        };
        let shields = (0..5)
            .map(|i| Shield {
                sprite: Sprite::new(&assets, "shield"),
                pos: Point::new(i * 45 + 32, 192),
            })
            .collect();

        // Create a screen with the correct size
        let mut screen = Vec::new();
        screen.resize_with(SCREEN_WIDTH * SCREEN_HEIGHT * 4, Default::default);

        World {
            invaders,
            lasers: Vec::new(),
            shields,
            player,
            cannons: Vec::new(),
            score: 0,
            assets,
            screen,
        }
    }

    pub fn update(&mut self) {
        // Update the next invader
        let row = self.invaders.stepper.row;
        let col = self.invaders.stepper.col;

        // Animate the invader
        if let Some(invader) = &mut self.invaders.grid[row][col] {
            invader.sprite.frame = match invader.sprite.frame.as_ref() {
                "blipjoy1" => {
                    invader.sprite.pixels = self.assets.sprites.get("blipjoy2").unwrap().2.clone();
                    "blipjoy2".into()
                }
                "blipjoy2" => {
                    invader.sprite.pixels = self.assets.sprites.get("blipjoy1").unwrap().2.clone();
                    "blipjoy1".into()
                }
                "ferris1" => {
                    invader.sprite.pixels = self.assets.sprites.get("ferris2").unwrap().2.clone();
                    "ferris2".into()
                }
                "ferris2" => {
                    invader.sprite.pixels = self.assets.sprites.get("ferris1").unwrap().2.clone();
                    "ferris1".into()
                }
                _ => unreachable!(),
            };
        }

        // Find the next invader
        self.invaders.stepper.col += 1;
        if self.invaders.stepper.col >= 11 {
            self.invaders.stepper.col = 0;
            if self.invaders.stepper.row == 0 {
                self.invaders.stepper.row = 4;
            } else {
                self.invaders.stepper.row -= 1;
            }
        }
    }

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

        &self.screen
    }

    fn clear(&mut self) {
        for (i, byte) in self.screen.iter_mut().enumerate() {
            *byte = if i % 4 == 3 { 255 } else { 0 };
        }
    }
}

impl Default for World {
    fn default() -> Self {
        World::new()
    }
}

impl Point {
    const fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Mul for Point {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y)
    }
}

impl Default for Stepper {
    fn default() -> Self {
        Self { row: 4, col: 0 }
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            left: START.x,
            right: START.x + 11 * GRID.x,
            bottom: START.y + 5 * GRID.y,
        }
    }
}

impl Sprite {
    fn new(assets: &Assets, name: &str) -> Sprite {
        let cached_sprite = assets.sprites.get(name).unwrap();
        Sprite {
            width: cached_sprite.0,
            height: cached_sprite.1,
            pixels: cached_sprite.2.to_vec(),
            frame: name.into(),
        }
    }
}

impl Sprites for Sprite {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    fn frame(&self) -> &str {
        &self.frame
    }
}

impl SpriteRef {
    fn new(assets: &Assets, name: &str) -> SpriteRef {
        let cached_sprite = assets.sprites.get(name).unwrap();
        SpriteRef {
            width: cached_sprite.0,
            height: cached_sprite.1,
            pixels: cached_sprite.2.clone(),
            frame: name.into(),
        }
    }
}

impl Sprites for SpriteRef {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    fn frame(&self) -> &str {
        &self.frame
    }
}

/// Load all static assets into an `Assets` structure
fn load_assets() -> Assets {
    let mut sprites = HashMap::new();

    sprites.insert(
        "blipjoy1".into(),
        load_pcx(include_bytes!("assets/blipjoy1.pcx")),
    );
    sprites.insert(
        "blipjoy2".into(),
        load_pcx(include_bytes!("assets/blipjoy2.pcx")),
    );
    sprites.insert(
        "ferris1".into(),
        load_pcx(include_bytes!("assets/ferris1.pcx")),
    );
    sprites.insert(
        "ferris2".into(),
        load_pcx(include_bytes!("assets/ferris2.pcx")),
    );
    sprites.insert(
        "player1".into(),
        load_pcx(include_bytes!("assets/player1.pcx")),
    );
    sprites.insert(
        "player2".into(),
        load_pcx(include_bytes!("assets/player2.pcx")),
    );
    sprites.insert(
        "shield".into(),
        load_pcx(include_bytes!("assets/shield.pcx")),
    );
    // sprites.insert("laser1".into(), load_pcx(include_bytes!("assets/laser1.pcx")));
    // sprites.insert("laser2".into(), load_pcx(include_bytes!("assets/laser2.pcx")));

    Assets { sprites }
}

/// Convert PCX data to raw pixels
fn load_pcx(pcx: &[u8]) -> CachedSprite {
    let mut reader = pcx::Reader::new(Cursor::new(pcx)).unwrap();
    let width = reader.width() as usize;
    let height = reader.height() as usize;
    let mut result = Vec::new();

    if reader.is_paletted() {
        // Read the raw pixel data
        let mut buffer = Vec::new();
        buffer.resize_with(width * height, Default::default);
        for y in 0..height {
            let a = y as usize * width;
            let b = a + width;
            reader.next_row_paletted(&mut buffer[a..b]).unwrap();
        }

        // Read the pallete
        let mut palette = Vec::new();
        let palette_length = reader.palette_length().unwrap() as usize;
        palette.resize_with(palette_length * 3, Default::default);
        reader.read_palette(&mut palette).unwrap();

        // Copy to result with an alpha component
        let pixels = buffer
            .into_iter()
            .map(|pal| {
                let i = pal as usize * 3;
                &palette[i..i + 3]
            })
            .flatten()
            .cloned()
            .collect::<Vec<u8>>();
        result.extend_from_slice(&pixels);
    } else {
        for _ in 0..height {
            // Read the raw pixel data
            let mut buffer = Vec::new();
            buffer.resize_with(width * 3, Default::default);
            reader.next_row_rgb(&mut buffer[..]).unwrap();

            // Copy to result with an alpha component
            let pixels = buffer
                .chunks(3)
                .map(|rgb| {
                    let mut rgb = rgb.to_vec();
                    rgb.push(255);
                    rgb
                })
                .flatten()
                .collect::<Vec<u8>>();
            result.extend_from_slice(&pixels);
        }
    }

    (width, height, Rc::new(result))
}

/// Create a grid of invaders.
fn make_invader_grid(assets: &Assets) -> Vec<Vec<Option<Invader>>> {
    const BLIPJOY_OFFSET: Point = Point::new(3, 4);
    const FERRIS_OFFSET: Point = Point::new(3, 5);

    (0..1)
        .map(|y| {
            (0..11)
                .map(|x| {
                    Some(Invader {
                        sprite: SpriteRef::new(assets, "blipjoy1"),
                        pos: START + BLIPJOY_OFFSET + Point::new(x, y) * GRID,
                        score: 10,
                    })
                })
                .collect()
        })
        .chain((1..3).map(|y| {
            (0..11)
                .map(|x| {
                    Some(Invader {
                        sprite: SpriteRef::new(assets, "ferris1"),
                        pos: START + FERRIS_OFFSET + Point::new(x, y) * GRID,
                        score: 10,
                    })
                })
                .collect()
        }))
        .chain((3..5).map(|y| {
            (0..11)
                .map(|x| {
                    Some(Invader {
                        // TODO: Need a third invader
                        sprite: SpriteRef::new(assets, "blipjoy1"),
                        pos: START + BLIPJOY_OFFSET + Point::new(x, y) * GRID,
                        score: 10,
                    })
                })
                .collect()
        }))
        .collect()
}

fn blit<S>(screen: &mut [u8], dest: &Point, sprite: &S)
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

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_pcx() {
        let pixels = load_pcx(include_bytes!("assets/blipjoy1.pcx"));
        let expected = vec![
            0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0,
            255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0,
            0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255,
            255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0,
            0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255,
            0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255,
            0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
            255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255,
            255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255,
            255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255,
            0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0,
            255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255,
        ];

        assert_eq!(pixels.0, 10, "Width differs");
        assert_eq!(pixels.1, 8, "Height differs");
        assert_eq!(Rc::try_unwrap(pixels.2).unwrap(), expected, "Pixels differ");
    }
}

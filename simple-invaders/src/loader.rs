use std::collections::HashMap;
use std::io::Cursor;
use std::rc::Rc;

use crate::sprites::CachedSprite;

/// A list of assets loaded into memory.
#[derive(Debug)]
pub(crate) struct Assets {
    // sounds: TODO
    sprites: HashMap<String, CachedSprite>,
}

impl Assets {
    pub(crate) fn sprites(&self) -> &HashMap<String, CachedSprite> {
        &self.sprites
    }
}

/// Load all static assets into an `Assets` structure
pub(crate) fn load_assets() -> Assets {
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

#[cfg(test)]
mod tests {
    use super::*;

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

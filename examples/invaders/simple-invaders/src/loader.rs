use crate::sprites::{CachedSprite, Frame};
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::vec::Vec;

/// A list of assets loaded into memory.
#[derive(Debug)]
pub(crate) struct Assets {
    // sounds: TODO
    sprites: BTreeMap<Frame, CachedSprite>,
}

impl Assets {
    pub(crate) fn sprites(&self) -> &BTreeMap<Frame, CachedSprite> {
        &self.sprites
    }
}

/// Load all static assets into an `Assets` structure
pub(crate) fn load_assets() -> Assets {
    use Frame::*;

    let mut sprites = BTreeMap::new();

    sprites.insert(Blipjoy1, load_pcx(include_bytes!("assets/blipjoy1.pcx")));
    sprites.insert(Blipjoy2, load_pcx(include_bytes!("assets/blipjoy2.pcx")));

    sprites.insert(Ferris1, load_pcx(include_bytes!("assets/ferris1.pcx")));
    sprites.insert(Ferris2, load_pcx(include_bytes!("assets/ferris2.pcx")));

    sprites.insert(Cthulhu1, load_pcx(include_bytes!("assets/cthulhu1.pcx")));
    sprites.insert(Cthulhu2, load_pcx(include_bytes!("assets/cthulhu2.pcx")));

    sprites.insert(Player1, load_pcx(include_bytes!("assets/player1.pcx")));
    sprites.insert(Player2, load_pcx(include_bytes!("assets/player2.pcx")));

    sprites.insert(Shield1, load_pcx(include_bytes!("assets/shield.pcx")));

    sprites.insert(Bullet1, load_pcx(include_bytes!("assets/bullet1.pcx")));
    sprites.insert(Bullet2, load_pcx(include_bytes!("assets/bullet2.pcx")));
    sprites.insert(Bullet3, load_pcx(include_bytes!("assets/bullet3.pcx")));
    sprites.insert(Bullet4, load_pcx(include_bytes!("assets/bullet4.pcx")));
    sprites.insert(Bullet5, load_pcx(include_bytes!("assets/bullet5.pcx")));

    sprites.insert(Laser1, load_pcx(include_bytes!("assets/laser1.pcx")));
    sprites.insert(Laser2, load_pcx(include_bytes!("assets/laser2.pcx")));
    sprites.insert(Laser3, load_pcx(include_bytes!("assets/laser3.pcx")));
    sprites.insert(Laser4, load_pcx(include_bytes!("assets/laser4.pcx")));
    sprites.insert(Laser5, load_pcx(include_bytes!("assets/laser5.pcx")));
    sprites.insert(Laser6, load_pcx(include_bytes!("assets/laser6.pcx")));
    sprites.insert(Laser7, load_pcx(include_bytes!("assets/laser7.pcx")));
    sprites.insert(Laser8, load_pcx(include_bytes!("assets/laser8.pcx")));

    Assets { sprites }
}

/// Convert PCX data to raw pixels
fn load_pcx(pcx: &[u8]) -> CachedSprite {
    let mut reader = pcx::Reader::new(pcx).unwrap();
    let width = reader.width() as usize;
    let height = reader.height() as usize;
    let mut result = Vec::new();

    if reader.is_paletted() {
        // Read the raw pixel data
        let mut buffer = Vec::new();
        buffer.resize_with(width * height, Default::default);
        for y in 0..height {
            let a = y * width;
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
            .flat_map(|pal| {
                let i = pal as usize * 3;
                &palette[i..i + 3]
            })
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
                .flat_map(|rgb| {
                    let mut rgb = rgb.to_vec();
                    rgb.push(255);
                    rgb
                })
                .collect::<Vec<u8>>();
            result.extend_from_slice(&pixels);
        }
    }

    (width, height, Rc::from(result.as_ref()))
}

#[cfg(test)]
mod tests {
    use alloc::vec;

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
        assert_eq!(pixels.2.to_vec(), expected, "Pixels differ");
    }
}

use crate::geo::Point;
use crate::loader::Assets;
use crate::sprites::{Frame, Sprite};

/// The shield entity.
#[derive(Debug)]
pub(crate) struct Shield {
    // Shield sprite is not referenced because we want to deform it when it gets shot
    pub sprite: Sprite,
    pub pos: Point,
}

impl Shield {
    // New
    pub fn new(assets: &Assets, pos: Point) -> Self {
        let sprite = Sprite::new(assets, Frame::Shield1);

        Shield { sprite, pos }
    }
}

use crate::geo::Point;
use crate::loader::Assets;
use crate::sprites::{Frame, SpriteRef};
use core::time::Duration;

// Player positioning
const PLAYER_START: Point = Point::new(80, 216);

/// The player entity.
#[derive(Debug)]
pub(crate) struct Player {
    pub sprite: SpriteRef,
    pub pos: Point,
    pub dt: Duration,
}

impl Player {
    pub fn new(assets: &Assets) -> Self {
        let sprite = SpriteRef::new(assets, Frame::Player1, Duration::from_millis(100));
        let pos = PLAYER_START;
        let dt = Duration::default();
        Player { sprite, pos, dt }
    }
}

/// Player control inputs.
#[derive(Debug, Default)]
pub struct Controls {
    /// Move the player.
    pub direction: Direction,
    /// Shoot the cannon.
    pub fire: bool,
}

/// The player can only move left or right, but can also be stationary.
#[derive(Debug)]
pub enum Direction {
    /// Do not move the player.
    Still,
    /// Move to the left.
    Left,
    /// Move to the right.
    Right,
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::Still
    }
}

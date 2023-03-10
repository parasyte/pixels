/// Player control inputs.
#[derive(Debug, Default)]
pub struct Controls {
    /// Move the player.
    pub direction: Direction,
    /// Shoot the cannon.
    pub fire: bool,
}

/// The player can only move left or right, but can also be stationary.
#[derive(Debug, Default)]
pub enum Direction {
    /// Do not move the player.
    #[default]
    Still,
    /// Move to the left.
    Left,
    /// Move to the right.
    Right,
}

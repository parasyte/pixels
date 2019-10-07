/// Player control inputs.
pub struct Controls {
    /// Move the player.
    pub direction: Direction,
    /// Shoot the cannon.
    pub fire: bool,
}

/// The player can only move left or right, but can also be stationary.
pub enum Direction {
    /// Do not move the player.
    Still,
    /// Move to the left.
    Left,
    /// Move to the right.
    Right,
}

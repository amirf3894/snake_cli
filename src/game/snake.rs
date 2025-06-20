use std::vec;

use crate::game::init::start;

use super::init;
pub struct SnakeBody {
    pub len: u32,
    pub pieces: Vec<(BodyPieces)>,
}
pub struct BodyPieces {
    pub direction: Direction,
    pub coordinate: (u16, u16),
}
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
fn main_snake() -> Result<(), Box<dyn (std::error::Error)>> {
    let stdout = start()?;
    let vect = vec![1, 3, 4, 5, 6, 6];

    Ok(())
}

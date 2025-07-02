use std::vec;

use serde::{Deserialize, Serialize};

use crate::server::host::PlaygroundChanges;
#[derive(Clone)]
pub struct SnakeBody {
    pub len: usize,
    pub pieces: Vec<(u16, u16)>,
    pub movement_adder: (i16, i16),
}
#[derive(Clone)]
pub struct BodyPieces {
    pub direction: Direction,
    pub coordinate: (u16, u16),
}
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CommandKeys {
    Directions(Direction),
    EatFood,
    Invalid,
    Exit,
    End,
    ChangeSpeed,
    None,
}
impl CommandKeys {
    pub fn is_none(&self) -> bool {
        if let CommandKeys::None = *self {
            return true;
        }
        false
    }
}
impl SnakeBody {
    pub fn change_direction(&mut self, direction: &Direction) {
        let new_movement_adder = match direction {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };
        if new_movement_adder.0 * self.movement_adder.0 != 0
            || new_movement_adder.1 * self.movement_adder.1 != 0
        {
            return;
        }
        self.movement_adder = new_movement_adder;
    }
    pub fn move_forward(&mut self) -> (Vec<(u16, u16)>, PlaygroundChanges) {
        let &previous_head = self.pieces.get(self.len - 1).unwrap();
        let removed_tail = self.pieces.remove(0);
        let new_head = (
            (previous_head.0 as i16 + self.movement_adder.0)
                .try_into()
                .unwrap_or(0),
            (previous_head.1 as i16 + self.movement_adder.1)
                .try_into()
                .unwrap_or(0),
        );
        self.pieces.push(new_head);
        (
            self.pieces.clone(),
            PlaygroundChanges {
                change_to_x: vec![new_head],
                change_to_o: vec![previous_head],
                remove_char: vec![removed_tail],
                add_food: vec![],
            },
        )
    }
    pub fn eat_food(&mut self) {
        //let &head = self.pieces.last().unwrap();
        let tail = self.pieces.get(0).unwrap();

        //let move_vector = self.pieces.get(0).unwrap() - self.pieces.get(1).unwrap();

        self.pieces.insert(0, (tail.0, tail.1));
        self.len += 1;
    }
}

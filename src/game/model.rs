pub struct SnakeBody {
    pub len: usize,
    pub pieces: Vec<(u16, u16)>,
    pub movement_adder: (i16, i16),
}
pub struct BodyPieces {
    pub direction: Direction,
    pub coordinate: (u16, u16),
}
#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
#[derive(Debug)]
pub enum CommandKeys {
    Directions(Direction),
    EatFood,
    Invalid,
    Exit,
    End,
    None,
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
    pub fn move_toward(&mut self) -> ((u16, u16), (u16, u16), (u16, u16)) {
        let &previous_head = self.pieces.get(self.len - 1).unwrap();
        let removed_tail = self.pieces.remove(0);
        let new_head = (
            (previous_head.0 as i16 + self.movement_adder.0)
                .try_into()
                .unwrap(),
            (previous_head.1 as i16 + self.movement_adder.1)
                .try_into()
                .unwrap(),
        );
        self.pieces.push(new_head);
        (new_head, removed_tail, previous_head)
    }
    pub fn eat_food(&mut self) {
        let &head = self.pieces.last().unwrap();
        self.pieces.push((
            (head.0 as i16 + self.movement_adder.0).try_into().unwrap(),
            (head.1 as i16 + self.movement_adder.1).try_into().unwrap(),
        ));
        self.len += 1;
    }
}

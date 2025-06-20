use crate::game::{init::start, model::SnakeBody, snake};
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{self, size},
};
use std::{io::Write, thread::sleep, time::Duration, vec};

pub fn main_snake() -> Result<(), Box<dyn (std::error::Error)>> {
    let mut stdout = start()?;
    let terminal_size = size()?;
    let mut snake = SnakeBody {
        len: 1,
        pieces: vec![(terminal_size.0 / 2, terminal_size.1 / 2)],
        movement_adder: (1, 0),
    };
    loop {
        let removed_tail = snake.move_toward();
        let removed_tail = MoveTo(removed_tail.0, removed_tail.1);
        execute!(stdout, removed_tail)?;
        write!(stdout, "")?;
        sleep(Duration::from_millis(300));
    }
    Ok(())
}

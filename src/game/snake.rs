use crate::game::{
    init::start,
    model::{CommandKeys, Direction, SnakeBody},
};
use crossterm::{
    cursor::MoveTo,
    event::{self, read},
    execute,
    terminal::size,
};
use std::{
    io::{Stdout, Write},
    sync::{Arc, Mutex},
    time::Duration,
    vec,
};
use tokio::{task::spawn_blocking, time::sleep};

pub async fn main_snake() -> Result<(), Box<dyn (std::error::Error)>> {
    let mut stdout = start()?;
    let terminal_size = size()?;
    let snake = Arc::new(Mutex::new(SnakeBody {
        len: 1,
        pieces: vec![(terminal_size.0 / 2, terminal_size.1 / 2)],
        movement_adder: (1, 0),
    }));
    let command = Arc::new(Mutex::new(CommandKeys::None));
    loop {
        tokio::spawn(read_key_to_command(command.clone()));
        if let CommandKeys::Directions(ref direction) = *command.lock().unwrap() {
            snake.clone().lock().unwrap().change_direction(direction);
        }
        show_snake(&mut stdout, snake.clone())?;
        if let CommandKeys::EatFood = *command.lock().unwrap() {
            snake.clone().lock().unwrap().eat_food();
        }
        sleep(Duration::from_millis(200)).await;
    }
}

fn show_snake(
    stdout: &mut Stdout,
    snake: Arc<Mutex<SnakeBody>>,
) -> Result<(), Box<dyn (std::error::Error)>> {
    let (newhead, removed_tail) = snake.lock().unwrap().move_toward();
    let newhead = MoveTo(newhead.0, newhead.1);
    let removed_tail = MoveTo(removed_tail.0, removed_tail.1);
    execute!(stdout, newhead)?;
    write!(stdout, "#")?;
    execute!(stdout, removed_tail)?;
    write!(stdout, " ")?;
    Ok(())
}

async fn read_key_to_command(command: Arc<Mutex<CommandKeys>>) {
    *command.lock().unwrap() = CommandKeys::None;
    loop {
        let key_event = spawn_blocking(|| read().unwrap()).await.unwrap();
        let new_command = match key_event.as_key_press_event().unwrap().code {
            event::KeyCode::Up => CommandKeys::Directions(Direction::Up),
            event::KeyCode::Down => CommandKeys::Directions(Direction::Down),
            event::KeyCode::Right => CommandKeys::Directions(Direction::Right),
            event::KeyCode::Left => CommandKeys::Directions(Direction::Left),
            event::KeyCode::Enter => CommandKeys::EatFood,
            _ => continue,
        };
        *command.lock().unwrap() = new_command;
    }
}

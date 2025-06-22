use crate::game::{
    init::start,
    model::{self, SnakeBody},
    snake,
};
use crossterm::{
    cursor::MoveTo,
    event::{self, read},
    execute,
    terminal::{self, size},
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
    loop {
        let snake_for_move = snake.clone();
        tokio::spawn(change_direction(snake.clone()));
        show_snake(&mut stdout, snake_for_move);
        sleep(Duration::from_millis(200)).await;
    }
    Ok(())
}
async fn change_direction(snake: Arc<Mutex<SnakeBody>>) {
    loop {
        let key_event = spawn_blocking(|| read().unwrap()).await.unwrap();
        let direction = match key_event.as_key_press_event().unwrap().code {
            event::KeyCode::Up => model::Direction::Up,
            event::KeyCode::Down => model::Direction::Down,
            event::KeyCode::Right => model::Direction::Right,
            event::KeyCode::Left => model::Direction::Left,
            event::KeyCode::Enter => {
                snake.lock().unwrap().eat_food();
                continue;
            }
            _ => continue,
        };
        snake.lock().unwrap().change_direction(direction);
    }
}
fn show_snake(stdout: &mut Stdout, snake: Arc<Mutex<SnakeBody>>) {
    let (newhead, removed_tail) = snake.lock().unwrap().move_toward();
    let newhead = MoveTo(newhead.0, newhead.1);
    let removed_tail = MoveTo(removed_tail.0, removed_tail.1);
    execute!(stdout, newhead);
    write!(stdout, "#");
    execute!(stdout, removed_tail);
    write!(stdout, " ");
}

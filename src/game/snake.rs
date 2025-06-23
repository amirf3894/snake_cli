use crate::game::{
    init::{end, print_wall, start},
    model::{CommandKeys, Direction, SnakeBody},
};
use crossterm::{
    cursor::MoveTo,
    event::{self, read},
    execute,
    terminal::{Clear, ClearType, size},
};
use std::{
    io::{Stdout, Write},
    sync::{Arc, Mutex, RwLock},
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
    let mut eat_glag = false;
    let command = Arc::new(RwLock::new(CommandKeys::None));
    loop {
        tokio::spawn(read_key_to_command(command.clone()));
        if let CommandKeys::Directions(ref direction) = *command.read().unwrap() {
            snake.clone().lock().unwrap().change_direction(direction);
            //*command.write().unwrap() = CommandKeys::None;
        }
        let pieces_pos = snake.clone().lock().unwrap().move_forward(&mut eat_glag);
        if let CommandKeys::EatFood = *command.read().unwrap() {
            snake.clone().lock().unwrap().eat_food();
            eat_glag = true;
            //*command.write().unwrap() = CommandKeys::None;
        }
        if let CommandKeys::End = *command.read().unwrap() {
            end()?;
        }
        *command.write().unwrap() = CommandKeys::None;

        display_game(&mut stdout, pieces_pos).await?;
        sleep(Duration::from_millis(200)).await;
    }
}
// async fn move_toward(snake: Arc<Mutex<SnakeBody>>){

// }
async fn display_game(
    stdout: &mut Stdout,
    pieces_pos: Vec<(u16, u16)>,
) -> Result<(), Box<dyn (std::error::Error)>> {
    let len = pieces_pos.len();
    execute!(stdout, Clear(ClearType::All))?;
    print_wall(stdout)?;
    println!("{:?}", pieces_pos);
    for (index, piece) in pieces_pos.iter().enumerate() {
        let piece_position = MoveTo(piece.0, piece.1);
        execute!(stdout, piece_position)?;
        write!(stdout, "{}", if index == len - 1 { "X" } else { "O" })?;
        println!("")
    }
    Ok(())
}
// fn show_snake(
//     stdout: &mut Stdout,
//     snake: Arc<Mutex<SnakeBody>>,
// ) -> Result<(), Box<dyn (std::error::Error)>> {
//     let (new_head, removed_tail, previous_head) = snake.lock().unwrap().move_toward();
//     let new_head = MoveTo(new_head.0, new_head.1);
//     let removed_tail = MoveTo(removed_tail.0, removed_tail.1);
//     let previous_head = MoveTo(previous_head.0, previous_head.1);
//     execute!(stdout, new_head)?;
//     write!(stdout, "X")?;
//     execute!(stdout, previous_head)?;
//     write!(stdout, "O")?;
//     execute!(stdout, removed_tail)?;
//     write!(stdout, " ")?;
//     Ok(())
// }

async fn read_key_to_command(command: Arc<RwLock<CommandKeys>>) {
    loop {
        let key_event = spawn_blocking(|| read().unwrap()).await.unwrap();
        let new_command = match key_event.as_key_press_event().unwrap().code {
            event::KeyCode::Up => CommandKeys::Directions(Direction::Up),
            event::KeyCode::Down => CommandKeys::Directions(Direction::Down),
            event::KeyCode::Right => CommandKeys::Directions(Direction::Right),
            event::KeyCode::Left => CommandKeys::Directions(Direction::Left),
            event::KeyCode::Enter => CommandKeys::EatFood,
            event::KeyCode::Esc => CommandKeys::End,
            _ => continue,
        };
        *command.write().unwrap() = new_command;
    }
}

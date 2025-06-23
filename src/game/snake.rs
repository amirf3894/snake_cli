use crate::game::{
    init::{end, start},
    model::{CommandKeys, Direction, SnakeBody},
};
use crossterm::{
    cursor::MoveTo,
    event::{self, read},
    execute,
    terminal::size,
};
use rand::{random_range, rng, seq::IndexedRandom};
use std::{
    io::{Stdout, Write},
    ops::Deref,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
    vec,
};
use tokio::{task::spawn_blocking, time::sleep};
pub async fn main_snake() -> Result<(), Box<dyn (std::error::Error)>> {
    let mut conversion_vectore = (0, 0);
    let mut playground: [[char; 256]; 256] = [[' '; 256]; 256];
    let mut stdout = start(&mut playground)?;
    let snake = Arc::new(Mutex::new(SnakeBody {
        len: 2,
        pieces: vec![(1, 1), (1, 2)],
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
            // snake.clone().lock().unwrap().eat_food();
            // eat_glag = true;
            add_food(&mut playground);
            //*command.write().unwrap() = CommandKeys::None;
        }
        if let CommandKeys::End = *command.read().unwrap() {
            end()?;
        }
        *command.write().unwrap() = CommandKeys::None;
        display_playground(
            &mut stdout,
            &mut playground,
            pieces_pos,
            &mut conversion_vectore,
        )?;
        // display_game(&mut stdout, pieces_pos).await?;
        sleep(Duration::from_millis(200)).await;
    }
}
// async fn move_toward(snake: Arc<Mutex<SnakeBody>>){

// }
// async fn display_game(
//     stdout: &mut Stdout,
//     pieces_pos: Vec<(u16, u16)>,
// ) -> Result<(), Box<dyn (std::error::Error)>> {
//     let len = pieces_pos.len();
//     execute!(stdout, Clear(ClearType::All))?;
//     print_wall(stdout)?;
//     println!("{:?}", pieces_pos);
//     for (index, piece) in pieces_pos.iter().enumerate() {
//         let piece_position = MoveTo(piece.0, piece.1);
//         execute!(stdout, piece_position)?;
//         write!(stdout, "{}", if index == len - 1 { "X" } else { "O" })?;
//         println!("")
//     }
//     Ok(())
// }
pub fn add_food(playground: &mut [[char; 256]; 256]) {
    let mut x = 0;
    let mut y = 0;
    while playground[x][y] != ' ' {
        x = random_range(1..256);
        y = random_range(1..256);
    }
    let mut rng = rng();
    let &weight = [1, 1, 1, 2, 2, 3].choose(&mut rng).unwrap();
    playground[x][y] = std::char::from_digit(weight, 10).unwrap();
}
fn bind_snake_to_playground(playground: &mut [[char; 256]; 256], pieces_pos: &Vec<(u16, u16)>) {
    for x in 1..256 {
        for y in 1..256 {
            if playground[x][y].is_digit(10) {
                continue;
            }
            playground[x][y] = ' ';
        }
    }
    let len = pieces_pos.len();
    for (index, &(x, y)) in pieces_pos.iter().enumerate() {
        if index == len - 1 {
            playground[x as usize][y as usize] = 'X';
            continue;
        }
        playground[x as usize][y as usize] = 'O';
    }
}
fn display_playground(
    stdout: &mut Stdout,
    playground: &mut [[char; 256]; 256],
    pieces_pos: Vec<(u16, u16)>,
    conversion_vector: &mut (u16, u16),
) -> Result<(), Box<dyn (std::error::Error)>> {
    bind_snake_to_playground(playground, &pieces_pos);
    let snake_head = pieces_pos.get(pieces_pos.len() - 1).unwrap();
    let terminal_size = size()?;
    if snake_head.0.saturating_sub(conversion_vector.0) == 2 {
        *conversion_vector = (conversion_vector.0.saturating_sub(1), conversion_vector.1);
    } else if snake_head.1.saturating_sub(conversion_vector.1) == 2 {
        *conversion_vector = (conversion_vector.0, conversion_vector.1.saturating_sub(1));
    } else if (terminal_size.0 - 1 + conversion_vector.0).saturating_sub(snake_head.0) == 2 {
        *conversion_vector = (conversion_vector.0 + 1, conversion_vector.1);
    } else if (terminal_size.1 - 1 + conversion_vector.1).saturating_sub(snake_head.1) == 2 {
        *conversion_vector = (conversion_vector.0, conversion_vector.1 + 1);
    }

    // if snake_head.0 + 5 > terminal_size.0 + conversion_vector.0
    //     || snake_head.1 + 5 > terminal_size.1 + conversion_vector.1
    //     || snake_head.0 < conversion_vector.0 + 4
    //     || snake_head.1 < conversion_vector.1 + 4
    // {
    //     *conversion_vector = (
    //         (snake_head.0 + 5).saturating_sub(terminal_size.0),
    //         (snake_head.1 + 5).saturating_sub(terminal_size.1),
    //     );
    // }

    for x in 0..terminal_size.0 {
        for y in 0..terminal_size.1 {
            execute!(stdout, MoveTo(x, y))?;
            write!(
                stdout,
                "{}",
                playground[(x + conversion_vector.0) as usize][(y + conversion_vector.1) as usize]
            )?;
        }
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

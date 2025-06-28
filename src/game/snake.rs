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
    process::exit,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
    vec,
};
use tokio::{task::spawn_blocking, time::sleep};
pub async fn main_snake() -> Result<(), Box<dyn (std::error::Error)>> {
    const DEFUALT_DURATION: u64 = 100;
    const FASTER_DURATION: u64 = 50;
    let mut conversion_vectore = (0, 0);

    let mut playground: [[char; 256]; 256] = [[' '; 256]; 256];
    let mut stdout = start(&mut playground)?;
    let mut duration = DEFUALT_DURATION;
    let mut snake_loose_weight = false;
    let snake = Arc::new(Mutex::new(SnakeBody {
        len: 2,
        pieces: vec![(1, 1), (1, 2)],
        movement_adder: (1, 0),
    }));
    let command = Arc::new(RwLock::new(CommandKeys::None));
    tokio::spawn(read_key_to_command(command.clone()));
    loop {
        if let CommandKeys::Directions(ref direction) = *command.read().unwrap() {
            snake.clone().lock().unwrap().change_direction(direction);
            //*command.write().unwrap() = CommandKeys::None;
        }
        let (pieces_pos, _) = snake.clone().lock().unwrap().move_forward();
        snake_status_check(
            &pieces_pos.last().unwrap(),
            &playground,
            snake.clone(),
            &mut stdout,
        )?;

        if let CommandKeys::End = *command.read().unwrap() {
            let mut text = String::new();
            text += "*********************************\n";
            text += "*********************************\n";
            text += "*** PRESS A KEY TWICE TO QUIT ***\n";
            text += "***     HOPE YOU ENJOY :)     ***\n";
            text += "*********************************\n";
            text += "*********************************";
            end(&text, &mut stdout)?;
            exit(0);
        }
        display_playground(
            &mut stdout,
            &mut playground,
            &pieces_pos,
            &mut conversion_vectore,
        )?;
        if let CommandKeys::Faster = *command.read().unwrap() {
            if duration == FASTER_DURATION {
                duration = DEFUALT_DURATION;
                snake_loose_weight = false;
            } else {
                duration = FASTER_DURATION;
                snake_loose_weight = true;
            }
        }
        *command.write().unwrap() = CommandKeys::None;

        if duration == FASTER_DURATION {
            if snake_loose_weight {
                let mut snake_gaurd = snake.lock().unwrap();
                if snake_gaurd.len < 3 {
                    duration = DEFUALT_DURATION
                } else {
                    snake_gaurd.pieces.remove(0);
                    snake_gaurd.len -= 1;
                }
            }
            snake_loose_weight = !snake_loose_weight;
        }

        sleep(Duration::from_millis(duration)).await;
    }
}
fn snake_status_check(
    head: &(u16, u16),
    playground: &[[char; 256]; 256],
    snake: Arc<Mutex<SnakeBody>>,
    stdout: &mut Stdout,
) -> Result<(), Box<dyn (std::error::Error)>> {
    let character = playground[head.0 as usize][head.1 as usize];
    if character == '#' || character == 'O' || character == 'X' {
        let mut text = String::new();
        text += "*********************************\n";
        text += "*********************************\n";
        text += "***         YOU LOOSE         ***\n";
        text += "*** PRESS A KEY TWICE TO QUIT ***\n";
        text += "*********************************\n";
        text += "*********************************";

        end(&text, stdout)?;
        println!("you loose \nscore: {}", snake.lock().unwrap().len);
        exit(0);
    }
    if let Some(n) = character.to_digit(10) {
        (0..n).for_each(|_| snake.lock().unwrap().eat_food());
    }
    Ok(())
}

pub fn add_food(playground: &mut [[char; 256]; 256]) {
    const FOODS: [u32; 15] = [1, 1, 1, 2, 2, 4, 4, 3, 3, 8, 9, 12, 2, 9, 8];

    let mut x = 0;
    let mut y = 0;
    while playground[x][y] != ' ' {
        x = random_range(1..256);
        y = random_range(1..256);
    }
    let mut rng = rng();
    let &weight = FOODS.choose(&mut rng).unwrap();
    playground[x][y] = std::char::from_digit(weight, 16).unwrap();
}
fn update_playground(playground: &mut [[char; 256]; 256], pieces_pos: &Vec<(u16, u16)>) {
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
    pieces_pos: &Vec<(u16, u16)>,
    conversion_vector: &mut (u16, u16),
) -> Result<(), Box<dyn (std::error::Error)>> {
    update_playground(playground, &pieces_pos);
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

pub async fn read_key_to_command(command: Arc<RwLock<CommandKeys>>) {
    //~*command.write().unwrap() = CommandKeys::None
    loop {
        if !command.read().unwrap().is_none() {
            continue;
        }
        //let key_event = read().unwrap();
        let key_event = spawn_blocking(|| read().unwrap()).await.unwrap();
        let new_command = match key_event.as_key_press_event().unwrap().code {
            event::KeyCode::Up | event::KeyCode::Char('w') | event::KeyCode::Char('W') => {
                CommandKeys::Directions(Direction::Up)
            }
            event::KeyCode::Down | event::KeyCode::Char('s') | event::KeyCode::Char('S') => {
                CommandKeys::Directions(Direction::Down)
            }
            event::KeyCode::Right | event::KeyCode::Char('d') | event::KeyCode::Char('D') => {
                CommandKeys::Directions(Direction::Right)
            }
            event::KeyCode::Left | event::KeyCode::Char('a') | event::KeyCode::Char('A') => {
                CommandKeys::Directions(Direction::Left)
            }
            event::KeyCode::Char(' ') => CommandKeys::Faster,
            event::KeyCode::Esc => CommandKeys::End,
            _ => continue,
        };
        *command.write().unwrap() = new_command;
        // return;
    }
}

use common::model::*;
use crossterm::style::Stylize;
use rand::{random_range, rng, seq::IndexedRandom};
use std::sync::{Arc, RwLock};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};

///reads PlaygroundChanges that each spawned client_task sended to channel
/// then submits them on the playground
pub async fn update_playground(
    playground: Arc<RwLock<Box<[Box<[char]>]>>>,
    mut rx: Receiver<PlaygroundChanges>,
) {
    //for avoid longtime locking first i cloned it and submit changes on cloned playground
    let mut cloned_playground = {
        let guard = playground.read().unwrap();
        (*guard).clone()
    };

    loop {
        //read coordinates from PlaygroundChanges which sended to channel
        let playground_changes = rx.recv().await.unwrap();
        let remove_char = playground_changes.remove_char;
        let change_to_x = playground_changes.change_to_x;
        let change_to_o = playground_changes.change_to_o;
        let add_food = playground_changes.add_food;

        //changes each playground cell based on its definition
        remove_char
            .iter()
            .for_each(|&i| cloned_playground[i.0 as usize][i.1 as usize] = ' ');
        change_to_o
            .iter()
            .for_each(|i| cloned_playground[i.0 as usize][i.1 as usize] = 'O');
        change_to_x
            .iter()
            .for_each(|i| cloned_playground[i.0 as usize][i.1 as usize] = 'X');
        add_food
            .iter()
            .for_each(|(i, f)| cloned_playground[i.0 as usize][i.1 as usize] = *f);

        *playground.write().unwrap() = cloned_playground.clone();
    }
}

///This function happens when client can not continue the game for eny reason
/// it removes lost snake pieces from the playground by sending a proper PlaygroundChanges
/// to channel for update_playground function and sends the last data to client
pub async fn loose(
    host_side_data: HostSideData,
    snake: &mut SnakeBody,
    mut socket: TcpStream,
    mut playground_changes: PlaygroundChanges,
    tx: Sender<PlaygroundChanges>,
    err: Result<(), Box<dyn (std::error::Error)>>,
) {
    // host_side_data.status = GameStatus::Dead("YOU LOOSE".to_string());
    playground_changes.remove_char.append(&mut snake.pieces);
    if err.as_ref().unwrap_err().to_string() == "loose" {
        playground_changes.remove_char.pop();
    }
    playground_changes.change_to_o.clear();
    playground_changes.change_to_x.clear();
    tx.send(playground_changes).await.unwrap();
    println!("user left: ({})", err.err().unwrap().to_string());

    socket
        .write(serde_json::to_string(&host_side_data).unwrap().as_bytes())
        .await
        .unwrap();
}

///by calling this function a random food is created and returned
pub fn add_food(playground: &mut Box<[Box<[char]>]>) -> Vec<((u16, u16), char)> {
    const FOODS: [u8; 15] = [1, 1, 1, 1, 2, 2, 3, 3, 5, 5, 5, 7, 8, 8, 9];
    let mut x = 0;
    let mut y = 0;
    let width = playground.len();
    let height = playground[0].len();
    while playground[x][y] != ' ' {
        x = random_range(1..width - 1);
        y = random_range(1..height - 1);
    }
    let mut rng = rng();
    let &food = FOODS.choose(&mut rng).unwrap();
    let food = std::char::from_digit(food as u32, 10).unwrap();
    playground[x][y] = food;
    vec![((x as u16, y as u16), food)]
}

///this function put walls(#) on the playground also fills 1% of map with foods
pub fn start(playground: Arc<RwLock<Box<[Box<[char]>]>>>) {
    let mut cloned_playground = {
        let guard = playground.read().unwrap();
        (*guard).clone()
    };
    let len = (cloned_playground.len(), cloned_playground[0].len());
    for x in 0..len.0 {
        cloned_playground[x][0] = '#';
        cloned_playground[x][len.1 - 1] = '#';
    }
    for y in 0..len.1 {
        cloned_playground[0][y] = '#';
        cloned_playground[len.0 - 1][y] = '#';
    }
    (0..(len.0 * len.1 / 100)).for_each(|_| {
        add_food(&mut cloned_playground);
    });
    *playground.write().unwrap() = cloned_playground;
}

///it checks that if snake crashes with food(make snake bigger and returns OK(())) or an obstacle (returns Err("loose"))

pub fn snake_status_check(
    host_side_data: &mut HostSideData,
    playground: Arc<RwLock<Box<[Box<[char]>]>>>,
    snake: &mut SnakeBody,
    playground_changes: &mut PlaygroundChanges,
) -> Result<(), Box<dyn (std::error::Error)>> {
    let head = playground_changes.change_to_x.get(0).unwrap();
    let character = playground.read().unwrap()[head.0 as usize][head.1 as usize];
    if character == '#' || character == 'O' || character == 'X' {
        host_side_data.status = GameStatus::Dead("YOU LOST :(".to_string());
        Err("loose")?;
    }
    if let Some(n) = character.to_digit(10) {
        (0..n).for_each(|_| snake.eat_food());
        let mut cloned_playground = { (*playground.read().unwrap()).clone() };
        playground_changes.add_food = add_food(&mut cloned_playground);
    }
    Ok(())
}

///it generates a random coordinate for snake head
pub fn generate_head_location(playground_size: (usize, usize)) -> (usize, usize) {
    (
        random_range(1..playground_size.0 - 1),
        random_range(1..playground_size.1 - 1),
    )
}

///creates a String to sent data for client
/// it generates string based on clients terminal size it means by changing the
/// size of client's terminal window it wouldn't crash
/// it is possible by a conversion_vector actually it moves the terminal cells over the map cells
/// its formula is : terminal_cell + conversion_vector = a_cell_on_playground
pub fn user_display_generator(
    playground: Arc<RwLock<Box<[Box<[char]>]>>>,
    snake_head: &(u16, u16),
    conversion_vector: &mut (u16, u16),
    terminal_size: &(u16, u16),
) -> Result<String, Box<dyn (std::error::Error)>> {
    let cloned_playground = playground.read().unwrap();
    let playground_size = (cloned_playground.len(), cloned_playground[0].len());

    //here we have a gap which is 20% of terminal size that is the distance between snake head and terminal edges
    let gap = (terminal_size.0 / 5, terminal_size.1 / 5);

    //checks the distance of snake head and left wall
    if snake_head.0.saturating_sub(conversion_vector.0) < gap.0 {
        conversion_vector.0 = snake_head.0.saturating_sub(gap.0);
    }

    //checks the distance of snake head and up wall
    if snake_head.1.saturating_sub(conversion_vector.1) < gap.1 {
        conversion_vector.1 = snake_head.1.saturating_sub(gap.1);
    }

    //checks the distance of snake head and right wall
    if (terminal_size.0 + conversion_vector.0).saturating_sub(snake_head.0) < gap.0 {
        conversion_vector.0 = snake_head.0.saturating_sub(terminal_size.0 - gap.0);
    }

    //checks the distance of snake head and down wall
    if (terminal_size.1 + conversion_vector.1).saturating_sub(snake_head.1) < gap.1 {
        conversion_vector.1 = snake_head.1.saturating_sub(terminal_size.1 - gap.1);
    }

    //these are for prevent of index overflow for example maybe a moved terminal cell(conversion vector + terminal cell)
    //locates after the defined playground so we have to avoid this behavior
    if terminal_size.0 + conversion_vector.0 > playground_size.0 as u16
        || snake_head.0 + gap.0 > playground_size.0 as u16
    {
        conversion_vector.0 = (playground_size.0 as u16) - terminal_size.0;
    }
    if terminal_size.1 + conversion_vector.1 > playground_size.1 as u16
        || snake_head.1 + gap.1 > playground_size.1 as u16
    {
        conversion_vector.1 = (playground_size.1 as u16) - terminal_size.1;
    }

    let mut data = String::new();
    let y_range = 0..if terminal_size.1 >= playground_size.1 as u16 {
        //if client_terminal size is bigger than playground size
        playground_size.1 as u16
    } else {
        terminal_size.1
    };
    let x_range = 0..if terminal_size.0 >= playground_size.0 as u16 {
        //if client_terminal size is bigger than playground size
        playground_size.0 as u16
    } else {
        terminal_size.0
    };

    y_range.clone().for_each(|y| {
        x_range.clone().for_each(|x| {
            let item = cloned_playground[(x + conversion_vector.0) as usize]
                [(y + conversion_vector.1) as usize];

            let colored_item = match item {
                'X' => {
                    if (x + conversion_vector.0, y + conversion_vector.1) == *snake_head {
                        Some('X'.dark_green())
                        //data.push('Z');
                    } else {
                        Some('X'.magenta())
                    }
                }
                _ => None,
            };
            if let Some(colored) = colored_item {
                data.push_str(colored.to_string().as_str());
            } else {
                data.push(item);
            }
        });

        //without this if the terminal size is larger than playground size then data displayed in chaotic state in client terminal
        (0..terminal_size.0.saturating_sub(playground_size.0 as u16)).for_each(|_| data.push(' '));
    });
    Ok(data)
}

use crate::game::{
    self,
    model::{self, CommandKeys, SnakeBody},
    snake::{self},
};
use clap::{self};
use crossterm::style::Stylize;
use rand::{rand_core::le, random_range, rng, seq::IndexedRandom};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    error::Error,
    process::exit,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
    vec,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    join,
    net::{TcpListener, TcpStream},
    runtime::Runtime,
    sync::mpsc::{self, Receiver, Sender},
    time::sleep,
};
#[derive(Clone)]
pub struct PlaygroundChanges {
    pub change_to_x: Vec<(u16, u16)>,
    pub change_to_o: Vec<(u16, u16)>,
    pub remove_char: Vec<(u16, u16)>,
    pub add_food: Vec<((u16, u16), char)>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ClientSendData {
    pub terminal_size: (u16, u16),
    pub command: CommandKeys,
    pub loose_weight: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct HostSideData {
    pub display_data: String,
    pub status: GameStatus,
    pub len: usize,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum GameStatus {
    Dead(String),
    Alive,
}
pub async fn main_host(
    playground_size: (u16, u16),
    addr: &str,
) -> Result<(), Box<dyn (std::error::Error)>> {
    let (tx, rx) = mpsc::channel::<PlaygroundChanges>(300);
    let playground = Arc::new(RwLock::new(
        vec![vec![' '; playground_size.1 as usize].into_boxed_slice(); playground_size.0 as usize]
            .into_boxed_slice(),
    ));
    start(playground.clone());
    let listener = TcpListener::bind(addr).await?;
    let async_playground = playground.clone(); //let async_playground = playground.clone();
    tokio::spawn(async move {
        loop {
            //println!("inside loop");
            let thread_playground = async_playground.clone();
            let (mut socket, _) = listener.accept().await.unwrap();
            let thread_tx = tx.clone();
            println!("socket detected");
            //let thread_playground = async_playground.clone();
            tokio::task::spawn_blocking(move || {
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    let mut playground_changes = PlaygroundChanges {
                        change_to_x: vec![],
                        change_to_o: vec![],
                        remove_char: vec![],
                        add_food: vec![],
                    };
                    let mut host_side_data = HostSideData {
                        display_data: "".to_string(),
                        status: GameStatus::Alive,
                        len: 2,
                    };
                    let mut snake = SnakeBody {
                        len: 2,
                        pieces: vec![],
                        movement_adder: (-1, 0),
                    };
                    let err = clinet_tasks(
                        &mut snake,
                        &mut host_side_data,
                        &mut playground_changes,
                        thread_tx.clone(),
                        &mut socket,
                        thread_playground,
                        &playground_size,
                    )
                    .await;
                    loose(
                        host_side_data,
                        &mut snake,
                        socket,
                        playground_changes,
                        thread_tx,
                        err,
                    )
                    .await;
                });
            });

            //a.join();
        }
    });

    update_playground(playground, rx, &playground_size).await;
    Ok(())
}
//pub async fn wait_for_connect()
pub async fn clinet_tasks(
    snake: &mut SnakeBody,
    host_side_data: &mut HostSideData,
    playground_changes: &mut PlaygroundChanges,
    tx: Sender<PlaygroundChanges>,
    socket: &mut TcpStream,
    playground: Arc<RwLock<Box<[Box<[char]>]>>>,
    playground_size: &(u16, u16),
) -> Result<(), Box<dyn (std::error::Error)>> {
    let mut buf;
    println!("a user entered");
    let mut conversion_vector = (0, 0);
    let movement_adder = (-1, 0);
    let mut head_pos = (0, 0);
    //let len = socket.read(&mut buf).await?;
    // let mut command =
    //     serde_json::from_str::<ClientSendData>(&String::from_utf8_lossy(&buf[..len]).to_string())?;
    //socket.write_u8(1).await?;
    let mut tail_pos = (
        (head_pos.0 as i16 - movement_adder.0) as usize,
        (head_pos.1 as i16 - movement_adder.1) as usize,
    );
    {
        let readable_playground = playground.read().unwrap();
        while readable_playground[head_pos.0][head_pos.1] != ' '
            || readable_playground[tail_pos.0][tail_pos.1] != ' '
        {
            head_pos =
                generate_head_location((playground_size.0 as usize, playground_size.1 as usize));
            tail_pos = (
                (head_pos.0 as i16 - movement_adder.0) as usize,
                (head_pos.1 as i16 - movement_adder.1) as usize,
            );
        }
    }
    //let terminal_size = command.terminal_size;

    *snake = SnakeBody {
        len: 2,
        pieces: vec![
            (tail_pos.0 as u16, tail_pos.1 as u16),
            (head_pos.0 as u16, head_pos.1 as u16),
        ],
        movement_adder,
    };
    let mut client_side_data;
    loop {
        //let wait_handler = tokio::spawn(sleep(Duration::from_millis(1200)));
        // let mut snake = SnakeBody{
        //     len : 2,
        //     pieces: vec![()]

        // }
        buf = [0_u8; 500];
        let len = socket.read(&mut buf).await?;
        client_side_data =
            serde_json::from_str::<ClientSendData>(&String::from_utf8_lossy(&buf[..len]))?;

        // sleep(Duration::from_secs(1)).await;
        let command = client_side_data.command;
        //println!("{:?}", command);
        let terminal_size = client_side_data.terminal_size;
        //println!("{:?}", command);
        if let CommandKeys::Directions(direction) = command {
            snake.change_direction(&direction);
        }
        if let CommandKeys::End = command {
            host_side_data.status = GameStatus::Dead("YOU LEFT, HOPE YOU ENJOY THIS".to_string());
            Err("user pressed escape")?;
        }
        *playground_changes = snake.move_forward().1;
        if client_side_data.loose_weight {
            playground_changes.remove_char.push(snake.pieces.remove(0));
            snake.len -= 1;
        }
        let display_data = user_display_generator(
            playground.clone(),
            &playground_changes.change_to_o.get(0).unwrap(),
            &mut conversion_vector,
            &terminal_size,
        )?;
        //tx.send(pieces_pos).await?;
        *host_side_data = HostSideData {
            display_data,
            status: GameStatus::Alive,
            len: snake.len,
        };
        //println!("hooooy");
        // println!("{:#?}", host_side_data);
        snake_status_check(
            host_side_data,
            playground.clone(),
            snake,
            playground_changes,
        )?;
        socket
            .write(serde_json::to_string(&host_side_data)?.as_bytes())
            .await?;

        let async_tx = tx.clone();
        let cloned_playground_changes = playground_changes.clone();
        let mpsc_handler =
            tokio::spawn(async move { async_tx.send(cloned_playground_changes).await });
        //let async_tx = tx.clone();
        //let user_screen = println!("{}", String::from_utf8_lossy(&buf[..len]));
        let _ = join!(mpsc_handler);
    }
}
fn user_display_generator(
    playground: Arc<RwLock<Box<[Box<[char]>]>>>,
    snake_head: &(u16, u16),
    conversion_vector: &mut (u16, u16),
    terminal_size: &(u16, u16),
) -> Result<String, Box<dyn (std::error::Error)>> {
    // let cloned_playground = {
    //     let gaurd = playground.read().unwrap();
    //     (*gaurd).clone()
    // };
    let cloned_playground = playground.read().unwrap();
    let playground_len = (cloned_playground.len(), cloned_playground[0].len());
    let gap = (terminal_size.0 / 5, terminal_size.1 / 5);
    //let snake_head = pieces_pos.last().unwrap();
    if snake_head.0.saturating_sub(conversion_vector.0) < gap.0 {
        conversion_vector.0 = snake_head.0.saturating_sub(gap.0);
    }
    if snake_head.1.saturating_sub(conversion_vector.1) < gap.1 {
        conversion_vector.1 = snake_head.1.saturating_sub(gap.1);
    }

    if (terminal_size.0 + conversion_vector.0).saturating_sub(snake_head.0) < gap.0 {
        conversion_vector.0 = snake_head.0.saturating_sub(terminal_size.0 - gap.0);
        if snake_head.0 + gap.0 > playground_len.0 as u16 {
            conversion_vector.0 = (playground_len.0 as u16) - terminal_size.0;
        }
    }
    if (terminal_size.1 + conversion_vector.1).saturating_sub(snake_head.1) < gap.1 {
        conversion_vector.1 = snake_head.1.saturating_sub(terminal_size.1 - gap.1);
        if snake_head.1 + gap.1 > playground_len.1 as u16 {
            conversion_vector.1 = (playground_len.1 as u16) - terminal_size.1;
        }
    }

    // if snake_head.0.saturating_sub(conversion_vector.0) == 2 {
    //     *conversion_vector = (conversion_vector.0.saturating_sub(1), conversion_vector.1);
    // } else if snake_head.1.saturating_sub(conversion_vector.1) == 2 {
    //     *conversion_vector = (conversion_vector.0, conversion_vector.1.saturating_sub(1));
    // } else if (terminal_size.0 - 1 + conversion_vector.0).saturating_sub(snake_head.0) == 2 {
    //     *conversion_vector = (conversion_vector.0 + 1, conversion_vector.1);
    // } else if (terminal_size.1 - 1 + conversion_vector.1).saturating_sub(snake_head.1) == 2 {
    //     *conversion_vector = (conversion_vector.0, conversion_vector.1 + 1);
    // }
    // let mut data = String::new();
    // for y in 0..terminal_size.1 {
    //     for x in 0..terminal_size.0 {
    //         data.push(
    //             cloned_playground[(x + conversion_vector.0) as usize]
    //                 [(y + conversion_vector.1) as usize],
    //         );
    //     }
    //     //data.push('\n');
    // }

    // let mut data = [0_u8; 5000];
    // let mut index = 0;
    //println!("{:?}", conversion_vector);

    let mut data = String::new();
    (0..if terminal_size.1 > playground_len.1 as u16 {
        playground_len.1 as u16
    } else {
        terminal_size.1
    })
        .for_each(|y| {
            (0..if terminal_size.0 > playground_len.0 as u16 {
                playground_len.0 as u16
            } else {
                terminal_size.0
            })
                .for_each(|x| {
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
                        '#' => Some('#'.dark_red()),
                        other => {
                            if other.is_digit(10) {
                                Some(other.cyan())
                            } else {
                                None
                            }
                        }
                    };
                    if let Some(colored) = colored_item {
                        data.push_str(colored.to_string().as_str());
                    } else {
                        data.push(item);
                    }
                    // data.push_str(item.to_string().as_str());
                });
            (0..terminal_size.0.saturating_sub(playground_len.0 as u16))
                .for_each(|_| data.push(' ')); //without this if the termial size is larger than playground size then it displays chaotic in client terminal
        });

    // [..terminal_size.1].iter().for_each(|y| [..terminal_size.0].fore);
    //*playground.write().unwrap() = cloned_playground;
    //println!("{:?}", pieces_pos.last().unwrap());
    // println!("{data}");
    Ok(data)
}
fn generate_head_location(playground_size: (usize, usize)) -> (usize, usize) {
    (
        random_range(1..playground_size.0 - 1),
        random_range(1..playground_size.1 - 1),
    )
}
fn snake_status_check(
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
        //playground_changes.*playground.write().unwrap() = cloned_playground;
        //println!("added");
    }

    Ok(())
}
fn start(playground: Arc<RwLock<Box<[Box<[char]>]>>>) {
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
// fn loose(playground_changes: &mut PlaygroundChanges, snake: SnakeBody) {
//     playground_changes.remove_char = snake.pieces;
// }

fn add_food(playground: &mut Box<[Box<[char]>]>) -> Vec<((u16, u16), char)> {
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

async fn update_playground(
    playground: Arc<RwLock<Box<[Box<[char]>]>>>,
    mut rx: Receiver<PlaygroundChanges>,
    playground_size: &(u16, u16),
) {
    let mut cloned_playground = {
        let guard = playground.read().unwrap();
        (*guard).clone()
    };
    //let mut playground = playground.write().unwrap();
    //let (width, height) = (playground_size.0 as usize, playground_size.1 as usize);
    loop {
        let playground_changes = rx.recv().await.unwrap();
        let remove_char = playground_changes.remove_char;
        let change_to_x = playground_changes.change_to_x;
        let change_to_o = playground_changes.change_to_o;
        let add_food = playground_changes.add_food;
        //println!("recieved from channel");
        // for x in 1..width - 1 {
        //     for y in 1..height - 1 {
        //         if cloned_playground[x][y].is_digit(10) {
        //             continue;
        //         }
        //         cloned_playground[x][y] = ' ';
        //     }
        // }
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
        // let len = pieces_pos.len();
        // for (index, &(x, y)) in pieces_pos.iter().enumerate() {
        //     if index == len - 1 {
        //         cloned_playground[x as usize][y as usize] = 'X';
        //         continue;
        //     }
        //     cloned_playground[x as usize][y as usize] = 'O';
        // }
        *playground.write().unwrap() = cloned_playground.clone();
    }
}

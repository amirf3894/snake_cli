use crate::game::{
    model::{self, CommandKeys, SnakeBody},
    snake::{self},
};
use clap::{self};
use rand::random_range;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    collections::btree_set::Difference,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
    vec,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    runtime::Runtime,
    sync::mpsc::{self, Receiver, Sender},
    time::{error::Error, sleep},
};
#[derive(Serialize, Deserialize)]
struct UserInputData {
    terminal_size: (u16, u16),
    command: CommandKeys,
}
#[derive(Serialize, Deserialize)]
struct UserOutputData {
    display_data: String,
    status: String,
}
pub async fn main_host(size: (u16, u16), addr: &str) -> Result<(), Box<dyn (std::error::Error)>> {
    let (tx, rx) = mpsc::channel::<Vec<(u16, u16)>>(300);
    let mut playground = Arc::new(RwLock::new(
        vec![vec![' '; size.1 as usize].into_boxed_slice(); size.0 as usize].into_boxed_slice(),
    ));
    let listener = TcpListener::bind(addr).await?;
    let async_playground = playground.clone(); //let async_playground = playground.clone();
    let check_new_user_handler = tokio::spawn(async move {
        loop {
            //println!("inside loop");
            let thread_playground = async_playground.clone();
            let (socket, _) = listener.accept().await.unwrap();
            println!("socket detected");
            let thread_tx = tx.clone();
            //let thread_playground = async_playground.clone();
            thread::spawn(move || {
                let rt = Runtime::new().unwrap();
                rt.block_on(async { clinet_tasks(thread_tx, socket, thread_playground).await })
                    .unwrap();
            });

            //a.join();
        }
    });

    update_playground(playground, rx).await;
    Ok(())
}
async fn update_playground(
    playground: Arc<RwLock<Box<[Box<[char]>]>>>,
    mut rx: Receiver<Vec<(u16, u16)>>,
) {
    let mut playground = playground.write().unwrap();
    let (width, height) = (playground.len(), playground[0].len());

    loop {
        let pieces_pos = rx.recv().await.unwrap();
        for x in 1..width {
            for y in 1..height {
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
}
//pub async fn wait_for_connect()
pub async fn clinet_tasks(
    tx: Sender<Vec<(u16, u16)>>,
    mut socket: TcpStream,
    playground: Arc<RwLock<Box<[Box<[char]>]>>>,
) -> Result<(), Box<dyn (std::error::Error)>> {
    println!("a user entered");
    let mut conversion_vector = (0, 0);
    let mut buf = [0_u8; 500];
    let readable_playground = playground.read().unwrap();
    let playground_size = (readable_playground.len(), readable_playground[0].len());
    let movement_adder = (-1, 0);
    let mut head_pos = (0, 0);
    let mut tail_pos = (
        (head_pos.0 as i16 - movement_adder.0) as usize,
        (head_pos.1 as i16 - movement_adder.1) as usize,
    );
    while readable_playground[head_pos.0][head_pos.1] != ' '
        || readable_playground[tail_pos.0][tail_pos.1] != ' '
    {
        head_pos = generate_head_location(playground_size);
        tail_pos = (
            (head_pos.0 as i16 - movement_adder.0) as usize,
            (head_pos.1 as i16 - movement_adder.1) as usize,
        );
    }
    let mut snake = SnakeBody {
        len: 2,
        pieces: vec![
            (tail_pos.0 as u16, tail_pos.1 as u16),
            (head_pos.0 as u16, head_pos.1 as u16),
        ],
        movement_adder,
    };

    loop {
        let wait_handler = tokio::spawn(sleep(Duration::from_millis(200)));
        // let mut snake = SnakeBody{
        //     len : 2,
        //     pieces: vec![()]

        // }
        let len = socket.read(&mut buf).await?;
        let recieved_data = String::from_utf8_lossy(&buf);
        let recieved_data = serde_json::from_str::<UserInputData>(&recieved_data)?;
        let command = recieved_data.command;
        let terminal_size = recieved_data.terminal_size;
        if let CommandKeys::Directions(direction) = command {
            snake.change_direction(&direction);
        }
        let pieces_pos = snake.move_forward();
        if let Err(e) =
            snake_status_check(&pieces_pos.last().unwrap(), playground.clone(), &mut snake)
        {
            println!("{}", e.to_string());
            break;
        }
        let display_data = user_display_generator(
            playground.clone(),
            &pieces_pos,
            &mut conversion_vector,
            &terminal_size,
        )?;
        tx.send(pieces_pos).await?;
        let data_send = serde_json::to_string(&UserOutputData {
            display_data,
            status: "nothing".to_string(),
        })?;
        socket.write(data_send.as_bytes()).await;

        //let async_tx = tx.clone();
        //let user_screen = println!("{}", String::from_utf8_lossy(&buf[..len]));
        wait_handler.await;
    }

    Ok(())
}
fn user_display_generator(
    playground: Arc<RwLock<Box<[Box<[char]>]>>>,
    pieces_pos: &Vec<(u16, u16)>,
    conversion_vector: &mut (u16, u16),
    terminal_size: &(u16, u16),
) -> Result<String, Box<dyn (std::error::Error)>> {
    let playground = playground.read().unwrap();
    let snake_head = pieces_pos.last().unwrap();
    if snake_head.0.saturating_sub(conversion_vector.0) == 2 {
        *conversion_vector = (conversion_vector.0.saturating_sub(1), conversion_vector.1);
    } else if snake_head.1.saturating_sub(conversion_vector.1) == 2 {
        *conversion_vector = (conversion_vector.0, conversion_vector.1.saturating_sub(1));
    } else if (terminal_size.0 - 1 + conversion_vector.0).saturating_sub(snake_head.0) == 2 {
        *conversion_vector = (conversion_vector.0 + 1, conversion_vector.1);
    } else if (terminal_size.1 - 1 + conversion_vector.1).saturating_sub(snake_head.1) == 2 {
        *conversion_vector = (conversion_vector.0, conversion_vector.1 + 1);
    }
    let mut data = String::with_capacity(5000);
    for x in 0..terminal_size.0 {
        for y in 0..terminal_size.1 {
            data.push(
                playground[(x + conversion_vector.0) as usize][(y + conversion_vector.1) as usize],
            );
            data.push('\n');
        }
    }
    Ok(data)
}
fn generate_head_location(playground_size: (usize, usize)) -> (usize, usize) {
    (
        random_range(1..playground_size.0 - 1),
        random_range(1..playground_size.1 - 1),
    )
}
fn snake_status_check(
    head: &(u16, u16),
    playground: Arc<RwLock<Box<[Box<[char]>]>>>,
    snake: &mut SnakeBody,
) -> Result<(), Box<dyn (std::error::Error)>> {
    let character = playground.read().unwrap()[head.0 as usize][head.1 as usize];
    if character == '#' || character == 'O' || character == 'X' {
        Err("loose")?;
    }
    if let Some(n) = character.to_digit(10) {
        (0..n).for_each(|_| snake.eat_food());
    }

    Ok(())
}

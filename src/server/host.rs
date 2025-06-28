use crate::game::{model::{self, CommandKeys, SnakeBody}, snake::{self, snake_status_check}};
use clap::{self};
use rand::random_range;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    sync::{Arc, RwLock},
    thread, vec,
};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    runtime::Runtime,
    sync::mpsc::{self, Sender},
};
pub async fn main_host(size: (u16, u16), addr: &str) -> Result<(), Box<dyn (std::error::Error)>> {
    let (tx, rx) = mpsc::channel::<Vec<(u16, u16)>>(300);
    let mut playground = Arc::new(RwLock::new(
        vec![vec![' '; size.1 as usize].into_boxed_slice(); size.0 as usize].into_boxed_slice(),
    ));
    let listener = TcpListener::bind(addr).await?;
    //let async_playground = playground.clone();
    let hanler = tokio::spawn(async move {
        loop {
            //println!("inside loop");
            let thread_playground = playground.clone();
            let (socket, _) = listener.accept().await.unwrap();
            println!("socket detedted");
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
    hanler.await?;
    Ok(())
}
//pub async fn wait_for_connect()
pub async fn clinet_tasks(
    tx: Sender<Vec<(u16, u16)>>,
    mut socket: TcpStream,
    playground: Arc<RwLock<Box<[Box<[char]>]>>>,
) -> Result<(), Box<dyn (std::error::Error)>> {
    println!("a user entered");
    let mut buf = [0_u8; 500];
    let readable_playground = playground.read()?;
    let playground_size = (readable_playground.len(), readable_playground[0].len());
    let movement_adder = (-1, 0);
    let mut head_pos = (0, 0);
    while readable_playground[head_pos.0][head_pos.1] != ' ' {
        head_pos = generate_head_location(playground_size);
    }
    let mut snake = SnakeBody {
        len: 2,
        pieces: vec![
            (head_pos.0 as u16, head_pos.1 as u16),
            (
                (head_pos.0 as i16 + movement_adder.0) as u16,
                (head_pos.0 as i16 + movement_adder.1) as u16,
            ),
        ],
        movement_adder,
    };

    loop {
        // let mut snake = SnakeBody{
        //     len : 2,
        //     pieces: vec![()]

        // }
        let len = socket.read(&mut buf).await?;
        let command = String::from_utf8_lossy(&buf);
        let command = serde_json::from_str::<CommandKeys>(&command)?;
        if let CommandKeys::Directions(direction) = command {
            snake.change_direction(&direction);
        }
        let pieces_pos = snake.move_forward();
        snake_status_check(&pieces_pos.last().unwrap(), &readable_playground, snake, stdout)

        tx.send(command);
        println!("{}", String::from_utf8_lossy(&buf[..len]));
    }

    // let duration = 200;
    //let time_handle = tokio::spawn(sleep(Duration::from_millis(duration)));
    // let mut buf = vec![];
    //let listener = TcpListener::bind(addr)
    // loop {
    //     socket.read_to_end(&mut buf).await?;
    //     //time_handle.await?;
    //     println!("{}", String::from_utf8_lossy(&buf));
    // }
    Ok(())
}
fn generate_head_location(playground_size: (usize, usize)) -> (usize, usize) {
    (
        random_range(1..playground_size.0 - 1),
        random_range(1..playground_size.1 - 1),
    )
}
fn snake_status_check(head: &(u16, u16),
playground: Arc<RwLock<Box<[Box<[char]>]>>>,
snake: & mut SnakeBody,
) -> Result<(), Box<dyn (std::error::Error)>>{
    let character = playground.read().unwrap()[head.0 as usize][head.1 as usize];
    if character == '#' || character == 'O' || character == 'X' {
        Err("loose")?;
    }
    if let Some(n) = character.to_digit(10){
        (0..n).for_each(|_|snake.eat_food());
    }

Ok(())
}

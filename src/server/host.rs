use super::functions::*;
use crate::model::*;
use serde_json;
use std::sync::{Arc, RwLock};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    join,
    net::{TcpListener, TcpStream},
    runtime::Runtime,
    sync::mpsc::{Sender, channel},
};
/*main logic in host_side:
first it trys to creat a tcplistener then it run a async task to wait for new clients and perform theit task
clients send commands and host read them and if it contains some datas that affect on playground, will send
to a channel for update_playground so it make changes on playground and then clients read playgprund data(
playground is Arc<RwLock> so all clients can read and update_playground write changes on it
*/

/// creats a tcplistener to recive data from clients and if a client arrivs it spawns a special task
/// to handle each client
pub async fn main_host(
    playground_size: (u16, u16),
    addr: &str,
) -> Result<(), Box<dyn (std::error::Error)>> {
    let (tx, rx) = channel::<PlaygroundChanges>(300);
    let playground = Arc::new(RwLock::new(
        vec![vec![' '; playground_size.1 as usize].into_boxed_slice(); playground_size.0 as usize]
            .into_boxed_slice(),
    ));
    start(playground.clone()); //initialize the map

    let listener = TcpListener::bind(addr).await?;
    println!("server is visible in: {}", addr);
    let async_playground = playground.clone();

    //check for new clients and run a special task to handle them
    tokio::spawn(async move {
        loop {
            let thread_playground = async_playground.clone();
            let (mut socket, _) = listener.accept().await.unwrap();
            let thread_tx = tx.clone();
            println!("socket detected");

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

                    //runs client_task untill client leave, loose or an error occours
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
        }
    });

    //each client task send PlaygroundChanges to a channel and it recives them and changes game map
    update_playground(playground, rx).await;
    Ok(())
}

///reads client data and make a string from map and send it back
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
        buf = [0_u8; 500];
        let len = socket.read(&mut buf).await?;
        client_side_data =
            serde_json::from_str::<ClientSendData>(&String::from_utf8_lossy(&buf[..len]))?;
        let command = client_side_data.command;
        let terminal_size = client_side_data.terminal_size;
        if let CommandKeys::Directions(direction) = command {
            snake.change_direction(&direction);
        }
        if let CommandKeys::End = command {
            host_side_data.status = GameStatus::Dead("YOU LEFT\n HOPE YOU ENJOY THIS".to_string());
            Err("user pressed escape")?;
        }
        *playground_changes = snake.move_forward();
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
        *host_side_data = HostSideData {
            display_data,
            status: GameStatus::Alive,
            len: snake.len,
        };
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
        let _ = join!(mpsc_handler);
    }
}

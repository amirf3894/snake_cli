use crate::{
    game::{
        self,
        init::{self, end},
        model::{CommandKeys, Direction},
    },
    server::host::{ClientSendData, GameStatus, HostSideData},
};
use clap::builder::Str;
use crossterm::{
    self,
    cursor::{self, MoveTo},
    event::{KeyCode, KeyEvent, read},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, enable_raw_mode, size},
};
use std::{
    io::{Write, stdout},
    process::exit,
    sync::{Arc, RwLock, mpsc::Sender},
    thread::sleep,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Interest},
    net::TcpStream,
};
pub async fn main_client(name: &str, addr: &str) -> Result<(), Box<dyn (std::error::Error)>> {
    const FASTER_DURATION: u64 = 50;

    const SLOWER_DURATION: u64 = 200;
    let mut loose_weight = false;
    let mut duration = SLOWER_DURATION;
    let mut buff = [0_u8; 20_000];
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let mut stream = TcpStream::connect(addr)
        .await
        .map_err(|_| "COULDN'T CONNECT TO SERVER!")?;
    // let data = serde_json::to_string(&ClientSendData {
    //     terminal_size: size().unwrap(),
    //     command: CommandKeys::None,
    // })?;
    //stream.write(data.as_bytes()).await?;
    // let _ = stream.read_u8().await;
    let (tx, rx) = std::sync::mpsc::channel::<CommandKeys>();
    enable_raw_mode()?;
    // let command = Arc::new(RwLock::new(CommandKeys::None));
    tokio::spawn(read_key_to_command(tx));
    let mut host_side_data = HostSideData {
        display_data: "".to_string(),
        status: GameStatus::Alive,
        len: 2,
    };
    let mut client_side_data;
    loop {
        sleep(Duration::from_millis(duration));
        let mut command = rx.try_recv().unwrap_or(CommandKeys::None);
        if let CommandKeys::ChangeSpeed = command {
            command = CommandKeys::None;
            if duration == FASTER_DURATION {
                duration = SLOWER_DURATION;
                loose_weight = false;
            } else {
                duration = FASTER_DURATION;
                loose_weight = true;
            }
        }
        if duration == FASTER_DURATION {
            if host_side_data.len < 3 {
                duration = SLOWER_DURATION;
            }
            loose_weight = !loose_weight;
        }
        //print!("{:?}", command);
        //let terminal_size = size()?;

        // let command_to_send = {
        //     let gurad = command.read().unwrap();
        //     (*gurad).clone()
        // };
        // *command.write().unwrap() = CommandKeys::None;
        client_side_data = ClientSendData {
            terminal_size: size()?,
            command,
            loose_weight,
        };
        stream
            .write(serde_json::to_string(&client_side_data)?.as_bytes())
            .await
            .map_err(|_| "COULDN'T WRITE TO SERVER!")?;
        let len = stream
            .read(&mut buff)
            .await
            .map_err(|_| "COULDN't READ FROM SERVER!")?;
        //println!("{:#?}", String::from_utf8_lossy(&buff[..len]));
        host_side_data =
            serde_json::from_str::<HostSideData>(&String::from_utf8_lossy(&buff[..len]))
                .map_err(|_| "COULDN't READ FROM SERVER!")?;

        execute!(stdout, MoveTo(0, 0),)?;
        write!(stdout, "{}", host_side_data.display_data)?;

        stdout.flush()?;
        if let GameStatus::Dead(msg) = host_side_data.status {
            // execute!(stdout, MoveTo((size()?.0 - 9) / 2, (size()?.1) / 2))?;
            // println!("you loose");
            Err(msg)?;
        }
    }
    //TcpStream
}
async fn read_key_to_command(tx: Sender<CommandKeys>) {
    let mut previous_command = CommandKeys::None;
    loop {
        let key_event = read().unwrap();
        let new_command = if let Some(key) = key_event.as_key_press_event() {
            match key.code {
                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                    CommandKeys::Directions(Direction::Up)
                }
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                    CommandKeys::Directions(Direction::Down)
                }
                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                    CommandKeys::Directions(Direction::Right)
                }
                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                    CommandKeys::Directions(Direction::Left)
                }
                KeyCode::Char(' ') => CommandKeys::ChangeSpeed,
                KeyCode::Esc => CommandKeys::End,
                _ => continue,
            }
        }
        // println!("pressed");
        //stdout().flush().unwrap();
        else {
            continue;
        };

        if let CommandKeys::Directions(_) = new_command {
            if new_command == previous_command {
                continue;
            }
        }
        previous_command = new_command.clone();

        tx.send(new_command).unwrap();
    }
    // print!("command");
    // return;
}

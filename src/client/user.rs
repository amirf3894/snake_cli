use crate::{
    game::{
        self, init,
        model::{CommandKeys, Direction},
    },
    server::host::{ClientSendData, HostSideData},
};
use clap::builder::Str;
use crossterm::{
    self,
    cursor::{self, MoveTo},
    event::{KeyCode, read},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, enable_raw_mode, size},
};
use std::{
    io::{Write, stdout},
    sync::{Arc, RwLock, mpsc::Sender},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Interest},
    net::TcpStream,
};
pub async fn main_client(name: &str, addr: &str) -> Result<(), Box<dyn (std::error::Error)>> {
    let mut buff = [0_u8; 10_000];
    let mut stream = TcpStream::connect(addr).await?;
    // let data = serde_json::to_string(&ClientSendData {
    //     terminal_size: size().unwrap(),
    //     command: CommandKeys::None,
    // })?;
    //stream.write(data.as_bytes()).await?;
    // let _ = stream.read_u8().await;
    let (tx, rx) = std::sync::mpsc::channel::<CommandKeys>();
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    // let command = Arc::new(RwLock::new(CommandKeys::None));
    tokio::spawn(read_key_to_command(tx));
    loop {
        let command = rx.try_recv().unwrap_or(CommandKeys::None);
        //print!("{:?}", command);
        //let terminal_size = size()?;

        // let command_to_send = {
        //     let gurad = command.read().unwrap();
        //     (*gurad).clone()
        // };
        // *command.write().unwrap() = CommandKeys::None;
        let data = serde_json::to_string(&ClientSendData {
            terminal_size: size()?,
            command,
        })?;
        stream.write(data.as_bytes()).await?;
        let len = stream.read(&mut buff).await?;
        let data = serde_json::from_str::<HostSideData>(&String::from_utf8_lossy(&buff[..len]))?;

        execute!(stdout, MoveTo(0, 0),)?;
        write!(stdout, "{}", data.display_data)?;

        stdout.flush()?;
    }
    //TcpStream
}
async fn read_key_to_command(tx: Sender<CommandKeys>) {
    loop {
        let key_event = read().unwrap();
        let new_command = match key_event.as_key_press_event().unwrap().code {
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
            KeyCode::Char(' ') => CommandKeys::Faster,
            KeyCode::Esc => CommandKeys::End,
            _ => continue,
        };
        tx.send(new_command).unwrap();
        // println!("pressed");
        //stdout().flush().unwrap();
    }

    // print!("command");
    // return;
}

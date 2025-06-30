use crate::{
    game::{self, init, model::CommandKeys, snake::read_key_to_command},
    server::host::{ClientSendData, HostSideData},
};
use clap::builder::Str;
use crossterm::{
    self,
    cursor::{self, MoveTo},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, enable_raw_mode, size},
};
use std::{
    io::{Write, stdout},
    sync::{Arc, RwLock},
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
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let command = Arc::new(RwLock::new(CommandKeys::None));
    tokio::spawn(read_key_to_command(command.clone()));
    loop {
        //let terminal_size = size()?;

        let command_to_send = {
            let gurad = command.read().unwrap();
            (*gurad).clone()
        };
        *command.write().unwrap() = CommandKeys::None;
        let data = serde_json::to_string(&ClientSendData {
            terminal_size: size()?,
            command: command_to_send,
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

use crate::model::*;
use colored::{self, Colorize};
use crossterm::{
    self,
    cursor::{self, MoveTo},
    event::{KeyCode, read},
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size,
    },
};
use std::{
    io::{self, Stdout, Write, stdout},
    sync::mpsc::Sender,
    thread::sleep,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
/*the logic of main client:
first it spawns a function to get pressed keys and send them to a channal for main client loop
also it reads data from Tcpstream which contains game status and a string to prilnt it on client terminal
then send some data containing user pressed key to server both read_key and send_recieve perfoms asynchroniusly */

///connects to a tcp server and reads data, shows data, sendsbadk user commans(change direction, exit, ...)
///i game stops for any reason(loose, server errors, ...) it returns an error
pub async fn main_client(addr: &str) -> Result<(), Box<dyn (std::error::Error)>> {
    const FASTER_DURATION: u64 = 50;
    const SLOWER_DURATION: u64 = 200;
    let mut loose_weight = false;
    let mut duration = SLOWER_DURATION;
    let mut buff;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let mut stream = TcpStream::connect(addr)
        .await
        .map_err(|_| "COULDN'T CONNECT TO SERVER!")?;

    let (tx, rx) = std::sync::mpsc::channel::<CommandKeys>();
    enable_raw_mode()?;
    tokio::spawn(read_key_to_command(tx));
    let mut host_side_data = HostSideData {
        display_data: "".to_string(),
        status: GameStatus::Alive,
        len: 2,
    };
    let mut client_side_data;
    loop {
        buff = [0_u8; 20_000];
        sleep(Duration::from_millis(duration - 30)); //i will explain it later
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
        client_side_data = ClientSendData {
            terminal_size: size()?,
            command,
            loose_weight,
        };
        stream
            .write(serde_json::to_string(&client_side_data)?.as_bytes())
            .await
            .map_err(|_| "COULDN'T WRITE TO SERVER!")?;
        sleep(Duration::from_millis(30)); //for slow networks it waits for server to send data
        let len = stream
            .read(&mut buff)
            .await
            .map_err(|_| "COULDN't READ FROM SERVER!")?;
        host_side_data =
            serde_json::from_str::<HostSideData>(&String::from_utf8_lossy(&buff[..len]))
                .map_err(|_| "COULDN't READ FROM SERVER!")?;

        execute!(stdout, MoveTo(0, 0),)?;
        write!(stdout, "{}", host_side_data.display_data)?;
        stdout.flush()?;
        if let GameStatus::Dead(msg) = host_side_data.status {
            Err(msg)?;
        }
    }
    //TcpStream
}

///reads pressed keys and convert them to CommandKey and ignore boilerplate keys
/// then send it for main_client via a channel
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
        } else {
            continue;
        };

        //igronring repeatitive keys
        if let CommandKeys::Directions(_) = new_command {
            if new_command == previous_command {
                continue;
            }
        }
        previous_command = new_command.clone();
        let _ = tx.send(new_command);
    }
}

///after stop playing game it gets a text and show that then
///closes alternate terminal and backs to place where user started the game
pub fn end(text: &str, stdout: &mut Stdout) -> io::Result<()> {
    let size = size()?;
    let max_width = text.split('\n').max_by_key(|p| p.len()).unwrap().len() + 4;
    let max_height = text.split('\n').count() + 2;
    execute!(
        stdout,
        MoveTo(
            (size.0 - max_width as u16) / 2,
            (size.1 - max_height as u16) / 2
        )
    )?;
    write!(stdout, "{}", "*".repeat(max_width))?;

    for (i, phrase) in text.split('\n').enumerate() {
        execute!(
            stdout,
            MoveTo(
                (size.0 - phrase.len() as u16) / 2,
                (size.1 - max_height as u16) / 2 + 1 + i as u16
            )
        )?;
        write!(stdout, "{}", phrase.bright_magenta())?;
    }
    execute!(
        stdout,
        MoveTo(
            (size.0 - max_width as u16) / 2,
            (size.1 + max_height as u16) / 2 - 1 as u16
        )
    )?;
    write!(stdout, "{}", "*".repeat(max_width))?;
    stdout.flush()?;
    read()?;
    execute!(stdout, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

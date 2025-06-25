use std::{io::stdout, process::exit};

use clap::{Arg, ArgMatches, Command};
use crossterm::{
    cursor, execute,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};
//use snake::game::snake;
use snakecli::{self, game};
#[tokio::main]
async fn main() {
    let matches = Command::new("snake")
        .subcommand(
            Command::new("host")
                .arg(
                    Arg::new("ip")
                        .short('i')
                        .long("ip")
                        .required(true)
                        .help("An ip that server is visible for client"),
                )
                .arg(
                    Arg::new("size")
                        .short('s')
                        .long("size")
                        .default_value("(200, 100)")
                        .help("Playground size which players play on it default is: (200, 100)"),
                )
                .about("Create a server that clients connect to it"),
        )
        .subcommand(
            Command::new("client")
                .arg(
                    Arg::new("ip")
                        .short('i')
                        .long("ip")
                        .required(true)
                        .help("A server ip address to connect and play with others"),
                )
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("Your username which shown in the game"),
                )
                .about("Connect to a server and you can play with otherse "),
        )
        .subcommand(
            Command::new("solo").about("Play the game lonely there is no player except you :)"),
        )
        .about("A cli snake game that you can play it with others")
        .subcommand_required(true)
        .get_matches();

    let result = match matches.subcommand() {
        Some(("host", arg)) => host(arg),
        Some(("client", arg)) => client(arg),
        Some(("solo", _)) => solo().await,
        _ => exit(0),
    };
    disable_raw_mode().unwrap();
    execute!(stdout(), LeaveAlternateScreen, cursor::Show).unwrap();
    if let Err(e) = result {
        println!("An error occoured");
        println!("{}", e.to_string());
        exit(1);
    }
}
fn host(arg: &ArgMatches) -> Result<(), Box<dyn (std::error::Error)>> {
    Ok(())
}
fn client(arg: &ArgMatches) -> Result<(), Box<dyn (std::error::Error)>> {
    Ok(())
}
async fn solo() -> Result<(), Box<dyn (std::error::Error)>> {
    game::snake::main_snake().await?;
    Ok(())
}

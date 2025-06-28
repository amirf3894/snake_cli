use std::{io::stdout, process::exit};

use clap::{Arg, ArgMatches, Command};
use crossterm::{
    cursor, execute,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};
//use snake::game::snake;
use snakecli::{self, client::user::main_client, game, server::host::main_host};
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
                    Arg::new("height")
                        .short('H')
                        .long("height")
                        .default_value("100")
                        .help("Playground height size which players play on it default is: 100"),
                )
                .arg(
                    Arg::new("width")
                        .short('W')
                        .long("width")
                        .default_value("200")
                        .help("Playground width size which players play on it default is: 200"),
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
        Some(("host", arg)) => host(arg).await,
        // Some(("client", arg)) => client(arg),
        Some(("solo", _)) => solo().await,
        Some(("client", arg)) => client(arg).await,
        _ => exit(0),
    };
    // disable_raw_mode().unwrap();

    // execute!(stdout(), LeaveAlternateScreen, cursor::Show).unwrap();
    if let Err(e) = result {
        println!("An error occoured");
        println!("{}", e.to_string());
        exit(1);
    }
}
async fn host(arg: &ArgMatches) -> Result<(), Box<dyn (std::error::Error)>> {
    let addr = arg.get_one::<String>("ip").unwrap();
    let width = arg.get_one::<String>("width").unwrap();
    let height = arg.get_one::<String>("height").unwrap();
    let width: u16 = width.trim().parse()?;
    let height: u16 = height.trim().parse()?;
    main_host((width, height), addr).await?;
    //println!("left");
    Ok(())
}
async fn client(arg: &ArgMatches) -> Result<(), Box<dyn (std::error::Error)>> {
    let addr = arg.get_one::<String>("ip").unwrap();
    let name = arg.get_one::<String>("name").unwrap();
    main_client(name, addr).await?;

    Ok(())
}
async fn solo() -> Result<(), Box<dyn (std::error::Error)>> {
    game::snake::main_snake().await?;
    Ok(())
}

use std::process::exit;

use clap::{Arg, ArgMatches, Command};
use snake::game::snake;

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
                        .help("Playground size which players play on it"),
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
                .about("Connect to a server and you can play with others "),
        )
        .subcommand(
            Command::new("solo").about("Play the game lonely there is no player except you :)"),
        )
        .about("A cli snake game that you can play it with others")
        .subcommand_required(true)
        .get_matches();

    match matches.subcommand() {
        Some(("host", arg)) => host(arg),
        Some(("client", arg)) => client(arg),
        Some(("solo", _)) => solo().await,
        _ => exit(0),
    }

    fn host(arg: &ArgMatches) {}
    fn client(arg: &ArgMatches) {}
    async fn solo() {
        snake::main_snake().await.unwrap();
    }
    snake::main_snake().await.unwrap();
}

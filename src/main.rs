use std::process::exit;

use clap::{Arg, ArgMatches, Command};
use snake::game::snake;

#[tokio::main]
async fn main() {
    let matches = Command::new("snake")
        .subcommand(
            Command::new("host")
                .arg(Arg::new("ip").short('i').long("ip").required(true))
                .arg(Arg::new("size").short('s').long("size")),
        )
        .subcommand(
            Command::new("client")
                .arg(Arg::new("ip").short('i').long("ip").required(true))
                .arg(Arg::new("name").required(true)),
        )
        .subcommand(Command::new("solo"))
        .subcommand_required(true)
        .get_matches();

    match matches.subcommand() {
        Some(("host", arg)) => host(arg),
        Some(("client", arg)) => client(arg),
        Some(("solo", _)) => solo(),
        _ => exit(0),
    }

    fn host(arg: &ArgMatches) {}
    fn client(arg: &ArgMatches) {}
    fn solo() {}
    snake::main_snake().await.unwrap();
}

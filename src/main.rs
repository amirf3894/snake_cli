use clap::{Arg, ArgMatches, Command};
use colored::{self, Colorize};
use snakecli::{client::user::main_client, server::host::main_host};
use std::{io::stdout, process::exit};
#[tokio::main]
async fn main() {
    let matches = Command::new("snake")
        .subcommand(
            Command::new("host")
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .default_value("1100")
                        .help("A port that server is visible for client (default: 1100)"),
                )
                .arg(
                    Arg::new("height")
                        .short('H')
                        .long("height")
                        .default_value("100")
                        .help("Playground height size which players play on it (default: 100)"),
                )
                .arg(
                    Arg::new("width")
                        .short('W')
                        .long("width")
                        .default_value("200")
                        .help("Playground width size which players play on it (default: 200)"),
                )
                .about("Create a server that clients connect to it"),
        )
        .subcommand(
            Command::new("client").arg(
                Arg::new("ip")
                    .short('i')
                    .long("ip")
                    .help("A server ip address to connect and play with others"),
            ),
        )
        .subcommand(Command::new("tutorial").about("theaches how to play and shows game options"))
        .subcommand_required(true)
        .about("A cli snake game that you can play it with others")
        .get_matches();

    let result = match matches.subcommand() {
        Some(("host", arg)) => host(arg).await,
        Some(("client", arg)) => client(arg).await,
        Some(("tutorial", _)) => {
            tutorial();
            Ok(())
        }
        _ => exit(0),
    };
    if let Err(e) = result {
        println!("An error occoured");
        println!("{}", e.to_string());
        exit(1);
    }
}

use snakecli::client::user::end;
async fn host(arg: &ArgMatches) -> Result<(), Box<dyn (std::error::Error)>> {
    let port = arg.get_one::<String>("port").unwrap();
    let width = arg.get_one::<String>("width").unwrap();
    let height = arg.get_one::<String>("height").unwrap();
    let width: u16 = width.trim().parse()?;
    let height: u16 = height.trim().parse()?;
    let port: u16 = port.trim().parse()?;
    main_host((width, height), port).await?;
    //println!("left");
    Ok(())
}

async fn client(arg: &ArgMatches) -> Result<(), Box<dyn (std::error::Error)>> {
    let addr = arg.get_one::<String>("ip").unwrap();
    let err = main_client(addr).await;

    end(&err.unwrap_err().to_string(), &mut stdout())?;
    exit(0);
}

fn tutorial() {
    println!("{}", "************************".bright_purple());
    println!("{}", "**Welcome to snake_cli**".bright_purple());
    println!("{}", "************************".bright_purple());
    println!("");
    println!("{}", "***To play the game***".bold().yellow());
    println!("{}", "1) Creat a server using host command");
    println!(
        "{}",
        "2) Connect to server by given IP using client command"
    );
    println!("{}", "3) Enjoy the game :)");
    println!("");
    println!("{}", "***How to play***".bold().yellow());
    println!(
        "{}",
        "Initilally your snake randomly spawns on the playground with green head (enemys are red heads)."
    );
    println!("{}", "Snake head is 'X' and its body pieces are 'O's.");
    println!(
        "{}",
        "You see some numbers on the ground, those are foods so you can eat them."
    );
    println!(
        "{}",
        "You can conttol you snake by (w, s, a, d) or (up, down, right, left) arrow keys."
    );
    println!(
        "{}",
        "Your sanke can accelerate by pressing space on the keyboard (to decelerate press space again)."
    );
    println!(
        "{}",
        "Be carefull, Acceleration consumes your body pieces!!".bright_red()
    );
    println!("{}", "To quit the game just press Esc.");
    println!("");
}

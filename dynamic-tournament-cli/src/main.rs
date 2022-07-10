mod systems;
mod tournaments;

use std::{
    io::{self, Write},
    str::FromStr,
};

use clap::{Parser, Subcommand};
use dynamic_tournament_api::{Client, Error};

#[derive(Debug, Parser)]
#[clap(version, about)]
pub struct Args {
    #[clap(short = 'h', long)]
    uri: String,
    #[clap(short, long)]
    username: Option<String>,
    #[clap(short, long)]
    password: Option<String>,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Systems {
        #[clap(subcommand)]
        command: systems::Command,
    },
    Tournaments {
        #[clap(subcommand)]
        command: tournaments::Command,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("URI: {}", args.uri);

    let client = Client::new(args.uri);

    if let Some((username, password)) = args.username.zip(args.password) {
        match client.v3().auth().login(&username, &password).await {
            Ok(_) => (),
            Err(err) => {
                match err {
                    Error::Unauthorized => println!("Failed to authorize: Unauthorized"),
                    err => println!("Failed to authorize: {}", err),
                }

                std::process::exit(1);
            }
        }

        println!("Logged in");
    }

    let res = match args.command {
        Command::Systems { command } => command.run(&client).await,
        Command::Tournaments { command } => command.run(&client).await,
    };

    if let Err(err) = res {
        println!("{}", err);
    }
}

pub fn read_line<T>(name: &str) -> Result<T, T::Err>
where
    T: FromStr,
{
    let buf = format!("{}: ", name);
    io::stdout().write_all(buf.as_bytes()).unwrap();
    io::stdout().flush().unwrap();

    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    FromStr::from_str(&buf[..buf.len() - 1])
}

mod systems;
mod tournaments;
mod utils;

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
    pretty_env_logger::init();

    let args = Args::parse();

    log::info!("Using base {}", args.uri);

    let client = Client::new(args.uri);

    if let Some((_username, _password)) = args.username.zip(args.password) {
        match client.v3().auth().login().await {
            Ok(_) => (),
            Err(err) => {
                match err {
                    Error::Unauthorized => log::error!("Failed to authorize: Unauthorized"),
                    err => log::error!("Failed to authorize: {}", err),
                }

                std::process::exit(1);
            }
        }

        log::info!("Logged in");
    } else {
        log::info!("No username or password provided, some operations are unavaliable");
    }

    let res = match args.command {
        Command::Systems { command } => command.run(&client).await,
        Command::Tournaments { command } => command.run(&client).await,
    };

    if let Err(err) = res {
        log::error!("{}", err);
    }
}

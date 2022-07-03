use clap::Subcommand;
use dynamic_tournament_api::{Client, Result};

#[derive(Debug, Subcommand)]
pub enum Command {
    List,
}

impl Command {
    pub async fn run(&self, client: &Client) -> Result<()> {
        match self {
            Self::List => {
                let tournaments = client.v3().tournaments().list().await?;

                println!("ID | Name | Date | Kind");
                for tournament in tournaments {
                    println!(
                        "{} | {} | {} | {}",
                        tournament.id, tournament.name, tournament.date, tournament.kind
                    );
                }
            }
        }

        Ok(())
    }
}

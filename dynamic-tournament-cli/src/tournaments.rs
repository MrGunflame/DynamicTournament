use clap::Subcommand;
use dynamic_tournament_api::{
    v3::{id::TournamentId, tournaments::Tournament},
    Client, Result,
};

use crate::utils::Prompt;

#[derive(Debug, Subcommand)]
pub enum Command {
    List,
    Create,
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
            Self::Create => {
                let name = Prompt::new("Name").read_valid();
                let date = Prompt::new("Date").read_valid();
                let description = Prompt::new("Description").read_valid();
                let kind = Prompt::new("Kind ('team' or 'player')").read_valid();

                client
                    .v3()
                    .tournaments()
                    .create(&Tournament {
                        id: TournamentId(0),
                        name,
                        description,
                        date,
                        kind,
                    })
                    .await?;
            }
        }

        Ok(())
    }
}

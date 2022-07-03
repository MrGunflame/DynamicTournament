use clap::Subcommand;
use dynamic_tournament_api::{
    v3::{id::TournamentId, tournaments::Tournament},
    Client, Result,
};

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
                let name = crate::read_line("Name").unwrap();
                let date = crate::read_line("Date").unwrap();
                let description = crate::read_line("Description").unwrap();
                let kind = crate::read_line("Kind ('team' or 'player')").unwrap();

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

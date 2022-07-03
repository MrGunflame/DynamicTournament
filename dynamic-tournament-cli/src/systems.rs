use clap::Subcommand;
use dynamic_tournament_api::{Client, Result};

#[derive(Debug, Subcommand)]
pub enum Command {
    List,
}

impl Command {
    pub async fn run(&self, client: &Client) -> Result<()> {
        let systems = client.v3().systems().list().await?;

        println!("ID | Name");
        for system in systems {
            println!("{} | {}", system.id, system.name);
        }

        Ok(())
    }
}

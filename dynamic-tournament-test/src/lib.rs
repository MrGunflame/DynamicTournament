use dynamic_tournament_api::tournament::{BracketType, Entrants, Team, Tournament, TournamentId};

use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Default)]
pub struct TournamentGenerator {
    pub name: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub bracket_type: Option<BracketType>,
    pub entrants: usize,
}

impl TournamentGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate(&self) -> Tournament {
        let name = self.name.clone().unwrap_or("Test".to_owned());
        let date = self.date.unwrap_or(Utc::now());
        let bracket_type = self.bracket_type.unwrap_or(BracketType::SingleElimination);

        let mut teams = Vec::new();
        for i in 0..self.entrants {
            teams.push(Team {
                name: format!("Team {}", i),
                players: Vec::new(),
            });
        }

        Tournament {
            id: TournamentId(0),
            name,
            description: String::new(),
            date,
            bracket_type,
            entrants: Entrants::Teams(teams),
        }
    }
}

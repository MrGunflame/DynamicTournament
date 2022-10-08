use std::collections::HashMap;

use chrono::Utc;
use dynamic_tournament_api::{
    tournament::{Entrants, Player, Role, Team, Tournament},
    v3::{
        self,
        id::SystemId,
        tournaments::{entrants::EntrantVariant, EntrantKind},
    },
    Client,
};

#[tokio::main]
async fn main() {
    let client = Client::new("http://localhost:3030");
    client.v3().auth().login("a", "a").await.unwrap();

    test_v3(&client).await;
    v2_to_v3(&client).await;
}

async fn v2_to_v3(client: &Client) {
    let date = Utc::now();

    let tournament = Tournament {
        id: 0.into(),
        name: "test tournament".into(),
        date,
        description: "test description".into(),
        bracket_type: dynamic_tournament_api::tournament::BracketType::SingleElimination,
        entrants: Entrants::Teams(vec![
            Team {
                name: "team0".into(),
                players: vec![
                    Player {
                        name: "player0_0".into(),
                        role: Role::Unknown,
                        rating: None,
                    },
                    Player {
                        name: "player0_1".into(),
                        role: Role::Roamer,
                        rating: None,
                    },
                    Player {
                        name: "player0_2".into(),
                        role: Role::Teamfighter,
                        rating: None,
                    },
                    Player {
                        name: "player0_3".into(),
                        role: Role::Duelist,
                        rating: None,
                    },
                    Player {
                        name: "player0_4".into(),
                        role: Role::Support,
                        rating: None,
                    },
                ],
            },
            Team {
                name: "team1".into(),
                players: vec![],
            },
            Team {
                name: "team2".into(),
                players: vec![],
            },
        ]),
    };

    client.tournaments().create(&tournament).await.unwrap();

    let tournaments = client.v3().tournaments().list().await.unwrap();
    let resp = tournaments.first().unwrap();

    assert_eq!(resp.name, tournament.name);
    // assert_eq!(resp.date, tournament.date.to_rfc3339());
    assert_eq!(resp.kind, EntrantKind::Team);

    let roles = {
        let mut map = HashMap::new();

        let roles = client
            .v3()
            .tournaments()
            .roles(resp.id)
            .list()
            .await
            .unwrap();

        for role in roles {
            let r = match role.name.as_str() {
                "Unknown" => Role::Unknown,
                "Roamer" => Role::Roamer,
                "Teamfighter" => Role::Teamfighter,
                "Support" => Role::Support,
                "Duelist" => Role::Duelist,
                _ => panic!("invalid role name {}", role.name),
            };

            map.insert(r, role.id);
        }

        assert_eq!(map.len(), 5);

        map
    };

    let entrants = client
        .v3()
        .tournaments()
        .entrants(resp.id)
        .list()
        .await
        .unwrap();

    let mut entrant_ids = Vec::new();
    for (index, entrant) in entrants.into_iter().enumerate() {
        client
            .v3()
            .tournaments()
            .entrants(resp.id)
            .get(entrant.id)
            .await
            .unwrap();

        entrant_ids.push(entrant.id);

        match entrant.inner {
            EntrantVariant::Player(_) => panic!("found a player entrant"),
            EntrantVariant::Team(team) => match index {
                0 => {
                    assert_eq!(team.name, "team0");

                    for (i, player) in team.players.into_iter().enumerate() {
                        match i {
                            0 => {
                                assert_eq!(player.name, "player0_0");
                                assert_eq!(player.role, *roles.get(&Role::Unknown).unwrap());
                            }
                            1 => {
                                assert_eq!(player.name, "player0_1");
                                assert_eq!(player.role, *roles.get(&Role::Roamer).unwrap());
                            }
                            2 => {
                                assert_eq!(player.name, "player0_2");
                                assert_eq!(player.role, *roles.get(&Role::Teamfighter).unwrap());
                            }
                            3 => {
                                assert_eq!(player.name, "player0_3");
                                assert_eq!(player.role, *roles.get(&Role::Duelist).unwrap());
                            }
                            4 => {
                                assert_eq!(player.name, "player0_4");
                                assert_eq!(player.role, *roles.get(&Role::Support).unwrap());
                            }
                            _ => panic!("too many players in team"),
                        }
                    }
                }
                1 => {
                    assert_eq!(team.name, "team1");
                    assert_eq!(team.players.len(), 0);
                }
                2 => {
                    assert_eq!(team.name, "team2");
                    assert_eq!(team.players.len(), 0);
                }
                _ => panic!("too many entrants created"),
            },
        }
    }

    let brackets = client
        .v3()
        .tournaments()
        .brackets(resp.id)
        .list()
        .await
        .unwrap();

    assert_eq!(brackets.len(), 1);

    let bracket = client
        .v3()
        .tournaments()
        .brackets(resp.id)
        .get(brackets[0].id)
        .await
        .unwrap();

    assert_eq!(bracket.system, SystemId(1));
    assert_eq!(bracket.name, "test tournament");
    assert_eq!(bracket.entrants, entrant_ids);

    client.v3().tournaments().delete(resp.id).await.unwrap();
}

async fn test_v3(client: &Client) {
    let date = Utc::now();

    let tournament = v3::tournaments::Tournament {
        id: 0.into(),
        name: "test tournament".into(),
        description: "test description".into(),
        date: date.clone(),
        kind: v3::tournaments::EntrantKind::Team,
    };

    let resp = client.v3().tournaments().create(&tournament).await.unwrap();
    assert_eq!(tournament.name, resp.name);
    assert_eq!(tournament.date, resp.date);
    assert_eq!(resp.date, date);
    assert_eq!(tournament.description, resp.description);
    assert_eq!(tournament.kind, resp.kind);

    let entrant = v3::tournaments::entrants::Entrant {
        id: 0.into(),
        inner: v3::tournaments::entrants::EntrantVariant::Player(
            v3::tournaments::entrants::Player {
                name: "test player".into(),
                role: 0.into(),
                rating: None,
            },
        ),
    };

    client
        .v3()
        .tournaments()
        .entrants(resp.id)
        .create(&entrant)
        .await
        .unwrap_err();

    // let entrant = v3::tournaments::entrants::Entrant {
    //     id: 0.into(),
    //     inner: v3::tournaments::entrants::EntrantVariant::Team(v3::tournaments::entrants::Team {
    //         name: "test team".into(),
    //         players: vec![],
    //     }),
    // };
}

mod entrant;
mod r#match;

use dynamic_tournament_api::tournament::Entrants as EntrantsVar;
use dynamic_tournament_generator::options::TournamentOptions;
use dynamic_tournament_generator::tournament::TournamentKind;
use dynamic_tournament_generator::{
    EntrantScore, EntrantSpot, Entrants, Match, MatchResult, Matches, Node, SingleElimination,
    System,
};
use entrant::BracketEntrant;
use r#match::{Action, BracketMatch};

use dynamic_tournament_generator::render::{
    self, BracketRound, BracketRounds, Position, Renderer, Round,
};
use dynamic_tournament_generator::tournament::Tournament;

use dynamic_tournament_api::tournament::{self, BracketType, Player, Team, TournamentId};
use dynamic_tournament_api::{websocket, Client};
use yew_agent::{Bridge, Bridged};

use std::fmt::Display;
use std::rc::Rc;

use yew::prelude::*;

use crate::components::confirmation::Confirmation;
use crate::components::popup::Popup;
use crate::components::providers::{ClientProvider, Provider};
use crate::components::update_bracket::BracketUpdate;
use crate::services::{EventBus, WebSocketService};
use crate::utils::FetchData;

pub struct Bracket {
    websocket: WebSocketService,
    _producer: Box<dyn Bridge<EventBus>>,
    popup: Option<PopupState>,
    bracket: FetchData<AnyTournament>,
}

impl Component for Bracket {
    type Message = Message;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::take(ctx);
        let id = ctx.props().tournament.id;

        let websocket = WebSocketService::new(&client, ctx.props().tournament.id.0);

        let kind = match ctx.props().tournament.bracket_type {
            BracketType::SingleElimination => TournamentKind::SingleElimination,
            BracketType::DoubleElimination => TournamentKind::DoubleElimination,
        };

        let options = match kind {
            TournamentKind::SingleElimination => {
                SingleElimination::<u8, EntrantScore<u8>>::options()
            }
            TournamentKind::DoubleElimination => TournamentOptions::default(),
        };

        let entrants = ctx.props().tournament.entrants.clone();

        ctx.link().send_future(async move {
            async fn fetch_data(
                kind: TournamentKind,
                entrants: EntrantsVar,
                client: Client,
                id: TournamentId,
                options: TournamentOptions,
            ) -> FetchData<AnyTournament> {
                let client = client.tournaments();

                match client.bracket(id).get().await {
                    Ok(bracket) => match entrants {
                        EntrantsVar::Players(entrants) => {
                            let tournament = Tournament::resume(
                                kind,
                                Entrants::from(entrants),
                                bracket.0,
                                options,
                            )
                            .unwrap();

                            FetchData::new_with_value(AnyTournament::Players(tournament))
                        }
                        EntrantsVar::Teams(entrants) => {
                            let tournament = Tournament::resume(
                                kind,
                                Entrants::from(entrants),
                                bracket.0,
                                options,
                            )
                            .unwrap();

                            FetchData::from(AnyTournament::Teams(tournament))
                        }
                    },
                    Err(_) => match entrants {
                        EntrantsVar::Players(entrants) => {
                            let mut tournament = Tournament::new(kind, options);
                            tournament.extend(Entrants::from(entrants));
                            FetchData::new_with_value(AnyTournament::Players(tournament))
                        }
                        EntrantsVar::Teams(entrants) => {
                            let mut tournament = Tournament::new(kind, options);
                            tournament.extend(Entrants::from(entrants));
                            FetchData::new_with_value(AnyTournament::Teams(tournament))
                        }
                    },
                }
            }

            let data = fetch_data(kind, entrants, client, id, options).await;

            Message::Update(data)
        });

        Self {
            bracket: FetchData::new(),
            websocket,
            _producer: EventBus::bridge(ctx.link().callback(Message::HandleMessage)),
            popup: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Update(bracket) => {
                self.bracket = bracket;

                true
            }
            Message::HandleMessage(msg) => {
                log::debug!("Received message: {:?}", msg);

                let bracket = self.bracket.as_mut().unwrap();

                match msg {
                    websocket::Message::UpdateMatch { index, nodes } => {
                        bracket.update_match(index.try_into().unwrap(), |m, res| {
                            let mut loser_index = None;

                            for (i, (entrant, node)) in m.entrants.iter_mut().zip(nodes).enumerate()
                            {
                                if let EntrantSpot::Entrant(entrant) = entrant {
                                    entrant.data = node;
                                }

                                if node.winner {
                                    res.winner_default(entrant);
                                    loser_index = Some(match i {
                                        0 => 1,
                                        _ => 1,
                                    });
                                }
                            }

                            if let Some(loser_index) = loser_index {
                                res.loser_default(&m.entrants[loser_index]);
                            }
                        });
                    }
                    websocket::Message::ResetMatch { index } => {
                        bracket.update_match(index, |_, res| {
                            res.reset_default();
                        });
                    }
                    _ => (),
                }

                true
            }
            Message::Action { index, action } => {
                log::debug!("Called action {:?} on {}", action, index);

                match action {
                    Action::UpdateMatch => {
                        self.popup = Some(PopupState::UpdateScores(index));
                    }
                    Action::ResetMatch => {
                        self.popup = Some(PopupState::ResetMatch(index));
                    }
                }

                true
            }

            Message::ClosePopup => {
                self.popup = None;
                true
            }
            Message::UpdateMatch { index, nodes } => {
                let mut websocket = self.websocket.clone();
                ctx.link().send_future_batch(async move {
                    websocket
                        .send(websocket::Message::UpdateMatch {
                            index: index.try_into().unwrap(),
                            nodes,
                        })
                        .await;

                    vec![Message::ClosePopup]
                });

                false
            }
            Message::ResetMatch(index) => {
                let mut websocket = self.websocket.clone();
                ctx.link().send_future_batch(async move {
                    websocket
                        .send(websocket::Message::ResetMatch { index })
                        .await;

                    vec![Message::ClosePopup]
                });

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.bracket.render(|bracket| {
            let popup = match self.popup {
                Some(PopupState::UpdateScores(index)) => {
                    let on_close = ctx.link().callback(|_| Message::ClosePopup);

                    let m = &bracket.matches()[index];

                    let entrants = dynamic_tournament_generator::Entrants::from(
                        ctx.props().tournament.entrants.clone().unwrap_teams(),
                    );

                    let teams = m
                        .entrants
                        .clone()
                        .map(|e| e.map(|e| e.entrant(&entrants).unwrap().clone()));
                    let nodes = m.entrants.clone().map(|e| e.unwrap().data);

                    let on_submit = ctx
                        .link()
                        .callback(move |nodes| Message::UpdateMatch { index, nodes });

                    html! {
                        <Popup on_close={on_close}>
                            <BracketUpdate {teams} {nodes} on_submit={on_submit} />
                        </Popup>
                    }
                }
                Some(PopupState::ResetMatch(index)) => {
                    let on_close = ctx.link().callback(|_| Message::ClosePopup);

                    let on_confirm = ctx.link().callback(move |_| Message::ResetMatch(index));

                    html! {
                        <Confirmation {on_close} {on_confirm} />
                    }
                }
                None => html! {},
            };

            let bracket = match bracket {
                AnyTournament::Players(tournament) => {
                    HtmlRenderer::new(tournament, ctx).into_output()
                }
                AnyTournament::Teams(tournament) => {
                    HtmlRenderer::new(tournament, ctx).into_output()
                }
            };

            html! {
                <>
                    { bracket }
                    { popup }
                </>
            }
        })
    }
}

pub enum Message {
    Update(FetchData<AnyTournament>),
    HandleMessage(websocket::Message),
    Action {
        index: usize,
        action: Action,
    },
    ClosePopup,
    UpdateMatch {
        index: usize,
        nodes: [EntrantScore<u64>; 2],
    },
    ResetMatch(usize),
}

#[derive(Clone, Debug, Properties)]
pub struct Properties {
    pub tournament: Rc<tournament::Tournament>,
}

impl PartialEq for Properties {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.tournament, &other.tournament)
    }
}

pub struct HtmlRenderer<'a, T, E>
where
    T: System<Entrant = E, NodeData = EntrantScore<u64>>,
    E: Clone + Display + 'static,
{
    output: Html,
    ctx: &'a Context<Bracket>,
    tournament: &'a T,
}

impl<'a, T, E> HtmlRenderer<'a, T, E>
where
    T: System<Entrant = E, NodeData = EntrantScore<u64>>,
    E: Clone + Display + 'static,
{
    pub fn new(tournament: &'a T, ctx: &'a Context<Bracket>) -> Self {
        Self {
            output: html! {},
            ctx,
            tournament,
        }
    }

    pub fn into_output(mut self) -> Html {
        self.tournament.render(&mut self);

        html! {
            <div class="dt-bracket">
                { self.output }
            </div>
        }
    }
}

impl<'a, T, E> HtmlRenderer<'a, T, E>
where
    T: System<Entrant = E, NodeData = EntrantScore<u64>>,
    E: Clone + Display + 'static,
{
    fn render_bracket_round(&self, input: BracketRound<'_, T>) -> Html {
        let brackets: Html = input.map(|bracket| self.render_bracket(bracket)).collect();

        html! {
            <div class="dt-bracket-bracket-row">
                { brackets }
            </div>
        }
    }

    fn render_bracket(&self, input: render::Bracket<'_, T>) -> Html {
        let rounds: Html = input.map(|round| self.render_round(round)).collect();

        html! {
            <div class="dt-bracket-bracket">
                { rounds }
            </div>
        }
    }

    fn render_round(&self, input: Round<'_, T>) -> Html {
        let round: Html = input
            .indexed()
            .enumerate()
            .map(|(match_index, (index, m, pos))| {
                html! {
                    { self.render_match(m, index, match_index.saturating_add(1), pos) }
                }
            })
            .collect();

        html! {
            <div class="dt-bracket-round">
                { round }
            </div>
        }
    }

    fn render_match(
        &self,
        input: &Match<Node<EntrantScore<u64>>>,
        index: usize,
        match_index: usize,
        position: Position,
    ) -> Html {
        let on_action = self
            .ctx
            .link()
            .callback(move |action| Message::Action { index, action });

        let entrants = input
            .entrants
            .clone()
            .map(|e| e.map(|e| e.entrant(&self.tournament.borrow()).unwrap().clone()));

        let nodes = input.entrants.clone().map(|e| e.map(|e| e.data));

        html! {
            <BracketMatch<E> {entrants} {nodes} {on_action} number={match_index} {position} />
        }
    }
}

impl<'a, T, E> Renderer<T, E, EntrantScore<u64>> for HtmlRenderer<'a, T, E>
where
    T: System<Entrant = E, NodeData = EntrantScore<u64>>,
    E: Clone + Display + 'static,
{
    fn render(&mut self, input: BracketRounds<'_, T>) {
        self.output = input
            .map(|bracket_round| self.render_bracket_round(bracket_round))
            .collect();
    }
}

enum PopupState {
    UpdateScores(usize),
    ResetMatch(usize),
}

/// A [`Tournament`] with either [`Player`]s or [`Team`]s as the entrants.
#[derive(Clone, Debug)]
pub enum AnyTournament {
    Players(Tournament<Player, EntrantScore<u64>>),
    Teams(Tournament<Team, EntrantScore<u64>>),
}

impl AnyTournament {
    #[inline]
    pub fn matches(&self) -> &Matches<EntrantScore<u64>> {
        match self {
            Self::Players(ref tournament) => tournament.matches(),
            Self::Teams(ref tournament) => tournament.matches(),
        }
    }

    #[inline]
    pub fn update_match<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut Match<Node<EntrantScore<u64>>>, &mut MatchResult<EntrantScore<u64>>),
    {
        match self {
            Self::Players(ref mut tournament) => tournament.update_match(index, f),
            Self::Teams(ref mut tournament) => tournament.update_match(index, f),
        }
    }
}

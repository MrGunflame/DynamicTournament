mod entrant;
mod r#match;

use dynamic_tournament_api::v3::tournaments::brackets::matches::Frame;
use dynamic_tournament_api::v3::tournaments::entrants::{Entrant, EntrantVariant};
use dynamic_tournament_generator::options::TournamentOptions;
use dynamic_tournament_generator::tournament::TournamentKind;
use dynamic_tournament_generator::{
    EntrantScore, EntrantSpot, Match, Node, SingleElimination, System,
};
use entrant::BracketEntrant;
use r#match::{Action, BracketMatch};

use dynamic_tournament_generator::render::{
    self, BracketRound, BracketRounds, Position, Renderer, Round,
};
use dynamic_tournament_generator::tournament::Tournament;

use yew_agent::{Bridge, Bridged};

use std::fmt::Display;
use std::rc::Rc;

use yew::prelude::*;

use dynamic_tournament_api::v3::id::SystemId;
use dynamic_tournament_api::v3::tournaments::brackets::Bracket as ApiBracket;
use dynamic_tournament_api::v3::tournaments::Tournament as ApiTournament;

use crate::components::confirmation::Confirmation;
use crate::components::popup::Popup;
use crate::components::providers::{ClientProvider, Provider};
use crate::components::update_bracket::BracketUpdate;
use crate::services::errorlog::ErrorLog;
use crate::services::{EventBus, WebSocketService};

pub struct Bracket {
    websocket: WebSocketService,
    _producer: Box<dyn Bridge<EventBus>>,
    popup: Option<PopupState>,
    state: Option<Tournament<String, EntrantScore<u64>>>,
}

impl Component for Bracket {
    type Message = Message;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::take(ctx);

        let websocket =
            match WebSocketService::new(&client, ctx.props().tournament.id, ctx.props().bracket.id)
            {
                Ok(ws) => ws,
                Err(err) => {
                    ErrorLog::error(err.to_string());
                    panic!("{}", err);
                }
            };

        let mut ws = websocket.clone();
        ctx.link().send_future_batch(async move {
            let _ = ws.send(Frame::SyncMatchesRequest).await;

            vec![]
        });

        Self {
            state: None,
            websocket,
            _producer: EventBus::bridge(ctx.link().callback(Message::HandleFrame)),
            popup: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::HandleFrame(msg) => {
                log::debug!("Received message: {:?}", msg);

                match msg {
                    Frame::UpdateMatch { index, nodes } => {
                        log::warn!(
                            "Received an UpdateMatch frame before initializing the state, ignoring"
                        );

                        let bracket = self.state.as_mut().unwrap();

                        {
                            bracket.update_match(index.try_into().unwrap(), |m, res| {
                                let mut loser_index = None;

                                for (i, (entrant, node)) in
                                    m.entrants.iter_mut().zip(nodes).enumerate()
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
                    }
                    Frame::ResetMatch { index } => {
                        log::warn!(
                            "Received a ResetMatch frame before initializing the state, ignoring"
                        );

                        let bracket = self.state.as_mut().unwrap();

                        {
                            bracket.update_match(index, |_, res| {
                                res.reset_default();
                            });
                        }
                    }
                    Frame::SyncMatchesResponse(matches) => {
                        let system_kind = match ctx.props().bracket.system {
                            SystemId(1) => TournamentKind::SingleElimination,
                            SystemId(2) => TournamentKind::DoubleElimination,
                            _ => unimplemented!(),
                        };

                        let options = match system_kind {
                            TournamentKind::SingleElimination => {
                                SingleElimination::<u8, EntrantScore<u8>>::options()
                            }
                            TournamentKind::DoubleElimination => TournamentOptions::default(),
                        };

                        let entrants = ctx
                            .props()
                            .bracket
                            .entrants
                            .iter()
                            .map(|id| {
                                // Map the EntrantId to an entrant name (from props).
                                for e in ctx.props().entrants.iter() {
                                    if e.id == *id {
                                        return match &e.inner {
                                            EntrantVariant::Player(player) => player.name.clone(),
                                            EntrantVariant::Team(team) => team.name.clone(),
                                        };
                                    }
                                }

                                // Id was not found in entrants.
                                String::from("Unknown")
                            })
                            .collect();

                        self.state =
                            match Tournament::resume(system_kind, entrants, matches, options) {
                                Ok(tournament) => Some(tournament),
                                Err(err) => {
                                    ErrorLog::error(err.to_string());
                                    None
                                }
                            };
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
                        .send(Frame::UpdateMatch {
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
                    websocket.send(Frame::ResetMatch { index }).await;

                    vec![Message::ClosePopup]
                });

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(bracket) = &self.state {
            let popup = match self.popup {
                Some(PopupState::UpdateScores(index)) => {
                    let on_close = ctx.link().callback(|_| Message::ClosePopup);

                    let m = &bracket.matches()[index];

                    let entrants = m
                        .entrants
                        .map(|e| e.map(|e| e.entrant(bracket).unwrap().clone()));

                    let nodes = m.entrants.map(|e| e.unwrap().data);

                    let on_submit = ctx
                        .link()
                        .callback(move |nodes| Message::UpdateMatch { index, nodes });

                    html! {
                        <Popup on_close={on_close}>
                            <BracketUpdate teams={entrants} {nodes} on_submit={on_submit} />
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

            let bracket = HtmlRenderer::new(bracket, ctx).into_output();

            html! {
                <>
                    { bracket }
                    { popup }
                </>
            }
        } else {
            html! { <span>{ "Loading" }</span> }
        }
    }
}

pub enum Message {
    HandleFrame(Frame),
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
    pub tournament: Rc<ApiTournament>,
    pub bracket: Rc<ApiBracket>,
    pub entrants: Rc<Vec<Entrant>>,
}

impl PartialEq for Properties {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.tournament, &other.tournament)
            && Rc::ptr_eq(&self.bracket, &other.bracket)
            && Rc::ptr_eq(&self.entrants, &other.entrants)
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
            .map(|e| e.map(|e| e.entrant(&self.tournament.borrow()).unwrap().clone()));

        let nodes = input.entrants.map(|e| e.map(|e| e.data));

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

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Display;
use std::marker::PhantomData;

use dynamic_tournament_core::{EntrantScore, EntrantSpot, System};
use yew::{html, Component, Context, Html, Properties};

use crate::utils::Rc;

pub struct Standings<S>
where
    S: System + 'static,
{
    _marker: PhantomData<S>,
}

impl<S, E> Component for Standings<S>
where
    S: System<Entrant = E, NodeData = EntrantScore<u64>> + 'static,
    E: Display,
{
    type Message = ();
    type Properties = Props<S>;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        // index => (wins, loses)
        let mut scores: HashMap<usize, Score> = HashMap::new();

        for match_ in ctx.props().tournament.matches().iter() {
            for entrant in &match_.entrants {
                let EntrantSpot::Entrant(entrant) = entrant else  {
                    continue;
                };

                let mut score = match scores.get(&entrant.index) {
                    Some(score) => *score,
                    None => Score { wins: 0, loses: 0 },
                };

                if match_.is_concluded() {
                    if entrant.data.winner {
                        score.wins += 1;
                    } else {
                        score.loses += 1;
                    }
                }

                scores.insert(entrant.index, score);
            }
        }

        let mut scores: Vec<_> = scores.into_iter().collect();
        scores.sort_by(|a, b| a.1.cmp(&b.1).reverse());

        let scores = scores
            .into_iter()
            .enumerate()
            .map(|(position, (index, score))| {
                let name = ctx.props().tournament.entrants().get(index).unwrap();

                html! {
                    <tr>
                        <td>
                            { format!("{}.", position + 1) }
                        </td>
                        <td>
                            { name }
                        </td>
                        <td>
                            { score.wins }
                        </td>
                        <td>
                            { score.loses }
                        </td>
                    </tr>
                }
            })
            .collect::<Html>();

        html! {
            <table class="dt-table-striped">
                <tr>
                    <th>
                        { "Position" }
                    </th>
                    <th>
                        { "Name" }
                    </th>
                    <th>
                        { "W" }
                    </th>
                    <th>
                        { "L" }
                    </th>
                </tr>
                { scores }
            </table>
        }
    }
}

#[derive(Properties)]
pub struct Props<S>
where
    S: System<NodeData = EntrantScore<u64>>,
{
    pub tournament: Rc<S>,
}

impl<S> PartialEq for Props<S>
where
    S: System<NodeData = EntrantScore<u64>>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.tournament == other.tournament
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Score {
    wins: usize,
    loses: usize,
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.wins.cmp(&other.wins) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.loses.cmp(&other.loses).reverse(),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

use std::cmp::Ordering;
use std::fmt::Display;
use std::marker::PhantomData;

use dynamic_tournament_core::{EntrantScore, System};
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

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        let standings = ctx.props().tournament.standings();

        let scores = standings
            .iter()
            .enumerate()
            .map(|(pos, entry)| {
                let name = ctx.props().tournament.entrants().get(entry.index).unwrap();

                let values: Html = entry
                    .values
                    .iter()
                    .map(|value| {
                        html! {
                            <td>
                                { value }
                            </td>
                        }
                    })
                    .collect();

                html! {
                    <tr>
                        <td>
                            { format!("{}.", pos + 1) }
                        </td>
                        <td>
                            { name }
                        </td>
                        { values }
                    </tr>
                }
            })
            .collect::<Html>();

        let keys: Html = standings
            .keys()
            .map(|key| {
                html! {
                    <th>
                        { key }
                    </th>
                }
            })
            .collect();

        html! {
            <table class="dt-table dt-table-striped">
                <tr>
                    <th>
                        { "Position" }
                    </th>
                    <th>
                        { "Name" }
                    </th>
                    { keys }
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

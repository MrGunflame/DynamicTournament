use std::collections::HashSet;

use yew_agent::{Agent, AgentLink, Context, HandlerId};

pub struct ClientEventBus {
    link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for ClientEventBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = ();
    type Output = ();

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        for sub in self.subscribers.iter() {
            self.link.respond(*sub, msg);
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
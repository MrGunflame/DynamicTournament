use std::collections::HashSet;

use yew_agent::{Agent, AgentLink, Context, Dispatched, HandlerId};

pub type MessageLog = ErrorLog;

#[allow(unused)]
pub struct ErrorLog;

impl ErrorLog {
    /// Dispatches a new error message to the error log.
    #[inline]
    #[allow(unused)]
    pub fn error<T>(msg: T)
    where
        T: ToString,
    {
        ErrorLogBus::dispatcher().send(msg.to_string());
    }

    #[inline]
    pub fn info<T>(msg: T)
    where
        T: ToString,
    {
        ErrorLogBus::dispatcher().send(msg.to_string());
    }
}

pub struct ErrorLogBus {
    link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for ErrorLogBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = String;
    type Output = String;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        for sub in self.subscribers.iter() {
            self.link.respond(*sub, msg.clone());
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

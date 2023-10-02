use actix::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::packets::{
    Connect, ConnectNotification, Disconnect, DisconnectNotification, Play, Poll, Ready2, Skip, S2C,
};

#[derive(Default)]
pub struct DTPServer {
    active_broadcasts: HashMap<String, Recipient<S2C>>,
    ready2_clients: HashSet<String>,
    queue: VecDeque<String>,
}

impl DTPServer {
    fn notify_connect(&self, notif: ConnectNotification) {
        for (_, listner) in &self.active_broadcasts {
            listner.do_send(S2C::ConnectNotification(notif.clone()));
        }
    }

    fn notify_disconnect(&self, notif: DisconnectNotification) {
        for (_, listner) in &self.active_broadcasts {
            listner.do_send(S2C::DisconnectNotification(notif.clone()));
        }
    }

    fn notify_skip(&self) {
        for (_, listner) in &self.active_broadcasts {
            listner.do_send(S2C::Skip);
        }
    }
}

impl Actor for DTPServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for DTPServer {
    type Result = ();

    #[allow(implied_bounds_entailment)]
    fn handle(&mut self, event: Connect, _: &mut Self::Context) -> Self::Result {
        self.active_broadcasts.insert(event.id.clone(), event.addr);
        tracing::info!("Client connected with ID: {}", event.id);
        self.notify_connect(event.notif);
    }
}

impl Handler<Disconnect> for DTPServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        self.active_broadcasts.remove(&msg.id);
        tracing::info!("Client disconnected with ID: {}", msg.id);
        self.notify_disconnect(msg.notif);
    }
}

impl Handler<Play> for DTPServer {
    type Result = ();

    fn handle(&mut self, msg: Play, _: &mut Self::Context) -> Self::Result {
        tracing::info!("Play music: {}", msg.link.clone());
        for (_, listner) in &self.active_broadcasts {
            listner.do_send(S2C::Play {
                yt_link: msg.link.clone(),
            });
        }
        self.queue.push_back(msg.link);
    }
}

impl Handler<Skip> for DTPServer {
    type Result = ();

    fn handle(&mut self, _: Skip, _: &mut Self::Context) -> Self::Result {
        self.notify_skip();
    }
}

impl Handler<Ready2> for DTPServer {
    type Result = ();

    fn handle(&mut self, msg: Ready2, _: &mut Self::Context) -> Self::Result {
        tracing::info!("{} is ready to start", &msg.id);
        self.ready2_clients.insert(msg.id);
    }
}

impl Handler<Poll> for DTPServer {
    type Result = ();

    fn handle(&mut self, _: Poll, _: &mut Self::Context) -> Self::Result {
        if self.queue.len() > 0 {
            if self.active_broadcasts.len() == self.ready2_clients.len() {
                let _ = self.queue.pop_front().unwrap();
                for (_, listener) in &self.active_broadcasts {
                    listener.do_send(S2C::Start);
                }
                self.ready2_clients.clear();
            }
        }
    }
}

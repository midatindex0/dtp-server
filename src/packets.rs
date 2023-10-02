use actix::{Message, Recipient};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Message)]
#[rtype(result = "()")]
pub enum C2S {
    Play { yt_link: String },
    Skip,
    Ready1,
    Ready2,
    Playing,
}

#[derive(Serialize, Message)]
#[rtype(result = "()")]
pub enum S2C {
    ConnectNotification(ConnectNotification),
    DisconnectNotification(DisconnectNotification),
    Play { yt_link: String },
    Start,
    Skip,
}

#[derive(Clone, Debug, Message, Serialize)]
#[rtype(result = "()")]
pub struct ConnectNotification {
    pub id: String,
}

#[derive(Clone, Debug, Message, Serialize)]
#[rtype(result = "()")]
pub struct DisconnectNotification {
    pub id: String,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<S2C>,
    pub id: String,
    pub notif: ConnectNotification,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
    pub notif: DisconnectNotification,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Play {
    pub link: String,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Skip;

#[derive(Debug, Message)]
#[rtype(result = "Option<String>")]
pub struct Ready1;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Ready2 {
    pub id: String,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Poll;

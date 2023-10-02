use std::time::{Duration, Instant};

use actix::{
    fut, prelude::ContextFutureSpawner, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext,
    Handler, Running, StreamHandler, WrapFuture,
};
use actix_web_actors::ws::{self, WebsocketContext};

use crate::{
    packets::{
        Connect, ConnectNotification, Disconnect, DisconnectNotification, Play, Poll, Ready2, Skip,
        C2S, S2C,
    },
    server::DTPServer,
};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const SESSION_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct DTPSession {
    pub id: String,
    pub hb: Instant,
    pub playing: bool,
    pub addr: Addr<DTPServer>,
}

impl DTPSession {
    fn hb(&self, ctx: &mut WebsocketContext<Self>) {
        let server = self.addr.clone();
        ctx.run_interval(HEARTBEAT_INTERVAL, move |act, ctx| {
            if Instant::now().duration_since(act.hb) > SESSION_TIMEOUT {
                act.addr.do_send(Disconnect {
                    id: act.id.clone(),
                    notif: DisconnectNotification { id: act.id.clone() },
                });
                ctx.stop();
                return;
            }
            ctx.ping(b"");
            if !act.playing {
                server.do_send(Poll);
            }
        });
    }
}

impl Actor for DTPSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();

        self.addr
            .send(Connect {
                id: self.id.clone(),
                addr: addr.recipient(),
                notif: ConnectNotification {
                    id: self.id.clone(),
                },
            })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_) => {}
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.addr.do_send(Disconnect {
            id: self.id.clone(),
            notif: DisconnectNotification {
                id: self.id.clone(),
            },
        });
        Running::Stop
    }
}

impl Handler<S2C> for DTPSession {
    type Result = ();

    fn handle(&mut self, msg: S2C, ctx: &mut Self::Context) -> Self::Result {
        let msg = serde_json::to_string(&msg);
        match msg {
            Ok(s) => ctx.text(s),
            _ => {
                tracing::error!("Failed to serialize: {:?}", msg);
                ctx.stop();
            }
        };
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for DTPSession {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let item = match item {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(item) => item,
        };
        match item {
            ws::Message::Text(msg) => {
                let msg = msg.trim();

                let msg = serde_json::from_str::<C2S>(msg);
                match msg {
                    Ok(s) => match s {
                        C2S::Play { yt_link } => self.addr.do_send(Play { link: yt_link }),
                        C2S::Skip => self.addr.do_send(Skip),
                        C2S::Ready1 => {
                            self.playing = false;
                        }
                        C2S::Ready2 => self.addr.do_send(Ready2 {
                            id: self.id.clone(),
                        }),
                        C2S::Playing => self.playing = true,
                    },
                    Err(e) => {
                        tracing::warn!("Failed to deserialize request: {}", e);
                        ctx.stop();
                        return;
                    }
                }
            }
            ws::Message::Ping(x) => {
                self.hb = Instant::now();
                ctx.pong(&x);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

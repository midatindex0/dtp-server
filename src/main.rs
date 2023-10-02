mod packets;
mod server;
mod session;

use actix::{Actor, Addr};
use actix_web::{
    get,
    web::{self},
    App, Error, HttpRequest, HttpResponse, HttpServer,
};
use actix_web_actors::ws;
use server::DTPServer;
use session::DTPSession;
use std::time::Instant;

#[get("/{id}")]
pub async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<(String,)>,
    dtp_server: web::Data<Addr<DTPServer>>,
) -> Result<HttpResponse, Error> {
    let (id,) = path.into_inner();
    ws::start(
        DTPSession {
            id,
            hb: Instant::now(),
            addr: dtp_server.get_ref().clone(),
            playing: false,
        },
        &req,
        stream,
    )
}

struct DTPService;

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for DTPService {
    async fn bind(mut self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let server = HttpServer::new(move || {
            let dtp_server = DTPServer::default().start();

            App::new()
                .service(websocket)
                .app_data(web::Data::new(dtp_server))
        })
        .workers(1)
        .bind(addr)?
        .run();

        tokio::select!(
            _ = server  => {}
        );

        Ok(())
    }
}

#[shuttle_runtime::main]
async fn init() -> Result<DTPService, shuttle_runtime::Error> {
    Ok(DTPService)
}

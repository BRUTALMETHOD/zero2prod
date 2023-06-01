use actix_web::dev::Server;
use actix_web::{web, App, HttpServer, Result};
use std::net::TcpListener;

use crate::routes::{health_check, hello, subscribe};

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .service(hello)
            .route("/subscriptions", web::post().to(subscribe))
            .route("/health_check", web::get().to(health_check))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

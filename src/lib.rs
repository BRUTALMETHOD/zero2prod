use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::dev::Server;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}


async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn run() -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .service(hello)
            .route("/health_check", web::get().to(health_check))
    })
    .bind(("127.0.0.1", 8080))?
    .run();
    Ok(server)
}

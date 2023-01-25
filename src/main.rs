use actix_cors::Cors;
use actix_web::{get, middleware::Logger, App, HttpResponse, HttpServer, Responder};

#[get("/status")]
async fn status() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[get("/wms")]
async fn wms() -> impl Responder {
    HttpResponse::Ok().body("WMS")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let bind_port: u16 = std::env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting rewms server at http://localhost:{bind_port}");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(status)
            .service(wms)
    })
    .bind(("127.0.0.1", bind_port))?
    .run()
    .await
}

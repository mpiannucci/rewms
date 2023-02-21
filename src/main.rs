mod common;
mod wms;

use actix_cors::Cors;
use actix_web::{
    get,
    middleware::Logger,
    web::{Data},
    App, HttpResponse, HttpServer, Responder,
};
use awc::Client;

use crate::common::AppState;

#[get("/status")]
async fn status() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let bind_port: u16 = std::env::var("PORT")
        .unwrap_or("9080".to_string())
        .parse()
        .unwrap();

    let downstream = std::env::var("DOWNSTREAM").expect(
        "You must specify a downstream WMS server URL with the DOWNSTREAM environment variable.",
    );

    let downstream_parts = downstream.split("://").collect::<Vec<&str>>();
    let downstream_scheme = downstream_parts[0].to_string();
    let downstream_host = downstream_parts[1].to_string();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting rewms server at http://localhost:{bind_port}");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                wms_scheme: downstream_scheme.clone(),
                wms_host: downstream_host.clone(),
            }))
            .app_data(Data::new(Client::default()))
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(status)
            .service(wms::wms)
    })
    .bind(("0.0.0.0", bind_port))?
    .workers(1)
    .run()
    .await
}

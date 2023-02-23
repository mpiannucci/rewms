mod proxy;
mod state;
mod wms;

use actix_cors::Cors;
use actix_web::{
    get,
    middleware::Logger,
    web::{Data},
    App, HttpResponse, HttpServer, Responder,
};
use awc::Client;
use clap::{Parser, command, arg};

use crate::state::AppState;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    port: u16,

    #[arg(short, long)]
    wms_root: String,
}

#[get("/status")]
async fn status() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Args = Args::parse();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting rewms server at http://localhost:{port}", port=args.port);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState::new(&args.wms_root)))
            .app_data(Data::new(Client::default()))
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(status)
            .service(wms::wms)
    })
    .bind(("0.0.0.0", args.port))?
    .workers(1)
    .run()
    .await
}

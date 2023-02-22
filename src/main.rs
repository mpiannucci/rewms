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
use clap::{Parser, command, arg};

use crate::common::AppState;

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

    let wms_parts = args.wms_root.split("://").collect::<Vec<&str>>();
    let wms_scheme = wms_parts[0].to_string();
    let wms_path_parts = wms_parts[1].split("/").collect::<Vec<&str>>();
    let wms_host = wms_path_parts[0].to_string();
    let wms_path = if wms_path_parts.len() > 1 {
        format!("/{path}", path=wms_path_parts[1..].join("/"))
    } else {
        "".to_string()
    };

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting rewms server at http://localhost:{port}", port=args.port);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                wms_scheme: wms_scheme.clone(),
                wms_host: wms_host.clone(),
                wms_path: wms_path.clone(),
            }))
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

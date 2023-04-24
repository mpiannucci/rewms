mod error;
mod proxy;
mod state;
mod wms;

use std::net::SocketAddr;

use axum::{routing::get, Router, Server};
use clap::{arg, command, Parser};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use crate::state::AppState;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    #[clap(default_value = "9080")]
    port: u16,

    #[arg(short, long)]
    wms_root: String,

    #[arg(short, long)]
    #[clap(default_value = "1")]
    workers: usize,
}

async fn status() -> &'static str {
    "Ok"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args: Args = Args::parse();

    let app_state = AppState::new(&args.wms_root);

    let app = Router::new()
        .route("/status", get(status))
        .route("/wms", get(wms::wms))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    tracing::info!(
        "starting rewms server at http://localhost:{port}",
        port = args.port
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await;        
}

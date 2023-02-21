use std::time::Duration;

use actix_web::{HttpResponse, error};
use awc::Client;

pub struct AppState {
    pub wms_scheme: String,
    pub wms_host: String,
}

pub async fn proxy(client: &Client, url: String) -> actix_web::Result<HttpResponse> {
    client
        .get(url)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(error::ErrorInternalServerError)
        .and_then(|resp| Ok::<HttpResponse, error::Error>(HttpResponse::Ok().streaming(resp)))
}
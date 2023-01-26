use std::time::Duration;

use actix_cors::Cors;
use actix_web::{
    error, get,
    middleware::Logger,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use awc::Client;
use serde::Deserialize;

async fn proxy(client: &Client, url: String) -> actix_web::Result<HttpResponse> {
    client
        .get(url)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(error::ErrorInternalServerError)
        .and_then(|resp| Ok::<HttpResponse, error::Error>(HttpResponse::Ok().streaming(resp)))
}

struct AppState {
    pub downstream: String,
}

#[get("/status")]
async fn status() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

// https://eds.ioos.us/wms/?service=WMS&request=GetMap&version=1.1.1&layers=GFS_WAVE_ATLANTIC/Significant_height_of_combined_wind_waves_and_swell_surface&styles=raster%2Fx-Occam&colorscalerange=0%2C10&units=m&width=256&height=256&format=image/png&transparent=true&time=2023-01-26T00:00:00.000Z&srs=EPSG:3857&bbox=-7827151.696402049,4383204.9499851465,-7514065.628545966,4696291.017841227

#[derive(Deserialize, Clone, Debug)]
struct WmsParams {
    #[serde(alias = "service", alias = "SERVICE")]
    service: String,
    #[serde(alias = "request", alias = "REQUEST")]
    request: String,
    #[serde(alias = "version", alias = "VERSION")]
    version: String,
    #[serde(alias = "layers", alias = "LAYERS")]
    layers: String,
    #[serde(alias = "styles", alias = "STYLES")]
    styles: Option<String>,
    #[serde(alias = "bbox", alias = "BBOX")]
    bbox: String,
    #[serde(alias = "width", alias = "WIDTH")]
    width: u32,
    #[serde(alias = "height", alias = "HEIGHT")]
    height: u32,
    #[serde(alias = "srs", alias = "SRS")]
    srs: String,
    #[serde(alias = "time", alias = "TIME")]
    time: Option<String>,
    #[serde(alias = "elevation", alias = "ELEVATION")]
    elevation: Option<i32>,
    #[serde(alias = "colorscalerange", alias = "COLORSCALERANGE")]
    colorscalerange: Option<String>,
}

impl WmsParams {
    fn passthrough_request(&self) -> bool {
        let Some(styles) = self.styles.as_ref() else {
            return true;
        };

        self.request != "GetMap" || (!styles.starts_with("values/") && !styles.starts_with("particles/"))
    }
}

#[get("/wms/")]
async fn wms(
    client: web::Data<Client>,
    app_state: web::Data<AppState>,
    req: HttpRequest,
    params: web::Query<WmsParams>,
) -> actix_web::Result<HttpResponse> {
    // For now we are only hijacking requests if the user is asking for a values or particles style
    if params.passthrough_request() {
        let downstream_request = format!("{}/?{}", app_state.downstream, req.query_string());
        return proxy(client.as_ref(), downstream_request).await;
    }

    let parm = format!("{}", params.request);
    Ok(HttpResponse::Ok().body(parm))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let bind_port: u16 = std::env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap();

    let downstream = std::env::var("DOWNSTREAM").expect(
        "You must specify a downstream WMS server URL with the DOWNSTREAM environment variable.",
    );

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting rewms server at http://localhost:{bind_port}");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                downstream: downstream.clone(),
            }))
            .app_data(Data::new(Client::default()))
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(status)
            .service(wms)
    })
    .bind(("127.0.0.1", bind_port))?
    .run()
    .await
}

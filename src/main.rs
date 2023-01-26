use actix_cors::Cors;
use actix_web::{get, middleware::Logger, App, HttpResponse, HttpServer, Responder, web::{self, Data}, HttpRequest, error};
use reqwest::Client;
use serde::Deserialize;

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
    service: String,
    request: String,
    version: String,
    layers: String,
    styles: String,
    bbox: String,
    width: String,
    height: String,
    format: String,
    transparent: String,
    srs: String,
    time: String,
    elevation: Option<String>,
    colorscalerange: String,
    abovemaxcolor: Option<String>,
    belowmincolor: Option<String>,
}

#[get("/wms/")]
async fn wms(app_state: web::Data<AppState>, req: HttpRequest, params: web::Query<WmsParams>) -> impl Responder {
    if params.request == "GetMap" {
        // Return downstream response
        let downstream_request = format!("{}/?{}", app_state.downstream, req.query_string());

        let client = awc::Client::new();
        return client
            .get(downstream_request)
            .send()
            .await
            .map_err(error::ErrorInternalServerError)
            .and_then(|resp| Ok::<HttpResponse, error::Error>(HttpResponse::Ok().streaming(resp)));
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
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(status)
            .service(wms)
    })
    .bind(("127.0.0.1", bind_port))?
    .run()
    .await
}

use actix_web::{get, http::Uri, web, HttpRequest, HttpResponse};
use awc::Client;
use futures::future::join_all;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::common::{proxy, AppState};

// https://eds.ioos.us/wms/?service=WMS&request=GetMap&version=1.1.1&layers=GFS_WAVE_ATLANTIC/Significant_height_of_combined_wind_waves_and_swell_surface&styles=raster%2Fx-Occam&colorscalerange=0%2C10&units=m&width=256&height=256&format=image/png&transparent=true&time=2023-01-26T00:00:00.000Z&srs=EPSG:3857&bbox=-7827151.696402049,4383204.9499851465,-7514065.628545966,4696291.017841227

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WmsMetadata {
    pub scale_range: (f64, f64),
    pub nearest_time_iso: String,
    pub units: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WmsMinMax {
    pub min: f64,
    pub max: f64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WmsParams {
    #[serde(alias = "request", alias = "REQUEST")]
    pub request: String,
    #[serde(alias = "version", alias = "VERSION")]
    pub version: String,
    #[serde(alias = "layers", alias = "LAYERS")]
    pub layers: String,
    #[serde(alias = "styles", alias = "STYLES")]
    pub styles: Option<String>,
    #[serde(alias = "bbox", alias = "BBOX")]
    pub bbox: String,
    #[serde(alias = "width", alias = "WIDTH")]
    pub width: u32,
    #[serde(alias = "height", alias = "HEIGHT")]
    pub height: u32,
    #[serde(alias = "srs", alias = "SRS")]
    pub srs: String,
    #[serde(alias = "time", alias = "TIME")]
    pub time: Option<String>,
    #[serde(alias = "elevation", alias = "ELEVATION")]
    pub elevation: Option<i32>,
    #[serde(alias = "colorscalerange", alias = "COLORSCALERANGE")]
    pub colorscalerange: Option<String>,
    #[serde(alias = "units", alias = "UNITS")]
    pub units: Option<String>,
}

impl WmsParams {
    fn passthrough_request(&self) -> bool {
        let Some(styles) = self.styles.as_ref() else {
            return true;
        };

        self.request != "GetMap"
            || (!styles.starts_with("values/") && !styles.starts_with("particles/"))
    }

    pub fn parse_layers(&self) -> Vec<String> {
        self.layers
            .split(",")
            .flat_map(|l| {
                if l.ends_with("-group") {
                    l.split("group")
                        .next()
                        .unwrap()
                        .split(":")
                        .map(|s| s.to_string())
                        .collect()
                } else {
                    vec![l.to_string()]
                }
            })
            .collect()
    }

    pub fn parse_colorscalerange(&self) -> (f64, f64) {
        let range: Vec<f64> = self
            .colorscalerange
            .as_ref()
            .unwrap_or(&"".to_string())
            .split(",")
            .map(|s| s.parse::<f64>().unwrap())
            .collect();
        (range[0], range[1])
    }

    pub fn get_metadata_url(&self, downstream: &str, layer: &str) -> Uri {
        Uri::builder()
            .scheme("https")
            .authority(downstream.split("/").next().unwrap())
            .path_and_query(format!(
                "/ncWMS2/wms/?service=WMS&request=GetMetadata&version=1.1.1&item=layerDetails&layername={layer}",
            ))
            .build()
            .unwrap()
    }

    pub fn get_minmax_url(&self, downstream: &str, layer: &str) -> Uri {
        Uri::builder()
            .scheme("https")
            .authority(downstream.split("/").next().unwrap())
            .path_and_query(format!(
                "/ncWMS2/wms/?service=WMS&request=GetMetadata&version=1.1.1&item=minmax&layername={layer}&layers={layer}&styles=&srs={}&bbox={}&width={}&height={}", self.srs, self.bbox, self.width, self.height
            ))
            .build()
            .unwrap()
    }

    pub fn get_reference_map_url(&self, downstream: &str, layer: &str, minmax: &WmsMinMax) -> Uri {
        Uri::builder()
        .scheme("https")
        .authority(downstream.split("/").next().unwrap())
        .path_and_query(format!(
            "/ncWMS2/wms/?service=WMS&request=GetMap&version=1.1.1&layers={layer}&styles=raster/seq-Greys-inv&format=image/png;mode=32bit&transparent=true&srs={}&bbox={}&width={}&height={}&colorscalerange={},{}",
            self.srs, self.bbox, self.width, self.height, minmax.min, minmax.max
        ))
        .build()
        .unwrap()
    }
}

#[get("/wms/")]
pub async fn wms(
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

    let layers = params.parse_layers();
    let metadata_futures = layers.iter().flat_map(|l| {
        let metadata_url = params.get_metadata_url(&app_state.downstream, l);
        let minmax_url = params.get_minmax_url(&app_state.downstream, l);
        let metadata = client.get(metadata_url).send();
        let minmax = client.get(minmax_url).send();
        vec![metadata, minmax]
    });

    let mut metadata = join_all(metadata_futures).await;
    // let metadata_futures = metadata
    //     .iter_mut()
    //     .enumerate()
    //     .step_by(2)
    //     .flat_map(|(i, m)| {
    //         let meta = m.as_mut().unwrap().json::<WmsMetadata>();
    //         let minmax = metadata[i + 1].as_mut().unwrap().json::<WmsMinMax>();
    //         vec![meta]
    //     });
    // let metadata = join_all(metadata_futures).await;

    let mut metadata_unpacked = vec![];
    let mut minmax_unpacked = vec![];
    for (i, m) in metadata.iter_mut().enumerate() {
        if i % 2 == 0 {
            let meta = m.as_mut().unwrap().json::<WmsMetadata>();
            metadata_unpacked.push(meta);
        } else {
            let minmax = m.as_mut().unwrap().json::<WmsMinMax>();
            minmax_unpacked.push(minmax);
        }
    }

    let metadata_unpacked = join_all(metadata_unpacked)
        .await
        .iter()
        .map(|m| m.as_ref().unwrap().clone())
        .collect::<Vec<_>>();

    let minmax_unpacked = join_all(minmax_unpacked)
        .await
        .iter()
        .map(|m| m.as_ref().unwrap().clone())
        .collect::<Vec<_>>();

    let parm = format!(
        "{}",
        params.get_reference_map_url(&app_state.downstream, "", &WmsMinMax { min: 0., max: 6. })
    );
    Ok(HttpResponse::Ok().body(parm))
}
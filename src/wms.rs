use std::{io::Cursor, time::Duration};

use actix_web::{
    get,
    http::Uri,
    web::{self},
    HttpRequest, HttpResponse,
};
use awc::Client;
use futures::future::join_all;
use image::ImageOutputFormat;
use serde::{Deserialize, Serialize};

use crate::common::{proxy, AppState};

// https://eds.ioos.us/wms/?service=WMS&request=GetMap&version=1.1.1&layers=GFS_WAVE_ATLANTIC/Significant_height_of_combined_wind_waves_and_swell_surface&styles=raster%2Fx-Occam&colorscalerange=0%2C10&units=m&width=256&height=256&format=image/png&transparent=true&time=2023-01-26T00:00:00.000Z&srs=EPSG:3857&bbox=-7827151.696402049,4383204.9499851465,-7514065.628545966,4696291.017841227
// https://eds.ioos.us/wms/?service=WMS&request=GetMap&version=1.1.1&layers=GFS_WAVE_ATLANTIC/Significant_height_of_combined_wind_waves_and_swell_surface&styles=values/default&colorscalerange=0%2C10&units=m&width=256&height=256&format=image/png&transparent=true&time=2023-01-26T00:00:00.000Z&srs=EPSG:3857&bbox=-7827151.696402049,4383204.9499851465,-7514065.628545966,4696291.017841227

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
    pub layers: Option<String>,
    #[serde(alias = "styles", alias = "STYLES")]
    pub styles: Option<String>,
    #[serde(alias = "bbox", alias = "BBOX")]
    pub bbox: Option<String>,
    #[serde(alias = "width", alias = "WIDTH")]
    pub width: Option<u32>,
    #[serde(alias = "height", alias = "HEIGHT")]
    pub height: Option<u32>,
    #[serde(alias = "srs", alias = "SRS")]
    pub srs: Option<String>,
    #[serde(alias = "time", alias = "TIME")]
    pub time: Option<String>,
    #[serde(alias = "elevation", alias = "ELEVATION")]
    pub elevation: Option<i32>,
    #[serde(alias = "units", alias = "UNITS")]
    pub units: Option<String>,
}

impl WmsParams {
    fn passthrough_request(&self) -> bool {
        let Some(styles) = self.styles.as_ref() else {
            return true;
        };

        self.request != "GetMap" || !styles.starts_with("values/")
    }

    pub fn parse_layers(&self) -> Vec<String> {
        self.layers
            .clone()
            .unwrap()
            .split(",")
            .flat_map(|l| {
                if l.contains("-") {
                    l.split("-")
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

    pub fn get_minmax_url(
        &self,
        wms_scheme: &str,
        wms_host: &str,
        wms_path: &str,
        layer: &str,
    ) -> Uri {
        let elevation = self
            .elevation
            .map(|e| format!("&elevation={e}"))
            .unwrap_or("".to_string());
        let time = self
            .time
            .as_ref()
            .map(|t| format!("&time={t}"))
            .unwrap_or("".to_string());
        let path = format!("{wms_path}/wms/?service=WMS&request=GetMetadata&version=1.1.1&item=minmax&layername={layer}&layers={layer}&styles=&srs={srs}&bbox={bbox}&width={width}&height={height}{elevation}{time}", srs=self.srs.clone().unwrap(), bbox=self.bbox.clone().unwrap(), width=self.width.unwrap(), height=self.height.unwrap());

        Uri::builder()
            .scheme(wms_scheme)
            .authority(wms_host)
            .path_and_query(path)
            .build()
            .unwrap()
    }

    pub fn get_reference_map_url(
        &self,
        wms_scheme: &str,
        wms_host: &str,
        wms_path: &str,
        layer: &str,
        minmax: &WmsMinMax,
    ) -> Uri {
        let elevation = self
            .elevation
            .map(|e| format!("&elevation={e}"))
            .unwrap_or("".to_string());
        let time = self
            .time
            .as_ref()
            .map(|t| format!("&time={t}"))
            .unwrap_or("".to_string());
        let path = format!("{wms_path}/wms/?service=WMS&request=GetMap&version=1.1.1&layers={layer}&styles=raster/seq-GreysRev&format=image/png;mode=32bit&transparent=true&srs={srs}&bbox={bbox}&width={width}&height={height}&colorscalerange={min},{max}&numcolorbands=250{elevation}{time}",
        srs=self.srs.clone().unwrap(), bbox=self.bbox.clone().unwrap(), width=self.width.unwrap(), height=self.height.unwrap(), min=minmax.min, max=minmax.max);

        Uri::builder()
            .scheme(wms_scheme)
            .authority(wms_host)
            .path_and_query(path)
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
        let downstream_request = format!(
            "{}://{}{}/wms/?{}",
            app_state.wms_scheme,
            app_state.wms_host,
            app_state.wms_path,
            req.query_string()
        );
        return proxy(client.as_ref(), downstream_request).await;
    }

    let layers = params.parse_layers();
    let minmax_futures = layers.iter().map(|l| {
        let minmax_url = params.get_minmax_url(
            &app_state.wms_scheme,
            &app_state.wms_host,
            &app_state.wms_path,
            l,
        );
        client
            .get(minmax_url)
            .timeout(Duration::from_secs(60))
            .send()
    });

    let mut minmax = join_all(minmax_futures).await;

    let mut minmax_unpacked = vec![];
    for m in minmax.iter_mut() {
        let mm = m.as_mut().unwrap();
        let minmax = mm.json::<WmsMinMax>();
        minmax_unpacked.push(minmax);
    }

    let mut minmax_unpacked = join_all(minmax_unpacked)
        .await
        .iter()
        .map(|m| m.as_ref().unwrap().clone())
        .collect::<Vec<_>>();

    let reference_url_futures = layers.iter().enumerate().map(|(i, l)| {
        let minmax = &minmax_unpacked[i];
        let url = params.get_reference_map_url(
            &app_state.wms_scheme,
            &app_state.wms_host,
            &app_state.wms_path,
            l,
            minmax,
        );
        client.get(url).timeout(Duration::from_secs(60)).send()
    });

    let mut raw_reference_images = join_all(reference_url_futures).await;
    let reference_images = raw_reference_images
        .iter_mut()
        .map(|r| r.as_mut().unwrap().body());

    let mut reference_images = join_all(reference_images)
        .await
        .iter()
        .map(|r| {
            image::load_from_memory(r.as_ref().unwrap().as_ref())
                .unwrap()
                .to_rgba8()
        })
        .collect::<Vec<_>>();

    let mut image_data = reference_images.pop().unwrap();

    let ref_min_max = minmax_unpacked.pop().unwrap();

    // This is done to match pyxms's behaviour, which uses matplotlibs colormapping and linspace to
    // directly map values to color bins. This might not be more accurate, for that we may eventually go back
    // to using straight linear scaling from min to max.
    let step = (ref_min_max.max as f32 - ref_min_max.min as f32) / 249.0;

    image_data
        .pixels_mut()
        .for_each(|pixel| {
            pixel.0 = if pixel[3] == 0 {
                [0; 4]
            } else {
                let raw_value = pixel[0];
                if raw_value == 0 {
                    [255; 4]
                } else {
                    let step_i = ((raw_value as f32 / 255.0) / (1.0 / 250.0)).ceil();
                    let v: f32 = step_i * step + ref_min_max.min as f32;
                    v.to_le_bytes()
                }
            };
        });

    let mut w = Cursor::new(Vec::new());
    image_data.write_to(&mut w, ImageOutputFormat::Png).unwrap();
    let raw = w.into_inner();

    let response = HttpResponse::Ok()
        .content_type("image/png;mode=32bit")
        .body(raw);

    Ok(response)
}

#[cfg(test)]
mod tests {
    use image::{ImageBuffer, Rgba};
    use rayon::prelude::*;

    use super::*;

    fn pixels_to_float(im: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Vec<f32> {
        (0..im.width() * im.height() * 4)
            .into_par_iter()
            .enumerate()
            .step_by(4)
            .map(|(i, _)| {
                let x = (i / 4) as u32 % 256;
                let y = (i / 4) as u32 / 256;
                let pixel = im.get_pixel(x, y);
                f32::from_le_bytes(pixel.0)
            })
            .collect()
    }

    #[test]
    fn render_matching_pyxms_values() {
        let mut image = image::open("tests/data/greys-rev.png").unwrap().to_rgba8();

        let min_max = WmsMinMax {
            min: 1.08,
            max: 2.02,
        };

        let step = (min_max.max as f32 - min_max.min as f32) / 249.0;
        image
            .pixels_mut()
            .for_each(|pixel| {
                pixel.0 = if pixel[3] == 0 {
                    [0; 4]
                } else {
                    let raw_value = pixel[0];
                    if raw_value == 0 {
                        [255; 4]
                    } else {
                        let step_i = ((raw_value as f32 / 255.0) / (1.0 / 250.0)).floor();
                        let v: f32 = step_i * step + min_max.min as f32;
                        v.to_le_bytes()
                    }
                };
            });

        let _ = image.save("tests/data/values-rs.png");

        let rendered_vals = pixels_to_float(&image);

        let truth_im = image::open("tests/data/values-new.png").unwrap().to_rgba8();
        let truth_vals = pixels_to_float(&truth_im);

        // println!("{:?}", &rendered_vals[100..108]);
        // println!("{:?}", &truth_vals[100..108]);
        let mut hits = 0;
        for i in 0..rendered_vals.len() {
            if (rendered_vals[i] - truth_vals[i]).abs() >= 0.01 {
                hits += 1;
                println!("{} -- {}", rendered_vals[i], truth_vals[i]);
                // assert!((rendered_vals[i] - truth_vals[i]).abs() < 0.01);
            }
        }

        println!("hits: {hits}");
    }
}

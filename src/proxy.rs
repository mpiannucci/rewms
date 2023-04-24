use axum::{response::IntoResponse, body::StreamBody};

use crate::error::{WmsError, wms_error_downstream};


pub async fn proxy(url: String) -> Result<impl IntoResponse, WmsError> {
    let stream = reqwest::get(url)
        .await
        .map_err(wms_error_downstream)?
        .bytes_stream();

    Ok(StreamBody::new(stream))
}
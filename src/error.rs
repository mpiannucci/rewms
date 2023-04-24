use axum::response::{IntoResponse, Response};
use http::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WmsError {
    #[error("Invalid layer in WMS request")]
    InvalidLayer,
    #[error("Downstream wms error")]
    DownstreamWmsError,
    #[error("Image error")]
    ImageError
}

impl IntoResponse for WmsError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("WMS Error: {}", self)).into_response()
    }
}

pub fn wms_error_downstream(e: reqwest::Error) -> WmsError {
    tracing::error!("Downstream wms error: {}", e);
    WmsError::DownstreamWmsError
}

pub fn wms_error_image(e: image::ImageError) -> WmsError {
    tracing::error!("Image error: {}", e);
    WmsError::ImageError
}
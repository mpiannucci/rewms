use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub wms_scheme: String,
    pub wms_host: String,
    pub wms_path: String,
}

impl AppState {
    pub fn new(wms_root_url: &str) -> Self {
        let wms_parts = wms_root_url.split("://").collect::<Vec<&str>>();
        let wms_scheme = wms_parts[0].to_string();
        let wms_path_parts = wms_parts[1].split("/").collect::<Vec<&str>>();
        let wms_host = wms_path_parts[0].to_string();
        let wms_path = if wms_path_parts.len() > 1 {
            format!("/{path}", path = wms_path_parts[1..].join("/"))
        } else {
            "".to_string()
        };

        Self {
            wms_scheme,
            wms_host, 
            wms_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppState;

    #[test]
    fn create_app_state() {
        let root = "https://eds.ioos.us/ncWMS2";

        let state = AppState::new(root);
        assert_eq!(state.wms_scheme, "https");
        assert_eq!(state.wms_host, "eds.ioos.us");
        assert_eq!(state.wms_path, "/ncWMS2");
    }
}
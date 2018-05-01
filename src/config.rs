use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub recent: Option<RecentSettings>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RecentSettings {
    pub map_path: Option<PathBuf>,
    pub map_width: Option<u8>,
    pub map_height: Option<u8>,
    pub tileset_path: Option<String>,
    pub blockset_path: Option<String>,
}

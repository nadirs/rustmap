use std::path::PathBuf;

#[derive(Debug,Default,Deserialize)]
pub struct Config {
    pub recent: Option<RecentSettings>
}

#[derive(Default,Debug,Deserialize)]
pub struct RecentSettings {
    pub map_path: Option<PathBuf>,
    pub tileset_path: Option<String>,
    pub blockset_path: Option<String>,
}

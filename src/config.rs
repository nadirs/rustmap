#[derive(Debug,Deserialize)]
pub struct Config {
    pub recent: RecentSettings
}

#[derive(Debug,Deserialize)]
pub struct RecentSettings {
    pub map_path: Option<String>,
    pub tileset_path: Option<String>,
    pub blockset_path: Option<String>,
}

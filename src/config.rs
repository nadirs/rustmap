#[derive(Debug,Deserialize)]
pub struct Config {
    pub recent: Option<RecentSettings>
}

impl Default for Config {
    fn default() -> Self {
        Config {
            recent: None,
        }
    }
}

#[derive(Debug,Deserialize)]
pub struct RecentSettings {
    pub map_path: Option<String>,
    pub tileset_path: Option<String>,
    pub blockset_path: Option<String>,
}

use serde::{Serialize, Deserialize};

pub type StorePluginList = Vec<StorePlugin>;

#[derive(Serialize, Deserialize, Clone)]
pub struct StorePlugin {
    pub id: usize,
    pub name: String,
    pub versions: Vec<StorePluginVersion>,
    pub author: String,
    pub description: String,
    pub tags: Vec<String>,
    pub image_url: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StorePluginVersion {
    pub name: String,
    pub hash: String,
    pub artifact: Option<String>,
}

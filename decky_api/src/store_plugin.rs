/*export interface StorePluginVersion {
  name: string;
  hash: string;
  artifact: string | undefined | null;
}

export interface StorePlugin {
  id: number;
  name: string;
  versions: StorePluginVersion[];
  author: string;
  description: string;
  tags: string[];
  image_url: string;
}*/

use serde::{Serialize, Deserialize};

pub type StorePluginList = Vec<StorePlugin>;

#[derive(Serialize, Deserialize, Clone)]
pub struct StorePlugin {
    id: usize,
    name: String,
    versions: Vec<StorePluginVersion>,
    author: String,
    description: String,
    tags: Vec<String>,
    image_url: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StorePluginVersion {
    pub name: String,
    pub hash: String,
    pub artifact: Option<String>,
}

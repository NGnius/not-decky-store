use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use decky_api::{StorePlugin, StorePluginList, StorePluginVersion};

use serde::{Serialize, Deserialize};

use super::IStorage;

#[derive(Serialize, Deserialize, Clone)]
pub struct PluginMetadata {
    id: usize,
    author: String,
    description: String,
    tags: Vec<String>,
}

impl PluginMetadata {
    fn complete(self, name: String, versions: Vec<StorePluginVersion>, image: String) -> StorePlugin {
        StorePlugin {
            id: self.id,
            name: name,
            versions: versions,
            author: self.author,
            description: self.description,
            tags: self.tags,
            image_url: image,
        }
    }
}

pub struct FileStorage {
    stats: Option<RwLock<HashMap<String, AtomicU64>>>, // TODO collect hit counts on actions
    root: PathBuf,
    domain_root: String,
}

impl FileStorage {
    pub fn new(root: PathBuf, domain_root: String, enable_stats: bool) -> Self {
        Self {
            root: root,
            domain_root: domain_root,
            stats: if enable_stats { Some(RwLock::new(HashMap::new())) } else { None },
        }
    }

    fn plugins_path(&self) -> PathBuf {
        self.root.join("plugins")
    }

    fn plugin_json_path(&self, plugin_root: impl AsRef<Path>) -> PathBuf {
        plugin_root.as_ref().join("plugin.json")
    }

    fn plugin_root_path(&self, plugin_name: &str) -> PathBuf {
        self.plugins_path().join(plugin_name)
    }

    fn plugin_artifact_path(&self, plugin_name: &str, version_name: &str, _hash: &str) -> PathBuf {
        self.plugin_root_path(plugin_name)
            .join(format!("{}.zip", version_name))
    }

    fn plugin_image_path(&self, plugin_name: &str) -> PathBuf {
        self.plugin_root_path(plugin_name)
            .join(format!("image.png"))
    }

    fn read_all_plugins(&self) -> std::io::Result<StorePluginList> {
        let plugins = self.plugins_path();
        let dir_reader = plugins.read_dir()?;
        let mut results = Vec::with_capacity(dir_reader.size_hint().1.unwrap_or(32));
        for entry in dir_reader {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                results.push(self.read_single_plugin(&entry.path())?);
            }
        }
        // build stats counters
        if let Some(stats) = &self.stats {
            let mut lock = stats.write().expect("Couldn't acquire stats write lock");
            for plugin in &results {
                for version in &plugin.versions {
                    if !lock.contains_key(&version.hash) {
                        lock.insert(version.hash.clone(), AtomicU64::new(0));
                    }
                }
            }
        }
        Ok(results)
    }

    fn read_single_plugin(&self, path: &PathBuf) -> std::io::Result<StorePlugin> {
        let plugin_name = path.file_name().unwrap().to_string_lossy().into_owned();
        let json_path = self.plugin_json_path(path);
        let plugin_info: PluginMetadata = match serde_json::from_reader(File::open(&json_path)?) {
            Ok(x) => x,
            Err(e) => {
                log::error!("`{}` JSON err: {}", json_path.display(), e);
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            }
        };
        // find plugin versions
        let dir_reader = path.read_dir()?;
        let mut versions = Vec::with_capacity(dir_reader.size_hint().1.unwrap_or(4));
        for entry in dir_reader {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let entry_path = entry.path();
                let extension = entry_path.extension().unwrap().to_string_lossy().into_owned();
                if extension == "zip" {
                    let version_name = entry_path.file_stem().unwrap().to_string_lossy().into_owned();
                    let hash_str = sha256::try_digest(entry_path.as_ref())?;
                    let artifact_url = format!("{}/plugins/{}/{}/{}.zip", self.domain_root, plugin_name, version_name, hash_str);
                    versions.push(StorePluginVersion {
                        name: version_name,
                        hash: hash_str,
                        artifact: Some(artifact_url)
                    });
                }
            }
        }
        versions.sort_by(|a, b| b.name.cmp(&a.name)); // sort e.g. v2 before v1
        let image_url = format!("{}/plugins/{}.png", self.domain_root, plugin_name);
        Ok(
            plugin_info.complete(
                plugin_name,
                versions,
                image_url,
            )
        )
    }
}

impl IStorage for FileStorage {
    fn plugins(&self) -> StorePluginList {
        match self.read_all_plugins() {
            Err(e) => {
                log::error!("Plugins read error: {}", e);
                vec![]
            },
            Ok(x) => x
        }
    }

    fn get_artifact(&self, name: &str, version: &str, hash: &str) -> Result<bytes::Bytes, std::io::Error> {
        let path = self.plugin_artifact_path(name, version, hash);
        log::debug!("Opening artifact path: {}", path.display());
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        if let Some(stats) = &self.stats {
            let lock = stats.read().expect("Failed to acquire stats read lock");
            if let Some(counter) = lock.get(hash) {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        }
        Ok(buffer.into())
    }

    fn get_image(&self, name: &str) -> Result<bytes::Bytes, std::io::Error> {
        let path = self.plugin_image_path(name);
        log::debug!("Opening image path: {}", path.display());
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer.into())
    }

    fn get_statistics(&self) -> std::collections::HashMap<String, u64> {
        if let Some(stats) = &self.stats {
            if let Ok(plugins) = self.read_all_plugins() {
                let lock = stats.read().expect("Failed to acquire stats read lock");
                let mut map = std::collections::HashMap::with_capacity(lock.len());
                for plugin in plugins {
                    let mut total = 0;
                    for version in plugin.versions {
                        if let Some(count) = lock.get(&version.hash) {
                            let count_val = count.load(Ordering::SeqCst);
                            total += count_val;
                            map.insert(format!("{} {}", plugin.name, version.name), count_val);
                        }
                    }
                    map.insert(format!("{}", plugin.name), total);
                }
                map
            } else {
                std::collections::HashMap::with_capacity(0)
            }
        } else {
            std::collections::HashMap::with_capacity(0)
        }
    }
}

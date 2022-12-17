use std::collections::HashMap;
use std::sync::RwLock;

use decky_api::{StorePluginList, StorePlugin};

use super::IStorage;

struct StoreIndex(usize);
#[derive(Hash, Eq, PartialEq)]
struct StoreName(String);

#[derive(Hash, Eq, PartialEq)]
struct HashablePluginVersion {
    plugin_name: String,
    version_name: String,
    hash: String,
}

pub struct MergedStorage<S: AsRef<dyn IStorage> + Send + Sync> {
    stores: Vec<S>,
    store_artifact_map: RwLock<HashMap<HashablePluginVersion, StoreIndex>>,
    store_image_map: RwLock<HashMap<StoreName, Vec<StoreIndex>>>,
}

impl<S: AsRef<dyn IStorage> + Send + Sync> MergedStorage<S> {
    pub fn new(inner: Vec<S>) -> Self {
        Self {
            stores: inner,
            store_artifact_map: RwLock::new(HashMap::new()),
            store_image_map: RwLock::new(HashMap::new()),
        }
    }

    /*pub fn add(mut self, store: S) -> Self {
        self.stores.push(store);
        self
    }*/

    fn map_to_vec(plugins: HashMap<StoreName, StorePlugin>) -> StorePluginList {
        let mut result = Vec::with_capacity(plugins.len());
        for (_, val) in plugins {
            result.push(val);
        }
        result
    }

    fn merge_plugins_into(dest: &mut HashMap<StoreName, StorePlugin>, source: StorePluginList) {
        for mut plugin in source {
            let store_name = StoreName(plugin.name.clone());
            if let Some(existing_plugin) = dest.get_mut(&store_name) {
                // combine versions if the plugin has the same name as an existing one
                existing_plugin.versions.append(&mut plugin.versions);
            } else {
                // create new plugin entry if not
                dest.insert(store_name, plugin);
            }
        }
    }

    fn merge_statistics_into(dest: &mut HashMap<String, u64>, source: HashMap<String, u64>) {
        for (entry, val) in source {
            if let Some(existing_stat) = dest.get_mut(&entry) {
                // combine if already exists
                *existing_stat += val;
            } else {
                // create new plugin entry if not
                dest.insert(entry, val);
            }
        }
    }
}

impl<S: AsRef<dyn IStorage> + Send + Sync> IStorage for MergedStorage<S> {
    fn plugins(&self) -> StorePluginList {
        let mut merged_plugins = HashMap::new();
        log::debug!("Acquiring store map write locks");
        let mut arti_lock = self.store_artifact_map.write().expect("Failed to acquire store_artifact_map write lock");
        let mut img_lock = self.store_image_map.write().expect("Failed to acquire store_image_map write lock");
        for (index, store) in self.stores.iter().enumerate() {
            let plugins = store.as_ref().plugins();
            // re-build store mappins
            for plugin in &plugins {
                for version in &plugin.versions {
                    let hashable_ver = HashablePluginVersion {
                        plugin_name: plugin.name.clone(),
                        version_name: version.name.clone(),
                        hash: version.hash.clone(),
                    };
                    arti_lock.insert(hashable_ver, StoreIndex(index));
                }
                let store_name = StoreName(plugin.name.clone());
                if let Some(stores) = img_lock.get_mut(&store_name) {
                    stores.push(StoreIndex(index));
                } else {
                    img_lock.insert(store_name, vec![StoreIndex(index)]);
                }
            }
            Self::merge_plugins_into(&mut merged_plugins, plugins);
        }
        Self::map_to_vec(merged_plugins)
    }

    fn get_artifact(&self, name: &str, version: &str, hash: &str) -> Result<bytes::Bytes, std::io::Error> {
        log::debug!("Acquiring store_artifact_map read lock");
        let lock = self.store_artifact_map.read().expect("Failed to acquire store_artifact_map read lock");
        if let Some(index) = lock.get(&HashablePluginVersion {
            plugin_name: name.to_owned(),
            version_name: version.to_owned(),
            hash: hash.to_owned(),
        }) {
            if let Some(store) = self.stores.get(index.0) {
                store.as_ref().get_artifact(name, version, hash)
            } else {
                Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Store index {} does not exist", index.0)))
            }
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Plugin version does not exist in any store"))
        }
    }

    fn get_image(&self, name: &str) -> Result<bytes::Bytes, std::io::Error> {
        log::debug!("Acquiring store_image_map read lock");
        let lock = self.store_image_map.read().expect("Failed to acquire store_image_map read lock");
        if let Some(indices) = lock.get(&StoreName(name.to_owned())) {
            for index in indices {
                if let Some(store) = self.stores.get(index.0) {
                    match store.as_ref().get_image(name) {
                        Ok(img) => return Ok(img),
                        Err(e) => log::error!("Error retrieving image from store #{}: {}", index.0, e),
                    }
                }
            }

            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Stores do not exist for that plugin"))
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Plugin does not exist in any store"))
        }
    }

    fn get_statistics(&self) -> std::collections::HashMap<String, u64> {
        let mut stats = HashMap::new();
        for store in &self.stores {
            let new_stats = store.as_ref().get_statistics();
            Self::merge_statistics_into(&mut stats, new_stats);
        }

        stats
    }
}

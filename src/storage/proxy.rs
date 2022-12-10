use decky_api::{StorePluginList, StorePluginVersion};

use super::IStorage;

pub struct ProxiedStorage<S: IStorage> {
    store_url: String,
    fallback: S,
    agent: ureq::Agent,
}

impl<S: IStorage> ProxiedStorage<S> {
    pub fn new(target_store: String, inner: S) -> Self {
        Self {
            store_url: target_store,
            fallback: inner,
            agent: ureq::Agent::new(),
        }
    }

    fn plugins_url(&self) -> String {
        format!("{}/plugins", self.store_url)
    }

    fn default_artifact_url(ver: &StorePluginVersion) -> String {
        format!("https://cdn.tzatzikiweeb.moe/file/steam-deck-homebrew/versions/{}.zip", ver.hash)
    }

    fn proxy_plugins(&self) -> StorePluginList {
        let url = self.plugins_url();
        match self.agent.get(&url).call() {
            Err(e) => {
                log::error!("Plugins proxy error for {}: {}", url, e);
                vec![]
            },
            Ok(resp) => {
                match resp.into_json::<StorePluginList>() {
                    Err(e) => {
                        log::error!("Plugins json error for {}: {}", url, e);
                        vec![]
                    }
                    Ok(x) => x,
                }
            }
        }
    }
}

impl<S: IStorage> IStorage for ProxiedStorage<S> {
    fn plugins(&self) -> StorePluginList {
        let mut proxy = self.proxy_plugins();
        for plugin in &mut proxy {
            for version in &mut plugin.versions {
                if version.artifact.is_none() {
                    version.artifact = Some(Self::default_artifact_url(version));
                }
            }
        }
        let fallback = self.fallback.plugins();
        proxy.extend_from_slice(&fallback);
        proxy
    }

    fn get_artifact(&self, name: &str, version: &str, hash: &str) -> Result<bytes::Bytes, std::io::Error> {
        self.fallback.get_artifact(name, version, hash)
    }

    fn get_image(&self, name: &str) -> Result<bytes::Bytes, std::io::Error> {
        self.fallback.get_image(name)
    }

    fn get_statistics(&self) -> std::collections::HashMap<String, u64> {
        self.fallback.get_statistics()
    }
}

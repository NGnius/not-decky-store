pub trait IStorage {
    fn plugins(&self) -> decky_api::StorePluginList;

    fn get_artifact(&self, _name: &str, _version: &str, _hash: &str) -> Result<bytes::Bytes, std::io::Error> {
        Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Artifact downloading not supported"))
    }

    fn get_image(&self, _name: &str) -> Result<bytes::Bytes, std::io::Error> {
        Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Image downloading not supported"))
    }

    fn get_statistics(&self) -> std::collections::HashMap<String, u64> {
        std::collections::HashMap::with_capacity(0)
    }
}

pub trait IStorageWrap: Sized + IStorage {
    fn wrap(self, conf: crate::cli::CliArgs) -> Box<dyn IStorage>;
}

impl<X: Sized + IStorage + 'static> IStorageWrap for X {
    fn wrap(self, conf: crate::cli::CliArgs) -> Box<dyn IStorage> {
        let proxy = if let Some(store) = conf.proxy_store {
            Some(store)
        } else if conf.proxy {
            Some("https://plugins.deckbrew.xyz".to_owned())
        } else {
            None
        };
        match (proxy, conf.cache_duration) {
            (Some(proxy), Some(cache_dur)) => Box::new(
                super::CachedStorage::new(
                    cache_dur,
                    super::ProxiedStorage::new(proxy, self),
                )
            ),
            (Some(proxy), None) => Box::new(super::ProxiedStorage::new(proxy, self)),
            (None, Some(cache_dur)) => Box::new(super::CachedStorage::new(cache_dur, self)),
            (None, None) => Box::new(self),
        }
    }
}

pub struct EmptyStorage;

impl IStorage for EmptyStorage {
    fn plugins(&self) -> decky_api::StorePluginList {
        Vec::new()
    }
}

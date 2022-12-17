pub trait IStorage: Send + Sync {
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

pub struct EmptyStorage;

impl IStorage for EmptyStorage {
    fn plugins(&self) -> decky_api::StorePluginList {
        Vec::new()
    }
}

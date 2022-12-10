mod cache;
mod filesystem;
mod interface;
mod proxy;

pub use cache::CachedStorage;
pub use filesystem::FileStorage;
pub use interface::{IStorage, EmptyStorage, IStorageWrap};
pub use proxy::ProxiedStorage;

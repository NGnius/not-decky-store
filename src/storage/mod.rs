mod cache;
mod filesystem;
mod interface;
mod merge;
mod proxy;

pub use cache::CachedStorage;
pub use filesystem::FileStorage;
pub use interface::{IStorage, EmptyStorage};
pub use merge::MergedStorage;
pub use proxy::ProxiedStorage;

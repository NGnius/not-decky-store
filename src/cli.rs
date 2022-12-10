use clap::{Parser, Subcommand, Args};

/// An alternative plugin store
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None, propagate_version = true)]
pub struct CliArgs {
    /// Proxy offerings from another store
    #[arg(name = "store", long)]
    pub proxy_store: Option<String>,
    /// Proxy main store offerings
    #[arg(name = "proxy", short, long)]
    pub proxy: bool,
    /// Cache results for a period
    #[arg(name = "cache", long)]
    pub cache_duration: Option<i64>,
    /// Storage adapter
   #[command(subcommand)]
   pub storage: StorageArgs,
}

impl CliArgs {
    pub fn get() -> Self {
        Self::parse()
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum StorageArgs {
    /// Use default storage settings (filesystem)
    Default,
    /// Use the filesystem
    Filesystem(FilesystemArgs),
}

#[derive(Args, Debug, Clone)]
pub struct FilesystemArgs {
    #[arg(name = "folder", default_value_t = {"./store".into()})]
    pub root: String,
    #[arg(name = "domain", default_value_t = {"http://localhost:22252".into()})]
    pub domain_root: String,
    #[arg(name = "stats", long)]
    pub enable_stats: bool,
}

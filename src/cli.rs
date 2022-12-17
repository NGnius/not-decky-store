use clap::{Parser, Subcommand, Args};
//use std::io::Write as _;
use std::fmt::Write as _;

/// An alternative plugin store
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None, propagate_version = true)]
pub struct CliArgs {
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
    /// Use an existing online store
    Proxy(ProxyArgs),
    /// Use no storage system
    Empty,
    /// Combine multiple storages together
    Merge(MergeArgs)
}

impl StorageArgs {
    // A cursed syntax with super simple parsing for describing storage settings
    pub fn from_descriptor(chars: &mut std::str::Chars) -> Result<Self, String> {
        //let mut chars = descriptor.chars();
        if let Some(char0) = chars.next() {
            Ok(match char0 {
                'd' | '_' => Self::Default,
                'f' => Self::Filesystem(FilesystemArgs::from_descriptor(chars)?),
                'p' => Self::Proxy(ProxyArgs::from_descriptor(chars)?),
                'e' | ' ' => Self::Empty,
                'm' | '+' => Self::Merge(MergeArgs::from_descriptor(chars)?),
                c => return Err(format!("Unexpected char {}, expected a descriptor prefix from {{d f p e m}}", c)),
            })
        } else {
            Err(format!("Empty storage descriptor"))
        }
    }

    pub fn to_descriptor(self) -> String {
        match self {
            Self::Default => "d".to_owned(),
            Self::Filesystem(fs) => format!("f{}", fs.to_descriptor()),
            Self::Proxy(px) => format!("p{}", px.to_descriptor()),
            Self::Empty => "e".to_owned(),
            Self::Merge(ls) => format!("m{}", ls.to_descriptor()),
        }
    }
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

impl FilesystemArgs {
    fn from_descriptor(chars: &mut std::str::Chars) -> Result<Self, String> {
        if let Some(char1) = chars.next() {
            if char1 != '{' {
                return Err(format!("Expected {{, got {}", char1));
            }
        } else {
            return Err(format!("Filesystem descriptor too short"));
        }
        let mut root = None;
        let mut domain = None;
        let mut stats = false;
        let mut buffer = Vec::<char>::new();
        let mut for_variable: Option<String> = None;
        let mut in_string = false;
        for c in chars {
            match c {
                '}' => return
                Ok(Self {
                    root: root.unwrap_or_else(|| "./store".into()),
                    domain_root: domain.unwrap_or_else(|| "http://localhost:22252".into()),
                    enable_stats: stats,
                }),
                '\'' => in_string = !in_string,
                '=' => if !in_string {
                    let value: String = buffer.drain(..).collect();
                    if for_variable.is_some() {
                        return Err("Unexpected = in filesystem descriptor".to_owned());
                    } else {
                        for_variable = Some(value);
                    }
                },
                ',' => if !in_string {
                    let value: String = buffer.iter().collect();
                    if let Some(var) = for_variable.take() {
                        let var_trimmed = var.trim();
                        match &var_trimmed as &str {
                            "r" | "root" => root = Some(value),
                            "d" | "domain" => domain = Some(value),
                            "s" | "stats" => stats = value == "1" || value == "y",
                            v => return Err(format!("Unexpected variable name {} in filesystem descriptor", v)),
                        }
                    } else {
                        return Err("Unexpected , in filesystem descriptor".to_owned())
                    }
                }
                c => buffer.push(c),
            }
        }
        Err("Unexpected end of descriptor".to_owned())
    }

    fn to_descriptor(self) -> String {
        format!("{{root='{}',domain='{}',stats:{}}}", self.root, self.domain_root, self.enable_stats as u8)
    }
}

#[derive(Args, Debug, Clone)]
pub struct ProxyArgs {
    /// Proxy offerings from another store
    #[arg(name = "store", long, default_value_t = {"https://plugins.deckbrew.xyz".into()})]
    pub proxy_store: String,
}

impl ProxyArgs {
    fn from_descriptor(chars: &mut std::str::Chars) -> Result<Self, String> {
        if let Some(char1) = chars.next() {
            if char1 != '{' {
                return Err(format!("Expected {{, got {}", char1));
            }
        } else {
            return Err(format!("Proxy descriptor too short"));
        }
        let mut buffer = Vec::new();
        for c in chars {
            match c {
                '}' => return
                    Ok(Self {
                        proxy_store: if buffer.is_empty() { "https://plugins.deckbrew.xyz".into() } else { buffer.iter().collect() }
                    }),
                c => buffer.push(c),
            }
        }
        Err("Unexpected end of descriptor".to_owned())
    }

    fn to_descriptor(self) -> String {
        format!("{{{}}}", self.proxy_store)
    }
}

#[derive(Args, Debug, Clone)]
pub struct MergeArgs {
    /// Settings descriptor
    pub settings: Vec<String>,
}

impl MergeArgs {
    fn from_descriptor(chars: &mut std::str::Chars) -> Result<Self, String> {
        if let Some(char1) = chars.next() {
            if char1 != '[' {
                return Err(format!("Expected [, got {}", char1));
            }
        } else {
            return Err(format!("Merge descriptor too short"));
        }
        let mut others = Vec::new();
        loop {
            if let Some(char_n) = chars.next() {
                if char_n != '(' {
                    return Err(format!("Expected (, got {}", char_n));
                }
            }
            others.push(StorageArgs::from_descriptor(chars)?.to_descriptor());
            if let Some(char_n) = chars.next() {
                if char_n != ')' {
                    return Err(format!("Expected ), got {}", char_n));
                }
            }
            if let Some(c) = chars.next() {
                match c {
                    ',' => {},
                    ']' => {
                        if others.len() < 2 {
                            return Err("Merge args too short (0-1 descriptors is useless!)".to_owned());
                        }
                        return Ok(Self {
                            settings: others,
                        });
                    },
                    c => return Err(format!("Unexpected char {}, expected ] or ,", c))
                }
            } else {
                break;
            }

        }
        Err("Unexpected end of descriptor".to_owned())
    }

    fn to_descriptor(self) -> String {
        let mut out = "[".to_owned();
        for descriptor in self.settings {
            write!(&mut out, "({})", descriptor).unwrap();
        }
        write!(&mut out, "]").unwrap();
        out
    }

    pub fn generate_args(&self) -> Result<Vec<StorageArgs>, String> {
        let mut results = Vec::with_capacity(self.settings.len());
        for args in &self.settings {
            results.push(StorageArgs::from_descriptor(&mut args.chars())?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_descriptor() {
        let descriptor = "f{root='',domain='',stats:0}";
        let parsed = StorageArgs::from_descriptor(&mut descriptor.chars());
        parsed.expect("StorageArgs parse error");
        let descriptor = "p{}";
        let parsed = StorageArgs::from_descriptor(&mut descriptor.chars());
        parsed.expect("StorageArgs parse error");
        let descriptor = "m[(p{}),(d)]";
        let parsed = StorageArgs::from_descriptor(&mut descriptor.chars());
        parsed.expect("StorageArgs parse error");
        let descriptor = "d";
        let parsed = StorageArgs::from_descriptor(&mut descriptor.chars());
        parsed.expect("StorageArgs parse error");
    }

    #[test]
    fn filesys_descriptor() {
        let descriptor = "{root='',domain='',stats:0}";
        let parsed = FilesystemArgs::from_descriptor(&mut descriptor.chars());
        parsed.expect("FilesystemArgs parse error");
    }

    #[test]
    fn proxy_descriptor() {
        let descriptor = "{}";
        let parsed = ProxyArgs::from_descriptor(&mut descriptor.chars());
        parsed.expect("ProxyArgs parse error");
    }

    #[test]
    fn merge_descriptor() {
        let descriptor = "[(f{}),(p{}),( )]";
        let parsed = MergeArgs::from_descriptor(&mut descriptor.chars());
        parsed.expect("MergeArgs parse error");
    }
}

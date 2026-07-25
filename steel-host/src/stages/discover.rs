use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::config::Config;

pub struct DiscoveredPlugin {
    pub root: PathBuf,
    pub config: Config,
}

pub fn discover_plugins(path: impl AsRef<Path>) -> Vec<DiscoveredPlugin> {
    WalkDir::new(path)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_dir())
        .filter_map(|e| {
            let root = e.into_path();
            let Ok(config) = Config::from_root(&root) else {
                return None;
            };
            Some(DiscoveredPlugin { root, config })
        })
        .collect()
}

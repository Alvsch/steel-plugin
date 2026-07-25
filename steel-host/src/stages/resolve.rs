use std::{
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
};

use semver::{Version, VersionReq};
use thiserror::Error;
use walkdir::WalkDir;

use crate::{HOST_API_VERSION, config::Config, stages::discover::DiscoveredPlugin};

pub struct ResolvedPlugin {
    pub root: PathBuf,
    pub config: Config,
    pub file_table: HashMap<String, PathBuf>,
}

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("duplicate plugin name: '{0}'")]
    DuplicateName(String),

    #[error("plugin '{plugin}' depends on missing plugin '{dependency}'")]
    MissingDependency { plugin: String, dependency: String },

    #[error("plugin '{plugin}' requires '{dependency}' {required}, found {found}")]
    DependencyVersionMismatch {
        plugin: String,
        dependency: String,
        required: VersionReq,
        found: Version,
    },

    #[error("plugin '{plugin}' requires host API version '{required}', which is not satisfied")]
    VersionMismatch { plugin: String, required: String },

    #[error("cyclic dependency detected among plugins: {0:?}")]
    CyclicDependency(Vec<String>),
}

pub fn resolve_plugins(
    plugins: Vec<DiscoveredPlugin>,
) -> Result<Vec<ResolvedPlugin>, ResolveError> {
    let mut by_name: HashMap<String, DiscoveredPlugin> = HashMap::new();
    for p in plugins {
        let name = p.config.name.clone();
        if by_name.insert(name.clone(), p).is_some() {
            return Err(ResolveError::DuplicateName(name));
        }
    }

    for p in by_name.values() {
        if !p.config.api_version.matches(&HOST_API_VERSION) {
            return Err(ResolveError::VersionMismatch {
                plugin: p.config.name.clone(),
                required: p.config.api_version.to_string(),
            });
        }
    }

    // dep existence + version check
    for p in by_name.values() {
        for (dep_name, req) in &p.config.dependencies {
            let Some(dep_plugin) = by_name.get(dep_name) else {
                return Err(ResolveError::MissingDependency {
                    plugin: p.config.name.clone(),
                    dependency: dep_name.clone(),
                });
            };
            if !req.matches(&dep_plugin.config.version) {
                return Err(ResolveError::DependencyVersionMismatch {
                    plugin: p.config.name.clone(),
                    dependency: dep_name.clone(),
                    required: req.clone(),
                    found: dep_plugin.config.version.clone(),
                });
            }
        }
    }

    // topo sort, Kahn
    let mut in_degree: HashMap<String, usize> = by_name.keys().map(|k| (k.clone(), 0)).collect();
    let mut dependents: HashMap<String, Vec<String>> = HashMap::new();

    for p in by_name.values() {
        for dep_name in p.config.dependencies.keys() {
            *in_degree.get_mut(&p.config.name).unwrap() += 1;
            dependents
                .entry(dep_name.clone())
                .or_default()
                .push(p.config.name.clone());
        }
    }

    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|(_, deg)| **deg == 0)
        .map(|(name, _)| name.clone())
        .collect();

    let mut order = Vec::with_capacity(by_name.len());

    while let Some(name) = queue.pop_front() {
        order.push(name.clone());
        if let Some(deps) = dependents.get(&name) {
            for dependent in deps {
                let deg = in_degree.get_mut(dependent).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(dependent.clone());
                }
            }
        }
    }

    if order.len() != by_name.len() {
        let stuck = in_degree
            .into_iter()
            .filter(|(_, d)| *d > 0)
            .map(|(n, _)| n)
            .collect();
        return Err(ResolveError::CyclicDependency(stuck));
    }

    let mut resolved = Vec::with_capacity(order.len());
    for name in order {
        let p = by_name.remove(&name).unwrap();
        let file_table = build_file_table(&p.root);
        resolved.push(ResolvedPlugin {
            root: p.root,
            config: p.config,
            file_table,
        });
    }

    Ok(resolved)
}

fn build_file_table(root: impl AsRef<Path>) -> HashMap<String, PathBuf> {
    WalkDir::new(root.as_ref())
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "luau" || ext == "lua")
        })
        .filter_map(|e| {
            let rel = e.path().strip_prefix(root.as_ref()).ok()?;
            let module_path = rel.with_extension("").to_string_lossy().replace('\\', "/");
            Some((module_path, e.into_path()))
        })
        .collect()
}

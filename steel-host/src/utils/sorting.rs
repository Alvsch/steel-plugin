use std::collections::{HashMap, HashSet, VecDeque};

/// Return plugins in dependency order and names that cannot be ordered.
pub fn sort_plugins(plugins: HashMap<String, Vec<String>>) -> (Vec<String>, Vec<String>) {
    let mut in_degree: HashMap<String, usize> =
        plugins.keys().map(|name| (name.clone(), 0)).collect();
    let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
    let mut invalid_plugins = HashSet::new();

    for (name, dependencies) in &plugins {
        for dependency in dependencies {
            if in_degree.contains_key(dependency) {
                *in_degree
                    .get_mut(name)
                    .expect("plugin name is in the graph") += 1;
                dependents
                    .entry(dependency.clone())
                    .or_default()
                    .push(name.clone());
            } else {
                invalid_plugins.insert(name.clone());
            }
        }
    }

    let mut ready: VecDeque<String> = in_degree
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(name, _)| name.clone())
        .collect();

    let mut sorted = Vec::with_capacity(plugins.len());
    while let Some(name) = ready.pop_front() {
        sorted.push(name.clone());
        if let Some(plugin_dependents) = dependents.get(&name) {
            for dependent in plugin_dependents {
                let count = in_degree
                    .get_mut(dependent)
                    .expect("dependent is in the graph");
                *count -= 1;
                if *count == 0 {
                    ready.push_back(dependent.clone());
                }
            }
        }
    }

    invalid_plugins.extend(
        in_degree
            .into_iter()
            .filter_map(|(name, count)| (count > 0).then_some(name)),
    );
    sorted.retain(|name| !invalid_plugins.contains(name));

    (sorted, invalid_plugins.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use semver::Version;
    use steel_plugin_core::STEEL_API_VERSION;

    use super::sort_plugins;
    use steel_plugin_core::PluginMeta;

    fn make_plugin(name: &str, depends: &[&str]) -> PluginMeta {
        PluginMeta {
            name: name.to_string(),
            description: String::new(),
            version: Version::new(0, 1, 0),
            authors: Vec::new(),
            depends: depends.iter().map(|dep| (*dep).to_string()).collect(),
            api_version: STEEL_API_VERSION.clone(),
        }
    }

    #[test]
    fn sorts_dependency_graph_topologically() {
        let plugins = vec![
            make_plugin("consumer", &["provider"]),
            make_plugin("provider", &[]),
        ];

        let plugins = plugins
            .into_iter()
            .map(|meta| (meta.name, meta.depends))
            .collect();

        let (sorted, invalid) = sort_plugins(plugins);
        let names: Vec<&str> = sorted.iter().map(String::as_str).collect();

        assert_eq!(names, vec!["provider", "consumer"]);
        assert_eq!(invalid, [] as [std::string::String; 0]);
    }

    #[test]
    fn unresolved_dependencies_are_marked_invalid() {
        let plugins = vec![
            make_plugin("provider", &[]),
            make_plugin("consumer", &["missing"]),
        ];

        let plugins = plugins
            .into_iter()
            .map(|meta| (meta.name, meta.depends))
            .collect();

        let (sorted, invalid) = sort_plugins(plugins);
        let sorted_names: Vec<&str> = sorted.iter().map(String::as_str).collect();
        let invalid_names: Vec<&str> = invalid.iter().map(String::as_str).collect();

        assert_eq!(sorted_names, vec!["provider"]);
        assert_eq!(invalid_names, vec!["consumer"]);
    }

    #[test]
    fn cyclic_dependencies_are_marked_invalid() {
        let plugins = vec![
            make_plugin("a", &["b"]),
            make_plugin("b", &["a"]),
            make_plugin("consumer", &["a"]),
        ];

        let plugins = plugins
            .into_iter()
            .map(|meta| (meta.name, meta.depends))
            .collect();

        let (sorted, invalid) = sort_plugins(plugins);
        let mut invalid_names: Vec<&str> = invalid.iter().map(String::as_str).collect();
        invalid_names.sort_unstable();

        assert_eq!(sorted, [] as [std::string::String; 0]);
        assert_eq!(invalid_names, vec!["a", "b", "consumer"]);
    }
}

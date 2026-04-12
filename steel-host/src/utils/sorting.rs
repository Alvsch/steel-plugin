use crate::PluginMeta;
use std::collections::{HashMap, HashSet, VecDeque};

pub fn sort_plugins(plugins: Vec<PluginMeta>) -> (Vec<PluginMeta>, Vec<PluginMeta>) {
    let mut in_degree: Vec<usize> = vec![0; plugins.len()];
    let mut adj: Vec<Vec<usize>> = vec![vec![]; plugins.len()];
    let mut unresolved_indices: HashSet<usize> = HashSet::new();

    {
        let name_to_idx: HashMap<&str, usize> = plugins
            .iter()
            .enumerate()
            .map(|(i, p)| (&*p.name, i))
            .collect();

        for (i, plugin) in plugins.iter().enumerate() {
            for dep in &plugin.depends {
                if let Some(&dep_idx) = name_to_idx.get(&**dep) {
                    in_degree[i] += 1;
                    adj[dep_idx].push(i);
                } else {
                    unresolved_indices.insert(i);
                }
            }
        }
    }

    let mut queue: VecDeque<usize> = (0..plugins.len()).filter(|&i| in_degree[i] == 0).collect();

    let mut sorted_indices = Vec::new();
    while let Some(idx) = queue.pop_front() {
        sorted_indices.push(idx);
        for &dep in &adj[idx] {
            in_degree[dep] -= 1;
            if in_degree[dep] == 0 {
                queue.push_back(dep);
            }
        }
    }

    let mut cyclic_indices: HashSet<usize> =
        (0..plugins.len()).filter(|&i| in_degree[i] > 0).collect();
    // TODO: maybe change
    cyclic_indices.extend(unresolved_indices);

    let mut slots: Vec<Option<PluginMeta>> = plugins.into_iter().map(Some).collect();

    let invalid: Vec<PluginMeta> = cyclic_indices
        .iter()
        .filter_map(|&i| slots[i].take())
        .collect();

    let sorted: Vec<PluginMeta> = sorted_indices
        .iter()
        .filter_map(|&i| slots[i].take())
        .collect();

    (sorted, invalid)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use semver::Version;
    use steel_plugin_sdk::STEEL_API_VERSION;

    use super::sort_plugins;
    use crate::PluginMeta;

    fn make_plugin(name: &str, depends: &[&str]) -> PluginMeta {
        PluginMeta {
            name: name.to_string(),
            description: String::new(),
            version: Version::new(0, 1, 0),
            authors: Vec::new(),
            depends: depends.iter().map(|dep| (*dep).to_string()).collect(),
            api_version: STEEL_API_VERSION.clone(),
            file_path: PathBuf::new(),
        }
    }

    #[test]
    fn sorts_dependency_graph_topologically() {
        let plugins = vec![
            make_plugin("consumer", &["provider"]),
            make_plugin("provider", &[]),
        ];

        let (sorted, invalid) = sort_plugins(plugins);
        let names: Vec<&str> = sorted.iter().map(|plugin| plugin.name.as_str()).collect();

        assert_eq!(names, vec!["provider", "consumer"]);
        assert!(invalid.is_empty());
    }

    #[test]
    fn unresolved_dependencies_are_marked_invalid() {
        let plugins = vec![
            make_plugin("provider", &[]),
            make_plugin("consumer", &["missing"]),
        ];

        let (sorted, invalid) = sort_plugins(plugins);
        let sorted_names: Vec<&str> = sorted.iter().map(|plugin| plugin.name.as_str()).collect();
        let invalid_names: Vec<&str> = invalid.iter().map(|plugin| plugin.name.as_str()).collect();

        assert_eq!(sorted_names, vec!["provider"]);
        assert_eq!(invalid_names, vec!["consumer"]);
    }

    #[test]
    fn cyclic_dependencies_are_marked_invalid() {
        let plugins = vec![make_plugin("a", &["b"]), make_plugin("b", &["a"])];

        let (sorted, invalid) = sort_plugins(plugins);
        let mut invalid_names: Vec<&str> =
            invalid.iter().map(|plugin| plugin.name.as_str()).collect();
        invalid_names.sort_unstable();

        assert!(sorted.is_empty());
        assert_eq!(invalid_names, vec!["a", "b"]);
    }
}

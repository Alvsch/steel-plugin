use std::collections::{HashMap, HashSet, VecDeque};

use crate::PluginContainer;

pub fn sort_plugins(plugins: Vec<PluginContainer>) -> (Vec<PluginContainer>, Vec<PluginContainer>) {
    let mut in_degree: Vec<usize> = vec![0; plugins.len()];
    let mut adj: Vec<Vec<usize>> = vec![vec![]; plugins.len()];
    let mut unresolved_indices: HashSet<usize> = HashSet::new();

    {
        let name_to_idx: HashMap<&str, usize> = plugins
            .iter()
            .enumerate()
            .map(|(i, p)| (p.borrow_dependent().name, i))
            .collect();

        for (i, plugin) in plugins.iter().enumerate() {
            for &dep in &plugin.borrow_dependent().depends {
                if let Some(&dep_idx) = name_to_idx.get(dep) {
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

    let mut cyclic_indices: HashSet<usize> = (0..plugins.len()).filter(|&i| in_degree[i] > 0).collect();
    // TODO: maybe change
    cyclic_indices.extend(unresolved_indices);

    let mut slots: Vec<Option<PluginContainer>> = plugins.into_iter().map(Some).collect();

    let invalid: Vec<PluginContainer> = cyclic_indices
        .iter()
        .filter_map(|&i| slots[i].take())
        .collect();

    let sorted: Vec<PluginContainer> = sorted_indices
        .iter()
        .filter_map(|&i| slots[i].take())
        .collect();

    (sorted, invalid)
}

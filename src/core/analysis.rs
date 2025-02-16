use std::{collections::HashSet, path::Path};
use std::io::Error;

use crate::config::types::Config;
use crate::core::{node::Node, dependencies::NodeDependencies};

pub trait GraphDependencies {
    fn collect_nodes(root_path: &Path, config: &Config) -> Result<Vec<Node>, Error>;
    fn find_dependencies(projects: &[Node], config: &Config) -> Result<NodeDependencies, Error>;
}

pub fn detect_cycles(nodes: &[Node], node_dependencies: &NodeDependencies) -> bool {
    let mut has_cycle = false;
    for i in 0..nodes.len() {
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        if dfs(i, &mut stack, &mut visiting, &mut visited, node_dependencies, nodes) {
            println!("===============================");
            has_cycle = true;
        }
    }
    if !has_cycle {
        println!("No circular dependencies detected.");
    }
    has_cycle
}

fn dfs(
    node: usize,
    stack: &mut Vec<usize>,
    visiting: &mut HashSet<usize>,
    visited: &mut HashSet<usize>,
    deps: &NodeDependencies,
    nodes: &[Node],
) -> bool {
    if visiting.contains(&node) {
        let cycle_start_index = stack.iter().position(|&x| x == node).unwrap();
        println!("Cycle detected in dependencies starting at '{}':", nodes[node].name);
        for &index in &stack[cycle_start_index..] {
            print!("{} -> ", nodes[index].name);
        }
        println!("{}", nodes[node].name);
        return true;
    }

    if visited.contains(&node) {
        return false;
    }

    visiting.insert(node);
    stack.push(node);

    for next in &deps[node] {
        if dfs(next.to, stack, visiting, visited, deps, nodes) {
            return true;
        }
    }

    stack.pop();
    visiting.remove(&node);
    visited.insert(node);
    false
}

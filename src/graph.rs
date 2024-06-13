use std::{collections::HashSet, path::Path};
use std::io::Error;

use crate::configuration::Config;

#[derive(Debug)]
pub struct Node {
    // relative_path: String,
    pub id: String, // Unique identifier per node
    pub name: String,
    pub layer: String, // core, io, usecase
    pub node_type: String, // project, namespace, class, folder
    pub color: String,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct EdgeInfo{
    pub to: usize,
    pub allowed: bool,
    pub label: String,
}
pub type EdgesInfo = Vec<EdgeInfo>;
pub type NodeDependencies = Vec<EdgesInfo>;

pub trait GraphDependencies {
    fn collect_nodes(root_path: &Path, config: &Config) -> Result<Vec<Node>, Error>;
    fn find_dependencies(projects: &[Node], config: &Config) -> Result<NodeDependencies, Error>;
    // fn detect_cycles(nodes: &[Node], node_dependencies: &NodeDependencies);
}

pub fn detect_cycles(nodes: &[Node], node_dependencies: &NodeDependencies) {
    let mut has_cycle = false;
    for i in 0..nodes.len() {
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        if dfs(i, &mut stack, &mut visiting, &mut visited, node_dependencies, nodes) {
            println!("Cycle initiated from node: {}", nodes[i].name);
            has_cycle = true;
        }
    }
    if !has_cycle {
        println!("No circular dependencies detected.");
    }
}

/// Helper function to perform Depth-First Search (DFS) to detect cycles
fn dfs(
    node: usize,
    stack: &mut Vec<usize>,
    visiting: &mut HashSet<usize>,
    visited: &mut HashSet<usize>,
    deps: &NodeDependencies,
    nodes: &[Node],
) -> bool {
    if visiting.contains(&node) {
        // Cycle detected, print the cycle
        let cycle_start_index = stack.iter().position(|&x| x == node).unwrap();
        println!("Cycle detected in dependencies starting at '{}':", nodes[node].name);
        for &index in &stack[cycle_start_index..] {
            print!("{} -> ", nodes[index].name);
        }
        println!("{}", nodes[node].name); // Complete the cycle
        return true;
    }

    if visited.contains(&node) {
        return false; // This node has been fully explored
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
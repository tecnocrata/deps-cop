use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Read};
use std::path::Path;
use walkdir::WalkDir;

use crate::configuration::{determine_layer, Config};
use crate::graph::{EdgeInfo, Node, NodeDependencies};

pub struct NamespaceDependencyManager;

impl NamespaceDependencyManager {

    pub fn collect_nodes(root_path: &Path, config: &Config) -> Result<Vec<Node>, Error> {
        let mut namespaces: HashMap<String, Node> = HashMap::new();
        let mut namespace_files = Vec::new();
    
        for entry in WalkDir::new(root_path) {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "cs") {
                namespace_files.push(path.to_path_buf());
            }
        }
    
        for file in namespace_files {
            //get the filename
            let filename = file.file_name().unwrap().to_str().unwrap();
            println!("Collecting nodes from file: {}", filename);
            let mut file = File::open(&file)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
    
            for line in contents.lines() {
                // let mut current_namespace = String::new();
                if line.starts_with("namespace") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(ns) = parts.get(1) {
                        let current_namespace = ns.trim_end_matches(';').to_string();
    
                        if !namespaces.contains_key(&current_namespace) {
                            let layer = determine_layer(&current_namespace, &config.csharp.namespaces);
                            let color = config.get_color(&layer).unwrap_or(&"gray".to_string()).to_string();
                            namespaces.insert(current_namespace.clone(), Node {
                                id: current_namespace.clone(),
                                name: current_namespace.clone(),
                                node_type: "namespace".to_string(),
                                layer,
                                color,
                            });
                        }
                    }
                } else if line.trim_start().starts_with("using") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(ns) = parts.get(1) {
                        let namespace = ns.trim_end_matches(';').to_string();
    
                        if !namespaces.contains_key(&namespace) {
                            let layer = determine_layer(&namespace, &config.csharp.namespaces);
                            let color = config.get_color(&layer).unwrap_or(&"gray".to_string()).to_string();
                            namespaces.insert(namespace.clone(), Node {
                                id: namespace.clone(),
                                name: namespace.clone(),
                                node_type: "namespace".to_string(),
                                layer,
                                color,
                            });
                        }
                    }
                }
            }
        }
        let global_namespace = "global::namespace";
        namespaces.insert(global_namespace.to_string(), Node {
            id: global_namespace.to_string(),
            name: global_namespace.to_string(),
            node_type: "namespace".to_string(),
            layer: "unknown".to_string(),
            color: "gray".to_string(),
        });
    
        let nodes: Vec<Node> = namespaces.into_values().collect();
    
        Ok(nodes)
    }

    // Resolves dependencies of each project
    pub fn find_dependencies(root_path: &Path, nodes: &[Node], config: &Config) -> Result<NodeDependencies, Error> {
        let mut node_dependencies: NodeDependencies = vec![Vec::new(); nodes.len()];
        let mut namespace_files = Vec::new();
    
        for entry in WalkDir::new(root_path) {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "cs") {
                namespace_files.push(path.to_path_buf());
            }
        }
    
        // Create a map of project id (absolute path) to index in the projects vector (for quick lookup)
        let node_index_map: HashMap<String, usize> = nodes.iter().enumerate()
            .map(|(index, project)| (project.id.clone(), index))
            .collect();
    
        for file in namespace_files {
            let filename = file.file_name().unwrap().to_str().unwrap();
            println!("Collecting dependencies from file: {}", filename);
            let mut file = File::open(&file)?;
            // let reader = BufReader::new(file);
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
    
            // let mut parent_namespace = String::new();
            let mut edges_info = Vec::new();
            let mut from_node_index: Option<usize> = None;
    
            for line in contents.lines() {
                // let line = line?;
                if line.trim_start().starts_with("namespace") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(ns) = parts.get(1) {
                        let parent_namespace = ns.trim_end_matches(';').to_string();
                        from_node_index = node_index_map.get(&parent_namespace).map(|&i| i);
                    }
                } else if line.trim_start().starts_with("using") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(child_ns) = parts.get(1) {
                        let child_namespace = child_ns.trim_end_matches(';').to_string();
    
                        if let Some(&index) = node_index_map.get(&child_namespace) {
                            // let parent_layer = &parent_namespace;
                            let to_layer = &nodes[index].layer;
                            // let allowed_layers = config.global.allowed.get_layers(parent_layer).cloned().unwrap_or_else(Vec::new);
                            let allowed_layers = config.global.allowed.get_layers(to_layer).cloned().unwrap_or_else(Vec::new);
                            let ok = allowed_layers.contains(to_layer);
                            let label = format!("to -> {}", nodes[index].name);
                            edges_info.push(EdgeInfo { to: index, allowed: ok, label });
                        }
                    }
                }
            }
            let index = if let Some(index) = from_node_index {
                //we need to update all the edges_info with the correct ok value because we need to retrieve the parent's layer and check if it is allowed to access the child's layer
                for edge in &mut node_dependencies[index] {
                    let parent_layer = &nodes[index].layer;
                    let to_layer = &nodes[edge.to].layer;
                    let allowed_layers = config.global.allowed.get_layers(parent_layer).cloned().unwrap_or_else(Vec::new);
                    edge.allowed = allowed_layers.contains(to_layer);
                }
                index
            } else {
                nodes.len() - 1
            };

            node_dependencies[index].extend(edges_info);
        }
    
        Ok(node_dependencies)
    }    
    
}
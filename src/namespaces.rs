use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Read};
use std::path::Path;
use walkdir::WalkDir;
use regex::Regex;

use crate::configuration::{determine_layer, should_exclude, Config};
use crate::graph::{EdgeInfo, Node, NodeDependencies};
use crate::stringsutils::RemoveBom;

pub struct NamespaceDependencyManager;

impl NamespaceDependencyManager {
    pub fn collect_nodes(root_path: &Path, config: &Config) -> Result<Vec<Node>, Error> {
        let mut namespaces: HashMap<String, Node> = HashMap::new();
        let mut namespace_files = Vec::new();

        for entry in WalkDir::new(root_path) {
            let entry = entry?;
            let path = entry.path();
            if should_exclude(&path.to_path_buf(), &config.csharp.as_ref().unwrap().exclude) {
                continue;
            }
            if path.extension().map_or(false, |e| e == "cs") {
                namespace_files.push(path.to_path_buf());
            }
        }

        let namespace_regex = Regex::new(r"^namespace\s+([\p{L}\p{N}_\.]+);?$").unwrap();
        let using_regex = Regex::new(r"^using\s+([\p{L}\p{N}_\.]+);?$").unwrap();

        for file in namespace_files {
            let mut file = File::open(&file)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            contents = RemoveBom::remove_bom(&contents);

            for line in contents.lines() {
                if let Some(captures) = namespace_regex.captures(line) {
                    let current_namespace = captures.get(1).map(|m| m.as_str().to_string());
                    if let Some(namespace) = current_namespace {
                        if !namespaces.contains_key(&namespace) {
                            let csharp_config = config.csharp.as_ref().unwrap();
                            let layer = determine_layer(&namespace, &csharp_config.namespaces, csharp_config.case_sensitive, &csharp_config.pattern);
                            let color = config.get_color(&layer).cloned().unwrap_or_else(|| "gray".to_string());
                            namespaces.insert(namespace.clone(), Node {
                                id: namespace.clone(),
                                name: namespace.clone(),
                                node_type: "namespace".to_string(),
                                layer,
                                color,
                            });
                        }
                    }
                } else if let Some(captures) = using_regex.captures(line) {
                    let namespace = captures.get(1).map(|m| m.as_str().to_string());
                    if let Some(namespace) = namespace {
                        if !namespaces.contains_key(&namespace) {
                            let csharp_config = config.csharp.as_ref().unwrap();
                            let layer = determine_layer(&namespace, &csharp_config.namespaces, csharp_config.case_sensitive, &csharp_config.pattern);
                            let color = config.get_color(&layer).cloned().unwrap_or_else(|| "gray".to_string());
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

    pub fn find_dependencies(root_path: &Path, nodes: &[Node], config: &Config) -> Result<NodeDependencies, Error> {
        let mut node_dependencies: NodeDependencies = vec![Vec::new(); nodes.len()];
        let mut namespace_files = Vec::new();

        for entry in WalkDir::new(root_path) {
            let entry = entry?;
            let path = entry.path();
            if should_exclude(&path.to_path_buf(), &config.csharp.as_ref().unwrap().exclude) {
                continue;
            }
            if path.extension().map_or(false, |e| e == "cs") {
                namespace_files.push(path.to_path_buf());
            }
        }

        let node_index_map: HashMap<String, usize> = nodes.iter().enumerate()
            .map(|(index, project)| (project.id.clone(), index))
            .collect();

        let namespace_regex = Regex::new(r"^namespace\s+([\p{L}\p{N}_\.]+);?$").unwrap();
        let using_regex = Regex::new(r"^using\s+([\p{L}\p{N}_\.]+);?$").unwrap();

        for file in namespace_files {
            let mut file = File::open(&file)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            contents = RemoveBom::remove_bom(&contents);

            let mut edges_info = Vec::new();
            let mut from_node_index: Option<usize> = None;

            for line in contents.lines() {
                if let Some(captures) = namespace_regex.captures(line) {
                    let parent_namespace = captures.get(1).map(|m| m.as_str().to_string());
                    from_node_index = parent_namespace.as_ref().and_then(|ns| node_index_map.get(ns).cloned());
                } else if let Some(captures) = using_regex.captures(line) {
                    let child_namespace = captures.get(1).map(|m| m.as_str().to_string());

                    if let Some(child_namespace) = child_namespace {
                        if let Some(&index) = node_index_map.get(&child_namespace) {
                            let to_layer = &nodes[index].layer;
                            let allowed_layers = config.global.rules.get(to_layer).cloned().unwrap_or_default();
                            let ok = allowed_layers.contains(to_layer);
                            let label = format!("to -> {}", nodes[index].name);
                            edges_info.push(EdgeInfo { to: index, allowed: ok, label });
                        }
                    }
                }
            }

            if let Some(parent_index) = from_node_index {
                for edge in &mut edges_info {
                    let parent_layer = &nodes[parent_index].layer;
                    let to_layer = &nodes[edge.to].layer;
                    let allowed_layers = config.global.rules.get(parent_layer).cloned().unwrap_or_default();
                    edge.allowed = allowed_layers.contains(to_layer);
                }
                let current_edges = &node_dependencies[parent_index];
                let new_edges: Vec<EdgeInfo> = edges_info.into_iter()
                    .filter(|edge| !current_edges.iter().any(|e| e.to == edge.to))
                    .collect();
                node_dependencies[parent_index].extend(new_edges);
            }
        }

        Ok(node_dependencies)
    }
}
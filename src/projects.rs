use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Error, Read};
use std::path::Path;
use path_slash::PathExt;
use serde::Deserialize;
use regex::Regex;
use walkdir::WalkDir;

use crate::configuration::{Config, StringOrVec};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct ProjectReference {
    include: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct ItemGroup {
    #[serde(rename = "ProjectReference", default)]
    project_references: Vec<ProjectReference>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Project {
    #[serde(rename = "ToolsVersion", default)]
    tools_version: Option<String>,
    #[serde(rename = "ItemGroup", default)]
    item_groups: Vec<ItemGroup>,
}

#[derive(Debug)]
pub struct Node {
    // relative_path: String,
    pub id: String, // Unique identifier per node
    pub name: String,
    pub layer: String, // core, io, usecase
    pub node_type: String // project, namespace, class, folder
}

#[derive(Debug)]
pub struct EdgeInfo{
    pub to: usize,
    pub allowed: bool,
    pub label: String,
}
pub type EdgesInfo = Vec<EdgeInfo>;
pub type NodeDependencies = Vec<EdgesInfo>;

pub struct ProjectDependencyManager;

pub trait ProjectDependencies {
    fn collect_csharp_projects(root_path: &Path, config: &Config) -> Result<Vec<Node>, Error>;
    fn find_dependencies(projects: &[Node], config: &Config) -> Result<NodeDependencies, Error>;
    fn detect_cycles(nodes: &[Node], node_dependencies: &NodeDependencies);
}

impl ProjectDependencies for ProjectDependencyManager {
    /// Collects CsProject data from .csproj files found under the given root path
    fn collect_csharp_projects(root_path: &Path, config: &Config) -> Result<Vec<Node>, Error> {
        let mut projects = Vec::new();

        for entry in WalkDir::new(root_path) {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "csproj") {
                let mut file = File::open(path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                // Parse XML to check for ToolsVersion
                let _project: Project = match serde_xml_rs::from_str(&contents) {
                    Ok(proj) => proj,
                    Err(err) => {
                        eprintln!("Failed to parse .csproj file, possible incompatible file: {}, error: {}", path.display(), err);
                        continue; // Skip this file if parsing fails
                    }
                };

                let absolute_path = path.to_str().unwrap().to_string();
                let name = path.file_name().unwrap().to_str().unwrap().to_string();

                let layer = determine_layer(&name, &config);
                projects.push(Node {
                    id: absolute_path,
                    name,
                    node_type: "project".to_string(),
                    layer,
                });
            }
        }
    
        Ok(projects)
    }

    /// Resolves dependencies of each project
    fn find_dependencies(projects: &[Node], config: &Config) -> Result<NodeDependencies, Error> {
        let mut project_dependencies = Vec::new();
    
        // It creates a map of project id (absolute path) to index in the projects vector (for quick lookup
        let path_index_map: HashMap<String, usize> = projects.iter().enumerate()
            .map(|(index, project)| (project.id.clone(), index))
            .collect();
    
        for project in projects {
            let project_path = Path::new(&project.id); // The id is the absolute path
            let file = File::open(project_path)?;
            let file_reader = BufReader::new(file);
            let csproj_data: Project = match serde_xml_rs::from_reader(file_reader) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("Failed to parse .csproj file: {}", project.id);
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, err.to_string()));
                }
            };
    
            let mut edges_info = Vec::new();
            let project_dir = project_path.parent().unwrap();
    
            for item_group in &csproj_data.item_groups {
                for project_reference in &item_group.project_references {
                    let normalized_path = if cfg!(target_os = "windows") {
                        Path::new(&project_reference.include).to_slash().unwrap()
                    } else {
                        project_reference.include.replace("\\", "/")
                    };
                    let dep_path = project_dir.join(normalized_path);
                    if let Ok(canonical_dep_path) = dep_path.canonicalize() {
                        let dep_path_str = canonical_dep_path.to_str().unwrap();
                        if let Some(&index) = path_index_map.get(dep_path_str) {
                            // Verify if the dependency is allowed
                            let from_layer = &project.layer;
                            let to_layer = &projects[index].layer;
                            static EMPTY_VEC: &Vec<String> = &Vec::new();
                            let allowed_layers = match &config.global.allowed.get_layers(from_layer) {
                                Some(layers) => layers,
                                None => EMPTY_VEC,
                            };//.unwrap_or(&vec![]);
                            let ok = allowed_layers.contains(to_layer);
                            let label = format!("{} -> {}", project.name, projects[index].name);
                            edges_info.push(EdgeInfo { to: index, allowed: ok, label });
                        }
                    }
                }
            }
            project_dependencies.push(edges_info);
        }
    
        Ok(project_dependencies)
    }

    fn detect_cycles(nodes: &[Node], node_dependencies: &NodeDependencies) {
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
    
}

fn determine_layer(name: &str, config: &Config) -> String {
    // println!("analyzed name: {}", name);
    for (layer, pattern) in &config.csharp.projects {
        let patterns = match pattern {
            StringOrVec::String(p) => vec![p.clone()],
            StringOrVec::Vec(ps) => ps.clone(),
        };
        // println!("patterns {:?} -> ", patterns);
        for pat in patterns {
            if let Ok(re) = Regex::new(&pat) {
                if re.is_match(name) {
                    // print!("{} -> ", name);
                    return layer.clone();
                }
            }
        }
    }
    "unknown".to_string()
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
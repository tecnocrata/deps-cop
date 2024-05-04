use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Error, Read};
use std::path::Path;
use path_slash::PathExt;
use serde::Deserialize;
use walkdir::WalkDir;

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
    id: String, // Unique identifier per node
    pub name: String,
    // type: String
}

pub struct ProjectDependencyManager;

pub trait ProjectDependencies {
    fn collect_projects(root_path: &Path) -> Result<Vec<Node>, Error>;
    fn find_dependencies(projects: &[Node]) -> Result<Vec<Vec<usize>>, Error>;
    fn detect_cycles(projects: &[Node], project_dependencies: &[Vec<usize>]);
}

impl ProjectDependencies for ProjectDependencyManager {
    /// Collects CsProject data from .csproj files found under the given root path
    fn collect_projects(root_path: &Path) -> Result<Vec<Node>, Error> {
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
                
                // if project.tools_version.is_some() {
                //     println!("Skipping '{}' due to incompatible csproj file.", path.display());
                //     continue; // Skip this file if ToolsVersion is present
                // }

                // If ToolsVersion is not present, process the file
                // let relative_path = match path.strip_prefix(root_path) {
                //     Ok(stripped_path) => stripped_path.to_str().unwrap().to_string(),
                //     Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to strip prefix")),
                // };
                let absolute_path = path.to_str().unwrap().to_string();
                let name = path.file_name().unwrap().to_str().unwrap().to_string();

                projects.push(Node { id: absolute_path, name });
            }
        }

        Ok(projects)
    }

    /// Resolves dependencies of each project
    fn find_dependencies(projects: &[Node]) -> Result<Vec<Vec<usize>>, Error> {
        let mut project_dependencies = Vec::new();
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

            let mut deps = Vec::new();
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
                            deps.push(index);
                        }
                    }
                }
            }
            project_dependencies.push(deps);
        }

        Ok(project_dependencies)
    }

    fn detect_cycles(nodes: &[Node], node_dependencies: &[Vec<usize>]) {
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

/// Helper function to perform Depth-First Search (DFS) to detect cycles
fn dfs(
    node: usize,
    stack: &mut Vec<usize>,
    visiting: &mut HashSet<usize>,
    visited: &mut HashSet<usize>,
    deps: &[Vec<usize>],
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

    for &next in &deps[node] {
        if dfs(next, stack, visiting, visited, deps, nodes) {
            return true;
        }
    }

    stack.pop();
    visiting.remove(&node);
    visited.insert(node);
    false
}
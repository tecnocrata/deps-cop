use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Error, Read};
use std::path::Path;
use path_slash::PathExt;
use serde::Deserialize;
// use regex::Regex;
use walkdir::WalkDir;

use crate::configuration::{determine_layer, should_exclude, Config};
use crate::graph::{EdgeInfo, GraphDependencies, Node, NodeDependencies};
use crate::stringsutils::RemoveBom;

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

pub struct ProjectDependencyManager;

impl GraphDependencies for ProjectDependencyManager {
    /// Collects CsProject data from .csproj files found under the given root path
    fn collect_nodes(root_path: &Path, config: &Config) -> Result<Vec<Node>, Error> {
        let mut projects = Vec::new();

        for entry in WalkDir::new(root_path) {
            let entry = entry?;
            let path = entry.path();
            if should_exclude(&path.to_path_buf(), &config.csharp.exclude_folders) {
                continue;
            }
            if path.extension().map_or(false, |e| e == "csproj") {
                let mut file = File::open(path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                // Remove BOM
                contents = contents.remove_bom();

                // Parse XML to check for ToolsVersion
                let _project: Project = match serde_xml_rs::from_str(&contents) {
                    Ok(proj) => proj,
                    Err(err) => {
                        eprintln!("Failed to parse .csproj file, possible incompatible file: {}, error: {}", path.display(), err);
                        continue; // Skip this file if parsing fails
                    }
                };

                let absolute_path = path.to_str().unwrap().to_string();
                let filename = path.file_name().unwrap().to_str().unwrap().to_string();

                let layer = determine_layer(&filename, &config.csharp.projects, config.csharp.case_sensitive, &config.csharp.pattern);
                let color = config.get_color(&layer).unwrap_or(&"gray".to_string()).to_string();
                projects.push(Node {
                    id: absolute_path,
                    name: filename,
                    node_type: "project".to_string(),
                    layer,
                    color
                });
            }
        }
    
        Ok(projects)
    }

    /// Resolves dependencies of each project
    fn find_dependencies(nodes: &[Node], config: &Config) -> Result<NodeDependencies, Error> {
        let mut node_dependencies = Vec::new();
    
        // It creates a map of project id (absolute path) to index in the projects vector (for quick lookup
        let path_index_map: HashMap<String, usize> = nodes.iter().enumerate()
            .map(|(index, project)| (project.id.clone(), index))
            .collect();
    
        for project in nodes {
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
                            let to_layer = &nodes[index].layer;
                            static EMPTY_VEC: &Vec<String> = &Vec::new();
                            let allowed_layers = config.global.rules.get(from_layer).unwrap_or(&EMPTY_VEC);//.unwrap_or(&vec![]);
                            let ok = allowed_layers.contains(to_layer);
                            let label = format!("{} -> {}", project.name, nodes[index].name);
                            edges_info.push(EdgeInfo { to: index, allowed: ok, label });
                        }
                    }
                }
            }
            node_dependencies.push(edges_info);
        }
    
        Ok(node_dependencies)
    }
    
}

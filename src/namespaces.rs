use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Error, Read};
use std::path::Path;
use path_slash::PathExt;
// use regex::Regex;
use walkdir::WalkDir;

use crate::configuration::{determine_layer, Config};
use crate::graph::{EdgeInfo, EdgesInfo,  Node, NodeDependencies};

pub struct NamespaceDependencyManager;

impl NamespaceDependencyManager {
    // Collects C# Namespaces data from .cs files found under the given root path. it will get the namespaces from using and namespace declarations
    // it is needed to consider newer namespace deplarations that does not enclose the class declaration.
    fn build_graph(root_path: &Path, config: &Config) -> Result<(Vec<Node>, NodeDependencies), Error> {
        let mut namespaces: HashMap<String,Node> = HashMap::new();
        let mut namespaces_dependencies: HashMap<String,EdgesInfo> = HashMap::new();
        let mut namespace_files = Vec::new();

        for entry in WalkDir::new(root_path) {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "cs") {
                namespace_files.push(path.to_path_buf());
            }
        }

        for file in namespace_files {
            let mut file = File::open(&file)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            
            // let mut using_namespaces = Vec::new();

            for line in contents.lines() {
                let mut namespace = String::new();
                let mut parent_namespace = false;
                if line.starts_with("namespace") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    namespace = parts[1].to_string();
                    parent_namespace = true;
                } else if line.starts_with("using") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    let namespace = parts[1].replace(";", "");
                }
                if !namespaces.contains_key(namespace.as_str()){
                    let layer = determine_layer(&namespace, &config.csharp.namespaces);
                    let color = config.get_color(&layer).unwrap_or(&"gray".to_string()).to_string();
                    namespaces.insert(namespace.clone(), Node {
                        id: namespace.clone(),
                        name: namespace.clone(),
                        node_type: "namespace".to_string(),
                        layer,
                        color
                    });
                }
                if parent_namespace {
                    if !namespaces_dependencies.contains_key(namespace.as_str()){
                        namespaces_dependencies.insert(namespace.clone(), Vec::new());
                    }
                }
                else{

                }
            }

            // let absolute_path = file.to_str().unwrap().to_string();
            // let filename = file.file_name().unwrap().to_str().unwrap().to_string();

            // let layer = determine_layer(&namespace, &config.csharp.namespaces);
            // let color = config.get_color(&layer).unwrap_or(&"gray".to_string()).to_string();
            // namespaces.push(Node {
            //     id: namespace,
            //     name: namespace,
            //     node_type: "namespace".to_string(),
            //     layer,
            //     color
            // });

            // for using_namespace in using_namespaces {
            //     let layer = determine_layer(&using_namespace, &config.csharp.namespaces);
            //     let color = config.get_color(&layer).unwrap_or(&"gray".to_string()).to_string();
            //     namespaces.push(Node {
            //         id: using_namespace,
            //         name: using_namespace,
            //         node_type: "namespace".to_string(),
            //         layer,
            //         color
            //     });
            // }
        }

        Ok(namespaces)
    }

    

    // fn collect_nodes(root_path: &Path, config: &Config) -> Result<Vec<Node>, Error> {
    //     let mut projects = Vec::new();

    //     for entry in WalkDir::new(root_path) {
    //         let entry = entry?;
    //         let path = entry.path();
    //         if path.extension().map_or(false, |e| e == "csproj") {
    //             let mut file = File::open(path)?;
    //             let mut contents = String::new();
    //             file.read_to_string(&mut contents)?;

    //             // Parse XML to check for ToolsVersion
    //             let _project: Project = match serde_xml_rs::from_str(&contents) {
    //                 Ok(proj) => proj,
    //                 Err(err) => {
    //                     eprintln!("Failed to parse .csproj file, possible incompatible file: {}, error: {}", path.display(), err);
    //                     continue; // Skip this file if parsing fails
    //                 }
    //             };

    //             let absolute_path = path.to_str().unwrap().to_string();
    //             let filename = path.file_name().unwrap().to_str().unwrap().to_string();

    //             let layer = determine_layer(&filename, &config.csharp.projects);
    //             let color = config.get_color(&layer).unwrap_or(&"gray".to_string()).to_string();
    //             projects.push(Node {
    //                 id: absolute_path,
    //                 name: filename,
    //                 node_type: "project".to_string(),
    //                 layer,
    //                 color
    //             });
    //         }
    //     }
    
    //     Ok(projects)
    // }

    //// Resolves dependencies of each project
    // fn find_dependencies(projects: &[Node], config: &Config) -> Result<NodeDependencies, Error> {
    //     let mut project_dependencies = Vec::new();
    
    //     // It creates a map of project id (absolute path) to index in the projects vector (for quick lookup
    //     let path_index_map: HashMap<String, usize> = projects.iter().enumerate()
    //         .map(|(index, project)| (project.id.clone(), index))
    //         .collect();
    
    //     for project in projects {
    //         let project_path = Path::new(&project.id); // The id is the absolute path
    //         let file = File::open(project_path)?;
    //         let file_reader = BufReader::new(file);
    //         let csproj_data: Project = match serde_xml_rs::from_reader(file_reader) {
    //             Ok(data) => data,
    //             Err(err) => {
    //                 eprintln!("Failed to parse .csproj file: {}", project.id);
    //                 return Err(std::io::Error::new(std::io::ErrorKind::Other, err.to_string()));
    //             }
    //         };
    
    //         let mut edges_info = Vec::new();
    //         let project_dir = project_path.parent().unwrap();
    
    //         for item_group in &csproj_data.item_groups {
    //             for project_reference in &item_group.project_references {
    //                 let normalized_path = if cfg!(target_os = "windows") {
    //                     Path::new(&project_reference.include).to_slash().unwrap()
    //                 } else {
    //                     project_reference.include.replace("\\", "/")
    //                 };
    //                 let dep_path = project_dir.join(normalized_path);
    //                 if let Ok(canonical_dep_path) = dep_path.canonicalize() {
    //                     let dep_path_str = canonical_dep_path.to_str().unwrap();
    //                     if let Some(&index) = path_index_map.get(dep_path_str) {
    //                         // Verify if the dependency is allowed
    //                         let from_layer = &project.layer;
    //                         let to_layer = &projects[index].layer;
    //                         static EMPTY_VEC: &Vec<String> = &Vec::new();
    //                         let allowed_layers = match &config.global.allowed.get_layers(from_layer) {
    //                             Some(layers) => layers,
    //                             None => EMPTY_VEC,
    //                         };//.unwrap_or(&vec![]);
    //                         let ok = allowed_layers.contains(to_layer);
    //                         let label = format!("{} -> {}", project.name, projects[index].name);
    //                         edges_info.push(EdgeInfo { to: index, allowed: ok, label });
    //                     }
    //                 }
    //             }
    //         }
    //         project_dependencies.push(edges_info);
    //     }
    
    //     Ok(project_dependencies)
    // }    
}
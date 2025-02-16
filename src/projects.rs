// use std::collections::HashMap;
// use std::fs::File;
// use std::io::{BufReader, Error, Read};
// use std::path::Path;
// use path_slash::PathExt;
// use serde::Deserialize;
// use walkdir::WalkDir;

// use crate::configuration::{determine_layer, exclude_files_and_folders, exclude_projects, Config};
// use crate::graph::{EdgeInfo, GraphDependencies, Node, NodeDependencies};
// use crate::stringsutils::RemoveBom;

// #[derive(Debug, Deserialize, PartialEq)]
// #[serde(rename_all = "PascalCase")]
// struct ProjectReference {
//     include: String,
// }

// #[derive(Debug, Deserialize, PartialEq)]
// #[serde(rename_all = "PascalCase")]
// struct ItemGroup {
//     #[serde(rename = "ProjectReference", default)]
//     project_references: Vec<ProjectReference>,
// }

// #[derive(Debug, Deserialize, PartialEq)]
// #[serde(rename_all = "PascalCase")]
// pub struct Project {
//     #[serde(rename = "ToolsVersion", default)]
//     tools_version: Option<String>,
//     #[serde(rename = "ItemGroup", default)]
//     item_groups: Vec<ItemGroup>,
// }

// pub struct ProjectDependencyManager;

// impl GraphDependencies for ProjectDependencyManager {
//     fn collect_nodes(root_path: &Path, config: &Config) -> Result<Vec<Node>, Error> {
//         let mut projects = Vec::new();

//         for entry in WalkDir::new(root_path) {
//             let entry = entry?;
//             let path = entry.path();
//             let csharp_config = config.csharp.as_ref().unwrap();
//             if exclude_files_and_folders(&path.to_path_buf(), &csharp_config.exclude, &csharp_config.pattern, csharp_config.case_sensitive) {
//                 continue;
//             }
//             if path.extension().map_or(false, |e| e == "csproj") {
//                 let mut file = File::open(path)?;
//                 let mut contents = String::new();
//                 file.read_to_string(&mut contents)?;
//                 contents = contents.remove_bom();

//                 let _project: Project = match serde_xml_rs::from_str(&contents) {
//                     Ok(proj) => proj,
//                     Err(err) => {
//                         eprintln!(
//                             "Failed to parse .csproj file, possible incompatible file: {}, error: {}",
//                             path.display(),
//                             err
//                         );
//                         continue;
//                     }
//                 };

//                 let absolute_path = match path.to_str() {
//                     Some(p) => p.to_string(),
//                     None => continue,
//                 };

//                 let filename = match path.file_name().and_then(|f| f.to_str()) {
//                     Some(f) => f.to_string(),
//                     None => continue,
//                 };

//                 if exclude_projects(&filename, &csharp_config.exclude, &csharp_config.pattern, csharp_config.case_sensitive) {
//                     continue;
//                 }

//                 let layer = determine_layer(&filename, &csharp_config.projects, csharp_config.case_sensitive, &csharp_config.pattern);
//                 let color = config.get_color(&layer).cloned().unwrap_or_else(|| "gray".to_string());

//                 projects.push(Node {
//                     id: absolute_path,
//                     name: filename,
//                     node_type: "project".to_string(),
//                     layer,
//                     color,
//                 });
//             }
//         }

//         Ok(projects)
//     }

//     fn find_dependencies(nodes: &[Node], config: &Config) -> Result<NodeDependencies, Error> {
//         let mut node_dependencies = Vec::new();
//         let path_index_map: HashMap<String, usize> = nodes.iter().enumerate()
//             .map(|(index, project)| (project.id.clone(), index))
//             .collect();

//         for project in nodes {
//             let project_path = Path::new(&project.id); // The id is the absolute path
//             let file = File::open(project_path)?;
//             let file_reader = BufReader::new(file);
//             let csproj_data: Project = match serde_xml_rs::from_reader(file_reader) {
//                 Ok(data) => data,
//                 Err(err) => {
//                     eprintln!("Failed to parse .csproj file: {}", project.id);
//                     return Err(std::io::Error::new(std::io::ErrorKind::Other, err.to_string()));
//                 }
//             };

//             let mut edges_info = Vec::new();
//             let project_dir = match project_path.parent() {
//                 Some(dir) => dir,
//                 None => continue,
//             };

//             for item_group in &csproj_data.item_groups {
//                 for project_reference in &item_group.project_references {
//                     let normalized_path = if cfg!(target_os = "windows") {
//                         Path::new(&project_reference.include).to_slash_lossy().into_owned()
//                     } else {
//                         project_reference.include.replace("\\", "/")
//                     };
//                     let dep_path = project_dir.join(normalized_path);
//                     if let Ok(canonical_dep_path) = dep_path.canonicalize() {
//                         let dep_path_str = match canonical_dep_path.to_str() {
//                             Some(s) => s,
//                             None => continue,
//                         };
//                         if let Some(&index) = path_index_map.get(dep_path_str) {
//                             let from_layer = &project.layer;
//                             let to_layer = &nodes[index].layer;
//                             static EMPTY_VEC: &Vec<String> = &Vec::new();
//                             let allowed_layers = config.global.rules.get(from_layer).unwrap_or(EMPTY_VEC);
//                             let ok = allowed_layers.contains(to_layer);
//                             let label = format!("{} -> {}", project.name, nodes[index].name);
//                             edges_info.push(EdgeInfo { to: index, allowed: ok, label });
//                         }
//                     }
//                 }
//             }
//             node_dependencies.push(edges_info);
//         }

//         Ok(node_dependencies)
//     }
// }

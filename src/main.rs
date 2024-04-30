use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Error, Read};
use std::path::{Path, PathBuf};
use path_slash::PathExt;
use serde::Deserialize;
use walkdir::WalkDir;
use clap::{App, Arg};

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
struct Project {
    #[serde(rename = "ToolsVersion", default)]
    tools_version: Option<String>,
    #[serde(rename = "ItemGroup", default)]
    item_groups: Vec<ItemGroup>,
}

#[derive(Debug)]
struct CsProject {
    relative_path: String,
    absolute_path: String,
    name: String,
}

// Main entry point of the application
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Project Dependency Analyzer")
        .version("1.0")
        .author("Enrique")
        .about("Analyzes dependencies from C# project files for now")
        .arg(Arg::new("folder")
             .long("folder")
             .short('f')
             .value_name("PATH")
             .help("Sets a custom folder path")
             .takes_value(true))
        .arg(Arg::new("list")
             .long("list")
             .short('l')
             .help("Displays all found projects"))
        .arg(Arg::new("output")
             .long("output")
             .value_name("FORMAT")
             .help("Selects output format ('mermaid' or 'graphviz')")
             .takes_value(true))
        .arg(Arg::new("output-html")
             .long("output-html")
             .value_name("PATH")
             .help("Generates an HTML page with the specified output format")
             .takes_value(true)
             .requires("output"))
        .arg(Arg::new("detect-cycles")
             .long("detect-cycles")
             .help("Detects cycles in project dependencies"))
        .get_matches();

    let root_path = matches.value_of("folder")
        .map_or_else(|| PathBuf::from("."), PathBuf::from);

    let projects = collect_projects(&root_path)?;
    let project_dependencies = resolve_dependencies(&projects)?;

    if matches.is_present("list") {
        display_project_information(&projects, &project_dependencies);
    }

    if let Some(format) = matches.value_of("output") {
        if let Some(html_path) = matches.value_of("output-html") {
            generate_html_output(&projects, &project_dependencies, html_path, format)?;
        } else {
            match format {
                "mermaid" => generate_mermaid_diagram(&projects, &project_dependencies),
                "graphviz" => generate_graphviz_diagram(&projects, &project_dependencies),
                _ => eprintln!("Invalid format. Use 'mermaid' or 'graphviz'."),
            }
        }
    }

    if matches.is_present("detect-cycles") {
        detect_cycles(&projects, &project_dependencies);
    }

    Ok(())
}

fn generate_html_output(projects: &[CsProject], dependencies: &[Vec<usize>], path: &str, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating HTML output at '{}' using format '{}'", path, format);
    // Here you would implement the actual logic to generate and save the HTML output.
    Ok(())
}

/// Collects CsProject data from .csproj files found under the given root path
fn collect_projects(root_path: &Path) -> Result<Vec<CsProject>, Error> {
    let mut projects = Vec::new();

    for entry in WalkDir::new(root_path) {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "csproj") {
            let mut file = File::open(path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            // Parse XML to check for ToolsVersion
            let project: Project = match serde_xml_rs::from_str(&contents) {
                Ok(proj) => proj,
                Err(err) => {
                    eprintln!("Failed to parse .csproj file, possible incompatible file: {}, error: {}", path.display(), err);
                    continue; // Skip this file if parsing fails
                }
            };
            
            if project.tools_version.is_some() {
                println!("Skipping '{}' due to incompatible csproj file.", path.display());
                continue; // Skip this file if ToolsVersion is present
            }

            // If ToolsVersion is not present, process the file
            let relative_path = match path.strip_prefix(root_path) {
                Ok(stripped_path) => stripped_path.to_str().unwrap().to_string(),
                Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to strip prefix")),
            };
            let absolute_path = path.to_str().unwrap().to_string();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();

            projects.push(CsProject { relative_path, absolute_path, name });
        }
    }

    Ok(projects)
}

/// Resolves dependencies of each project
fn resolve_dependencies(projects: &[CsProject]) -> Result<Vec<Vec<usize>>, Error> {
    let mut project_dependencies = Vec::new();
    let path_index_map: HashMap<String, usize> = projects.iter().enumerate()
        .map(|(index, project)| (project.absolute_path.clone(), index))
        .collect();

    for project in projects {
        let project_path = Path::new(&project.absolute_path);
        let file = File::open(project_path)?;
        let file_reader = BufReader::new(file);
        let csproj_data: Project = match serde_xml_rs::from_reader(file_reader) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Failed to parse .csproj file: {}", project.absolute_path);
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

/// Displays basic information about projects and their dependencies
fn display_project_information(projects: &[CsProject], project_dependencies: &[Vec<usize>]) {
    println!("Found projects:");
    for (i, project) in projects.iter().enumerate() {
        println!("{}: {:?}", i, project);
    }

    println!("\nProject dependencies:");
    for (i, deps) in project_dependencies.iter().enumerate() {
        let dep_indices = deps.iter().map(usize::to_string).collect::<Vec<_>>().join(", ");
        println!("Project {}: {}", i, dep_indices);
    }
}

/// Generates a Mermaid diagram based on project dependencies
fn generate_mermaid_diagram(projects: &[CsProject], project_dependencies: &[Vec<usize>]) {
    println!("```mermaid");
    println!("graph TD;");
    for (index, project) in projects.iter().enumerate() {
        println!("    P{}[\"{}\"]", index + 1, project.name);
    }
    for (index, deps) in project_dependencies.iter().enumerate() {
        for dep in deps {
            println!("    P{} --> P{}", index + 1, dep + 1);
        }
    }
    println!("```");
}

/// Generates a Graphviz diagram based on project dependencies
fn generate_graphviz_diagram(projects: &[CsProject], project_dependencies: &[Vec<usize>]) {
    println!("digraph G {{");
    for (index, project) in projects.iter().enumerate() {
        println!("    P{} [label=\"{}\"]", index + 1, project.name);
    }
    for (index, deps) in project_dependencies.iter().enumerate() {
        for dep in deps {
            println!("    P{} -> P{}", index + 1, dep + 1);
        }
    }
    println!("}}");
}

/// Detects cycles within project dependencies using Depth-First Search (DFS)
fn detect_cycles(projects: &[CsProject], project_dependencies: &[Vec<usize>]) {
    let mut has_cycle = false;
    for i in 0..projects.len() {
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        if dfs(i, &mut stack, &mut visiting, &mut visited, project_dependencies, projects) {
            println!("Cycle initiated from project: {}", projects[i].name);
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
    deps: &[Vec<usize>],
    projects: &[CsProject],
) -> bool {
    if visiting.contains(&node) {
        // Cycle detected, print the cycle
        let cycle_start_index = stack.iter().position(|&x| x == node).unwrap();
        println!("Cycle detected in dependencies starting at '{}':", projects[node].name);
        for &index in &stack[cycle_start_index..] {
            print!("{} -> ", projects[index].name);
        }
        println!("{}", projects[node].name); // Complete the cycle
        return true;
    }

    if visited.contains(&node) {
        return false; // This node has been fully explored
    }

    visiting.insert(node);
    stack.push(node);

    for &next in &deps[node] {
        if dfs(next, stack, visiting, visited, deps, projects) {
            return true;
        }
    }

    stack.pop();
    visiting.remove(&node);
    visited.insert(node);
    false
}

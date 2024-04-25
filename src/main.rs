use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::collections::{HashMap, HashSet};
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

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
    #[serde(default)]
    item_group: Vec<ItemGroup>,
}

#[derive(Debug)]
struct CsProject {
    relative_path: String,
    absolute_path: String,
    name: String,
}

fn main() -> std::io::Result<()> {
    let mut projects: Vec<CsProject> = Vec::new();
    let mut project_dependencies: Vec<Vec<usize>> = Vec::new();
    let mut path_index_map: HashMap<String, usize> = HashMap::new();

    // let root_path = Path::new("/home/enrique/sites/csharp-architecture");
    let root_path = Path::new("/home/enrique/sites/attendant");

    // Fase 1: Recolectar todos los archivos .csproj
    for entry in WalkDir::new(root_path) {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "csproj") {
            let relative_path = match path.strip_prefix(root_path) {
                Ok(stripped_path) => stripped_path.to_str().unwrap().to_string(),
                Err(_) => {
                    eprintln!("Failed to strip prefix from path");
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to strip prefix from path"));
                }
            };
            let absolute_path = path.to_str().unwrap().to_string();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();

            projects.push(CsProject {
                relative_path: relative_path.clone(),
                absolute_path: absolute_path.clone(),
                name: name.clone(),
            });
            path_index_map.insert(absolute_path.clone(), projects.len() - 1);
        }
    }

    // Fase 2: Resolver dependencias
    for project in &projects {
        let project_path = Path::new(&project.absolute_path);
        let file = File::open(project_path)?;
        let file_reader = BufReader::new(file);
        let csproj_data: Project = serde_xml_rs::from_reader(file_reader).unwrap();

        let mut deps = Vec::new();
        let project_dir = project_path.parent().unwrap();

        for item_group in &csproj_data.item_group {
            for project_reference in &item_group.project_references {
                let normalized_path = project_reference.include.replace("\\", "/");
                let dep_path = project_dir.join(normalized_path);
                let canonical_dep_path = match dep_path.canonicalize() {
                    Ok(path) => path,
                    Err(_) => continue,  // Ignorar si no puede ser canonicalizado
                };
                let dep_path_str = canonical_dep_path.to_str().unwrap();
                if let Some(index) = path_index_map.get(dep_path_str) {
                    deps.push(*index ); // Index adjustment for 1-based index
                }
            }
        }
        project_dependencies.push(deps);
    }

    println!("Proyectos encontrados:");
    for (i, project) in projects.iter().enumerate() {
        println!("Índice {}: {:?}", i + 1, project);
    }
    println!("\nDependencias de los proyectos:");
    for (i, deps) in project_dependencies.iter().enumerate() {
        let dep_indices = deps.iter().map(usize::to_string).collect::<Vec<_>>().join(", ");
        println!("Proyecto {}: {}", i, dep_indices);
    }

    // Fase 3: Generación de diagrama Mermaid
    println!("```mermaid");
    println!("graph TD;");
    for (index, project) in projects.iter().enumerate() {
        println!("    P{}[\"{}\"]", index + 1, project.name);
    }
    for (index, deps) in project_dependencies.iter().enumerate() {
        for dep in deps {
            println!("    P{} --> P{}", index + 1, dep);
        }
    }
    println!("```");

    // Phase 4: Generate Graphviz diagram
    println!("digraph G {{");
    for (index, project) in projects.iter().enumerate() {
        println!("    P{} [label=\"{}\"]", index + 1, project.name);
    }
    for (index, deps) in project_dependencies.iter().enumerate() {
        for dep in deps {
            println!("    P{} -> P{}", index + 1, dep+1);
        }
    }
    println!("}}");
    
    // Detectar ciclos en las dependencias
    fn dfs(
        node: usize,
        stack: &mut Vec<usize>,
        visiting: &mut HashSet<usize>,
        visited: &mut HashSet<usize>,
        deps: &[Vec<usize>],
        projects: &[CsProject],
    ) -> bool {
        if visiting.contains(&node) {
            // Ciclo detectado, imprimir ciclo
            let cycle_start_index = stack.iter().position(|&x| x == node).unwrap();
            println!("Ciclo detectado en las dependencias:");
            for &index in &stack[cycle_start_index..] {
                print!("{} -> ", projects[index].name);
            }
            println!("{}", projects[node].name); // Completar el ciclo
            return true;
        }
    
        if visited.contains(&node) && stack.contains(&node) {
            return false; // Este nodo ya ha sido explorado en la pila actual
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
    
    // Invocar dfs en el loop principal
    let mut has_cycle = false;
    for i in 0..projects.len() {
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        if dfs(i, &mut stack, &mut visiting, &mut visited, &project_dependencies, &projects) {
            println!("Ciclo iniciado desde el proyecto: {}", projects[i].name);
            has_cycle = true;
        }
    }
    if !has_cycle {
        println!("No se detectaron dependencias circulares.");
    }
    
    

    Ok(())
}

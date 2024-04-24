use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::collections::HashMap;
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

    let root_path = Path::new("/home/enrique/sites/csharp-architecture");

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
                    deps.push(*index+1);
                }
            }
        }
        project_dependencies.push(deps);
    }

    println!("Proyectos encontrados:");
    for project in &projects {
        println!("{:?}", project);
    }
    println!("\nDependencias de los proyectos:");
    for (i, deps) in project_dependencies.iter().enumerate() {
        let dep_indices = deps.iter().map(usize::to_string).collect::<Vec<_>>().join(", ");
        println!("Proyecto {}: {}", i + 1, dep_indices);
    }

    Ok(())
}

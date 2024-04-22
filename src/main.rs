use quick_xml::Reader;
use quick_xml::events::Event;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::collections::HashMap;

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
        let mut contents = std::fs::read_to_string(project_path)?;
        let mut reader = Reader::from_str(&contents);
        reader.trim_text(true);

        let mut buf = Vec::new();
        let mut deps = Vec::new();
        let project_dir = project_path.parent().unwrap();
        println!("File: {:?}", project_path);
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) if e.name() == b"ProjectReference" => {
                    println!("ProjectReference found");
                    if let Some(attr) = e.attributes().filter_map(|a| a.ok()).find(|a| a.key == b"Include") {
                        println!("Include found: {:?}", attr.value);
                        let dep_path = project_dir.join(std::str::from_utf8(&attr.value).unwrap());
                        let canonical_dep_path = dep_path.canonicalize()?;
                        let dep_path_str = canonical_dep_path.to_str().unwrap();
                        if let Some(index) = path_index_map.get(dep_path_str) {
                            deps.push(*index);
                        }
                    }
                },
                Ok(Event::Eof) => break,
                _ => (),
            }
            buf.clear();
        }
        project_dependencies.push(deps);
    }

    // Imprimir resultados
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

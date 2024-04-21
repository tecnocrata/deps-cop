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
    let mut project_dependencies: Vec<String> = Vec::new();
    let mut path_index_map: HashMap<String, usize> = HashMap::new();

    let root_path = Path::new("/home/enrique/sites/csharp-architecture"); // Ajusta esto a tu directorio raíz

    // Explorar directorios y encontrar archivos .csproj
    for entry in WalkDir::new(root_path) {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "csproj") {
            // let relative_path = path.strip_prefix(root_path)?.to_str().unwrap().to_string();
            let relative_path = match path.strip_prefix(root_path) {
                Ok(stripped_path) => stripped_path.to_str().unwrap().to_string(),
                Err(_) => {
                    // Handle the error here. For example, you can print an error message and return from the function.
                    eprintln!("Failed to strip prefix from path");
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to strip prefix from path"));
                }
            };
            let absolute_path = path.to_str().unwrap().to_string();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();

            projects.push(CsProject {
                relative_path: relative_path.clone(),
                absolute_path,
                name: name.clone(),
            });
            path_index_map.insert(relative_path.clone(), projects.len() - 1);

            // Leer y parsear el archivo .csproj
            let mut contents = std::fs::read_to_string(path)?;
            let mut reader = Reader::from_str(&contents);
            reader.trim_text(true);

            let mut buf = Vec::new();
            let mut deps = Vec::new();

            // Extraer referencias de proyectos
            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Start(ref e)) if e.name() == b"ProjectReference" => {
                        if let Some(attr) = e.attributes().filter_map(|a| a.ok()).find(|a| a.key == b"Include") {
                            let path = std::str::from_utf8(&attr.value).unwrap();
                            if let Some(index) = path_index_map.get(path) {
                                deps.push(*index);
                            }
                        }
                    },
                    Ok(Event::Eof) => break,
                    _ => (),
                }
                buf.clear();
            }
            if !deps.is_empty() {
                // let dep_string = deps.into_iter().map(usize::to_string).collect::<Vec<_>>().join(", ");
                let dep_string = deps.into_iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ");
                project_dependencies.push(dep_string);
            } else {
                project_dependencies.push("".to_string());
            }
        }
    }

    // Imprimir resultados
    println!("Proyectos encontrados:");
    for project in &projects {
        println!("{:?}", project);
    }
    println!("\nDependencias de los proyectos:");
    for (i, deps) in project_dependencies.iter().enumerate() {
        println!("Proyecto {}: {}", i + 1, deps);
    }

    Ok(())
}

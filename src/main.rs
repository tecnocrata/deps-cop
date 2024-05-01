use std::path::PathBuf;
use clap::{App, Arg};
use std::fs::File;
use std::io::{Write, Error};

mod projects;
use projects::Node;
use projects::{ProjectDependencyManager, ProjectDependencies};


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

    let projects = ProjectDependencyManager::collect_projects(&root_path)?;
    let project_dependencies = ProjectDependencyManager::find_dependencies(&projects)?;

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
        ProjectDependencyManager::detect_cycles(&projects, &project_dependencies);
    }

    Ok(())
}

fn generate_html_output(nodes: &[Node], node_dependencies: &[Vec<usize>], path: &str, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating HTML output at '{}' using format '{}'", path, format);

    let mut file = File::create(path)?;
    
    writeln!(file, "<!DOCTYPE html>")?;
    writeln!(file, "<html lang=\"en\">")?;
    writeln!(file, "<head>")?;
    writeln!(file, "    <meta charset=\"UTF-8\">")?;
    writeln!(file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
    writeln!(file, "    <title>Dependencies Analyzer</title>")?;
    writeln!(file, "    <link href=\"https://cdn.jsdelivr.net/npm/tailwindcss@3.1.0/dist/tailwind.min.css\" rel=\"stylesheet\">")?;
    // writeln!(file, "    <style>")?;
    // writeln!(file, "        .main-color {{ background-color: #4a5568; }}")?;
    // writeln!(file, "        .secondary-color {{ background-color: #718096; }}")?;
    // writeln!(file, "        .accent-color {{ background-color: #e2e8f0; }}")?;
    // writeln!(file, "    </style>")?;
    generate_header_content(&mut file, format)?;
    writeln!(file, "</head>")?;
    writeln!(file, "<body class=\"accent-color text-gray-800\">")?;
    writeln!(file, "    <header class=\"text-center p-4 secondary-color\">")?;
    writeln!(file, "        <h1>Dependencies Analyzer</h1>")?;
    writeln!(file, "        <p>This page was generated automatically.</p>")?;
    writeln!(file, "    </header>")?;
    writeln!(file, "    <section class=\"flex justify-center items-center p-4 h-screen\">")?;
    writeln!(file, "        <div class=\"w-full\">")?;
    generate_body_content(&mut file, format, nodes, node_dependencies)?;
    writeln!(file, "        </div>")?;
    writeln!(file, "    </section>")?;
    writeln!(file, "    <footer class=\"text-center p-4 secondary-color\">")?;
    writeln!(file, "        <p>Generated on: <script>document.write(new Date().toLocaleString());</script></p>")?;
    writeln!(file, "        <p>Everything was generated using Rust.</p>")?;
    writeln!(file, "        <img src=\"https://www.rust-lang.org/logos/rust-logo-blk.svg\" alt=\"Rust Logo\" class=\"h-8 mx-auto\">")?;
    writeln!(file, "    </footer>")?;
    generate_script_code(&mut file, format, nodes, node_dependencies)?;
    writeln!(file, "</body>")?;
    writeln!(file, "</html>")?;
    
    Ok(())
}

fn generate_header_content(file: &mut File, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    if format == "graphviz" {
        writeln!(file, "    <style>")?;
        writeln!(file, "        .main-color {{ background-color: #4a5568; }}")?;
        writeln!(file, "        .secondary-color {{ background-color: #718096; }}")?;
        writeln!(file, "        .accent-color {{ background-color: #ffffff; color: #333; }}")?;
        writeln!(file, "        #graph-container {{")?;
        writeln!(file, "            width: 100%; height: 80vh; overflow: auto; border: 1px solid #ccc;")?;
        writeln!(file, "        }}")?;
        writeln!(file, "    </style>")?;
        writeln!(file, "<script src=\"https://cdnjs.cloudflare.com/ajax/libs/viz.js/2.1.2/viz.js\"></script>")?;
        writeln!(file, "<script src=\"https://cdnjs.cloudflare.com/ajax/libs/viz.js/2.1.2/full.render.js\" integrity=\"sha512-1zKK2bG3QY2JaUPpfHZDUMe3dwBwFdCDwXQ01GrKSd+/l0hqPbF+aak66zYPUZtn+o2JYi1mjXAqy5mW04v3iA==\" crossorigin=\"anonymous\" referrerpolicy=\"no-referrer\"></script>")?;
    } else
    if format == "sigma" {
        writeln!(file, "<script src=\"https://cdnjs.cloudflare.com/ajax/libs/sigma.js/2.0.0/sigma.min.js\"></script>")?;
    }
    Ok(())
}

fn generate_script_code(file: &mut File, format: &str, nodes: &[Node], node_dependencies: &[Vec<usize>]) -> Result<(), Box<dyn std::error::Error>> {
    if format == "graphviz" {
        writeln!(file, "<script>")?;
        writeln!(file, "    var viz = new Viz();")?;
        writeln!(file, "    var graphvizData = `")?;
        writeln!(file, "digraph G {{")?;
    for (index, node) in nodes.iter().enumerate() {
        writeln!(file, "    P{} [label=\"{}\"]", index + 1, node.name)?;
    }
    for (index, deps) in node_dependencies.iter().enumerate() {
        for dep in deps {
            writeln!(file, "    P{} -> P{}", index + 1, dep + 1)?;
        }
    }
        writeln!(file, "}}`;")?;

        writeln!(file, "    viz.renderSVGElement(graphvizData)")?;
        writeln!(file, "            .then(function(element) {{")?;
        writeln!(file, "                document.getElementById('graph').appendChild(element);")?;
        writeln!(file, "    }})")?;
        writeln!(file, "    .catch(error => {{")?;
        writeln!(file, "     console.error('Error rendering graph:', error);")?;
        writeln!(file, "    }});")?;
        writeln!(file, "</script>")?;
    } else
    if format == "sigma" {
        writeln!(file, "<script>")?;
        writeln!(file, "    let graph = document.querySelector('p').textContent;")?;
        writeln!(file, "    let s = new sigma({{")?;
        writeln!(file, "        container: 'graph-container',")?;
        writeln!(file, "        graph: graph,")?;
        writeln!(file, "    }});")?;
        writeln!(file, "</script>")?;
    }
    Ok(())
}

fn generate_body_content(file: &mut File, format: &str, nodes: &[Node], node_dependencies: &[Vec<usize>]) -> Result<(), Box<dyn std::error::Error>> {
    if format == "graphviz" {
        generate_body_content_graphviz(file, nodes, node_dependencies)?;
    } else if format == "sigma" {
        generate_body_content_sigma(file)?;
    }
    Ok(())
}
fn generate_body_content_graphviz(file: &mut File, nodes: &[Node], node_dependencies: &[Vec<usize>]) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "            <div id=\"graph-container\">")?;
    writeln!(file, "                <div id=\"graph\"></div>")?;
    writeln!(file, "            </div>")?;
    Ok(())
}

fn generate_body_content_sigma(file: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    // Implementación de Sigma va aquí. Actualmente está vacío según instrucciones.
    writeln!(file, "<div class=\"bg-accent-color h-full flex justify-center items-center\">")?;
    writeln!(file, "<p>Placeholder for Sigma directed graph.</p>")?;
    writeln!(file, "</div>")?;
    Ok(())
}

/// Displays basic information about projects and their dependencies
fn display_project_information(projects: &[Node], node_dependencies: &[Vec<usize>]) {
    println!("Found projects:");
    for (i, project) in projects.iter().enumerate() {
        println!("{}: {:?}", i, project);
    }

    println!("\nProject dependencies:");
    for (i, deps) in node_dependencies.iter().enumerate() {
        let dep_indices = deps.iter().map(usize::to_string).collect::<Vec<_>>().join(", ");
        println!("Project {}: {}", i, dep_indices);
    }
}

/// Generates a Mermaid diagram based on project dependencies
fn generate_mermaid_diagram(nodes: &[Node], node_dependencies: &[Vec<usize>]) {
    println!("```mermaid");
    println!("graph TD;");
    for (index, project) in nodes.iter().enumerate() {
        println!("    P{}[\"{}\"]", index + 1, project.name);
    }
    for (index, deps) in node_dependencies.iter().enumerate() {
        for dep in deps {
            println!("    P{} --> P{}", index + 1, dep + 1);
        }
    }
    println!("```");
}

/// Generates a Graphviz diagram based on project dependencies
fn generate_graphviz_diagram(nodes: &[Node], node_dependencies: &[Vec<usize>]) {
    println!("digraph G {{");
    for (index, project) in nodes.iter().enumerate() {
        println!("    P{} [label=\"{}\"]", index + 1, project.name);
    }
    for (index, deps) in node_dependencies.iter().enumerate() {
        for dep in deps {
            println!("    P{} -> P{}", index + 1, dep + 1);
        }
    }
    println!("}}");
}
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
    writeln!(file, "    <title>Dependencies Cop</title>")?;
    writeln!(file, "    <link href=\"https://cdn.jsdelivr.net/npm/tailwindcss@3.1.0/dist/tailwind.min.css\" rel=\"stylesheet\">")?;
    writeln!(file, "    <style>")?;
    writeln!(file, "        .main-color {{ background-color: #4a5568; }}")?;
    writeln!(file, "        .secondary-color {{ background-color: #718096; }}")?;
    writeln!(file, "        .accent-color {{ background-color: #e2e8f0; }}")?;
    writeln!(file, "    </style>")?;
    writeln!(file, "</head>")?;
    writeln!(file, "<body class=\"main-color text-accent-color\">")?;
    writeln!(file, "    <header class=\"text-center p-4 secondary-color\">")?;
    writeln!(file, "        <h1>Dependencies Cop</h1>")?;
    writeln!(file, "        <p>This page was generated automatically.</p>")?;
    writeln!(file, "    </header>")?;
    writeln!(file, "    <section class=\"flex justify-center items-center p-4 h-screen\">")?;
    writeln!(file, "        <div class=\"w-full\">")?;
    if format == "graphviz" {
        generate_graphviz_diagram_html(&mut file, nodes, node_dependencies)?;
    } else if format == "sigma" {
        generate_sigma_html(&mut file)?;
    }
    writeln!(file, "        </div>")?;
    writeln!(file, "    </section>")?;
    writeln!(file, "    <footer class=\"text-center p-4 secondary-color\">")?;
    writeln!(file, "        <p>Generated on: <script>document.write(new Date().toLocaleString());</script></p>")?;
    writeln!(file, "        <p>Everything was generated using Rust.</p>")?;
    writeln!(file, "        <img src=\"https://www.rust-lang.org/logos/rust-logo-blk.svg\" alt=\"Rust Logo\" class=\"h-8 mx-auto\">")?;
    writeln!(file, "    </footer>")?;
    writeln!(file, "</body>")?;
    writeln!(file, "</html>")?;
    
    Ok(())
}

fn generate_graphviz_diagram_html(file: &mut File, nodes: &[Node], node_dependencies: &[Vec<usize>]) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "<div class=\"bg-accent-color h-full flex justify-center items-center\">")?;
    writeln!(file, "<p>digraph G {{")?;
    for (index, node) in nodes.iter().enumerate() {
        writeln!(file, "    P{} [label=\"{}\"]", index + 1, node.name)?;
    }
    for (index, deps) in node_dependencies.iter().enumerate() {
        for dep in deps {
            writeln!(file, "    P{} -> P{}", index + 1, dep + 1)?;
        }
    }
    writeln!(file, "}}</p>")?;
    writeln!(file, "</div>")?;
    Ok(())
}

fn generate_sigma_html(file: &mut File) -> Result<(), Box<dyn std::error::Error>> {
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
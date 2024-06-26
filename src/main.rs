use std::{collections::HashMap, path::PathBuf};
use std::env;
use clap::{App, Arg, AppSettings};

mod graph;
mod projects;
mod static_output;
mod configuration;
mod namespaces;
mod stringsutils;

use configuration::load_config;
use namespaces::NamespaceDependencyManager;
use graph::{detect_cycles, GraphDependencies, Node, NodeDependencies};
use projects::ProjectDependencyManager;
use static_output::{generate_html_output, generate_mermaid_diagram, generate_graphviz_diagram, display_graph_information};


// Main entry point of the application
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Dependency Analyzer Cop")
        .version("0.1.45")
        .mut_arg("version", |a| a.short('v'))  // It shows the version with -v
        .author("tecnocrata <")
        .about("Analyzes dependencies from project files")
        .setting(AppSettings::ArgRequiredElseHelp)  // it shows the help if no arguments are provided
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
             .help("Selects output format ('d3', 'mermaid' or 'graphviz')")
             .takes_value(true)
             .requires("folder"))
        .arg(Arg::new("output-html")
             .long("output-html")
             .value_name("PATH")
             .help("Generates an HTML page with the specified output format")
             .takes_value(true)
             .requires("output"))
        .arg(Arg::new("detect-cycles")
             .long("detect-cycles")
             .help("Detects cycles in project dependencies")
             .requires("folder"))
        .arg(Arg::new("analysis")
             .long("analysis")
             .short('a')
             .value_name("TYPE")
             .help("Specifies the analysis type (default: 'csharp:projects')")
             .takes_value(true)
             .default_value("csharp:projects")
             .requires("folder"))
        .get_matches();

    // Get the current directory
    let current_dir = env::current_dir()?;

    let root_path = matches.value_of("folder")
    .map_or_else(|| Ok(current_dir.clone()), |p| {
        let path = PathBuf::from(p);
        let complete_path = if path.is_relative() {
            current_dir.join(path)
        } else {
            path
        };
        // Canonicalize the path to resolve any '.' or '..'
        complete_path.canonicalize()
    })?;

    let config = load_config(&root_path);
    // println!("Configuration: {:#?}", config);

    let analysis = matches.value_of("analysis").unwrap();
    let layers: Vec<Node> = get_layers(&config);
    let layer_dependencies: NodeDependencies = get_layer_dependencies (&layers, &config.global.rules);

    match analysis {
        "csharp:projects" => {
            let nodes = ProjectDependencyManager::collect_nodes(&root_path, &config)?;
            let project_dependencies = ProjectDependencyManager::find_dependencies(&nodes, &config)?;

            generate_output(&matches, &nodes, &project_dependencies, &layers, &layer_dependencies)?;
        }
        "csharp:namespaces" => {
            let nodes = NamespaceDependencyManager::collect_nodes(&root_path, &config)?;
            let namespace_dependencies = NamespaceDependencyManager::find_dependencies(&root_path, &nodes, &config)?;

            generate_output(&matches, &nodes, &namespace_dependencies, &layers, &layer_dependencies)?;
        }
        // "javascript:folders" => {
        //     let folder_dependencies = JavaScriptDependencyManager::find_folder_dependencies(&root_path, &config)?;

        //     generate_output(&matches, &folder_dependencies.nodes, &folder_dependencies.dependencies)?;
        // }
        _ => {
            eprintln!("Unsupported analysis type. Please specify 'csharp:projects', 'csharp:namespaces', or 'javascript:folders'.");
        }
    }

    Ok(())
}

fn get_layer_dependencies(layers: &[Node], rules: &configuration::Rules) -> Vec<Vec<graph::EdgeInfo>> {
    // Precompute layer indices for quick lookup
    let layer_indices: HashMap<&String, usize> = layers.iter().enumerate()
        .map(|(index, layer)| (&layer.id, index))
        .collect();

    layers.iter().map(|layer| {
        rules.rules.get(&layer.id).unwrap_or(&Vec::new()).iter().map(|layer_rule| {
            let to_layer_index = *layer_indices.get(layer_rule).unwrap();
            let to_layer = &layers[to_layer_index];
            let label = format!("{} -> {}", layer.name, to_layer.name);
            graph::EdgeInfo { to: to_layer_index, allowed: true, label }
        }).collect()
    }).collect()
}

fn get_layers(config: &configuration::Config) -> Vec<Node> {
    let mut layers = Vec::new();
    for layer in &config.global.layers {
        layers.push(Node {
            id: layer.clone(),
            name: layer.clone(),
            layer: "layer".to_string(),
            node_type: "layer".to_string(),
            color: match config.global.colors.colors.get(layer) {
                Some(color) => color.clone(),
                None => "gray".to_string(), // Or handle the None case as needed
            },
        });
    }
    layers
}


fn generate_output(matches: &clap::ArgMatches, nodes: &[Node], dependencies: &NodeDependencies, layers: &[Node], layer_dependencies: &NodeDependencies) -> Result<(), Box<dyn std::error::Error>> {
    // display the number of elements that nodes and dependencies have
    println!("Nodes: {}", nodes.len());
    println!("Dependencies: {}", dependencies.len());
    println!("Layers: {}", layers.len());
    println!("Layer Dependencies: {}", layer_dependencies.len());
    if matches.is_present("list") {
        display_graph_information(&nodes, &dependencies);
        display_graph_information(&layers, &layer_dependencies)
    }

    if let Some(format) = matches.value_of("output") {
        if let Some(html_path) = matches.value_of("output-html") {
            generate_html_output(&nodes, &dependencies, &layers, &layer_dependencies, html_path, format)?;
        } else {
            match format {
                "mermaid" => generate_mermaid_diagram(&nodes, &dependencies),
                "graphviz" => generate_graphviz_diagram(&nodes, &dependencies, &layers, &layer_dependencies),
                "d3" => eprintln!("D3 output is only available for HTML output."),
                _ => eprintln!("Invalid format. Use 'mermaid' or 'graphviz'."),
            }
        }
    }

    if matches.is_present("detect-cycles") {
        detect_cycles(&nodes, &dependencies);
    }

    Ok(())
}

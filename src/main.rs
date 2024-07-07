use std::{collections::HashMap, fs::File, path::PathBuf};
use std::env;
use clap::{App, Arg, AppSettings};
use serde_json::{self, to_writer_pretty};

mod graph;
mod projects;
mod static_output;
mod configuration;
mod namespaces;
mod stringsutils;

use configuration::{load_config, Config};
use namespaces::NamespaceDependencyManager;
use graph::{detect_cycles, GraphDependencies, Node, NodeDependencies};
use projects::ProjectDependencyManager;
use static_output::{generate_html_output, generate_mermaid_diagram, generate_graphviz_diagram, display_graph_information};

// Main entry point of the application
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Dependency Analyzer Cop")
        .version("0.1.47")
        .mut_arg("version", |a| a.short('v'))  // It shows the version with -v
        .author("tecnocrata <")
        .about("Analyzes dependencies from project files")
        .setting(AppSettings::ArgRequiredElseHelp)  // it shows the help if no arguments are provided
        .arg(Arg::new("folder")
             .long("folder")
             .short('f')
             .value_name("PATH")
             .help("Sets a custom folder path")
             .takes_value(true)
             .required_unless_present("generate-config"))
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
        .arg(Arg::new("generate-config")
             .long("generate-config")
             .short('g')
             .value_name("LANGUAGES")
             .help("Generates the default configuration file for the specified languages (comma-separated, e.g., 'csharp,javascript')")
             .takes_value(true)
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

    if let Some(languages) = matches.value_of("generate-config") {
        generate_default_config(&root_path, languages)?;
        println!("Configuration file generated at: {:?}", root_path.join("depscoprc.json"));
        return Ok(());
    }

    let config = load_config(&root_path);
    // println!("Configuration: {:#?}", config);

    let analysis = matches.value_of("analysis").unwrap();
    let layers: Vec<Node> = get_layers(&config);
    let layer_dependencies: NodeDependencies = get_layer_dependencies (&layers, &config.global.rules);

    let result = match analysis {
        "csharp:projects" => {
            let nodes = ProjectDependencyManager::collect_nodes(&root_path, &config)?;
            let project_dependencies = ProjectDependencyManager::find_dependencies(&nodes, &config)?;

            generate_output(&matches, &nodes, &project_dependencies, &layers, &layer_dependencies, &config)
        }
        "csharp:namespaces" => {
            let nodes = NamespaceDependencyManager::collect_nodes(&root_path, &config)?;
            let namespace_dependencies = NamespaceDependencyManager::find_dependencies(&root_path, &nodes, &config)?;

            generate_output(&matches, &nodes, &namespace_dependencies, &layers, &layer_dependencies, &config)
        }
        // "javascript:folders" => {
        //     let folder_dependencies = JavaScriptDependencyManager::find_folder_dependencies(&root_path, &config)?;

        //     generate_output(&matches, &folder_dependencies.nodes, &folder_dependencies.dependencies)?;
        // }
        _ => {
            eprintln!("Unsupported analysis type. Please specify 'csharp:projects', 'csharp:namespaces', or 'javascript:folders'.");
            Err(Box::from("Unsupported analysis type"))
        }
    };

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn get_layer_dependencies(layers: &[Node], rules: &HashMap<String, Vec<String>>) -> Vec<Vec<graph::EdgeInfo>> {
    // Precompute layer indices for quick lookup
    let layer_indices: HashMap<&String, usize> = layers.iter().enumerate()
        .map(|(index, layer)| (&layer.id, index))
        .collect();

    layers.iter().map(|layer| {
        rules.get(&layer.id).unwrap_or(&Vec::new()).iter().map(|layer_rule| {
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
            color: match config.global.colors.get(layer) {
                Some(color) => color.clone(),
                None => "gray".to_string(), // Or handle the None case as needed
            },
        });
    }
    layers
}

fn generate_output(matches: &clap::ArgMatches, nodes: &[Node], dependencies: &NodeDependencies, layers: &[Node], layer_dependencies: &NodeDependencies, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
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
            generate_html_output(&nodes, &dependencies, &layers, &layer_dependencies, html_path, format, &config.global.toggles)?;
        } else {
            match format {
                "mermaid" => generate_mermaid_diagram(&nodes, &dependencies),
                "graphviz" => generate_graphviz_diagram(&nodes, &dependencies, &layers, &layer_dependencies, &config.global.toggles),
                "d3" => eprintln!("D3 output is only available for HTML output."),
                _ => eprintln!("Invalid format. Use 'mermaid' or 'graphviz'."),
            }
        }
    }

    if matches.is_present("detect-cycles") {
        let has_cycle = detect_cycles(&nodes, &dependencies);
        if has_cycle {
            eprintln!("Cycle detected in dependencies.");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn generate_default_config(folder: &PathBuf, languages: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::default();
    let langs: Vec<&str> = languages.split(',').collect();

    // Filter config based on the provided languages
    if !langs.contains(&"csharp") {
        config.csharp = None;
    }
    if !langs.contains(&"javascript") {
        config.javascript = None;
    }

    // Clean up the configuration to remove any None values
    let mut config_map = serde_json::to_value(&config)?.as_object().unwrap().clone();
    config_map.retain(|_, v| !v.is_null());

    let config_path = folder.join("depscoprc.json");
    let file = File::create(config_path)?;
    to_writer_pretty(file, &config_map)?;
    Ok(())
}
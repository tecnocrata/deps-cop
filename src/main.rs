use std::{collections::HashMap, fs::File, path::PathBuf};
use std::env;
use clap::{Parser, ValueEnum};
use serde_json::{self, to_writer_pretty};

mod core;
mod config;
mod analyzers;
mod output;
mod utils;

use config::loader::load_config;
use config::types::Config;
use analyzers::csharp::namespace::NamespaceDependencyManager;
use core::analysis::{detect_cycles, GraphDependencies};
use core::node::Node;
use core::dependencies::NodeDependencies;
use analyzers::csharp::project::ProjectDependencyManager;
use output::static_output::{generate_html_output, generate_mermaid_diagram, generate_graphviz_diagram, display_graph_information};

#[derive(Parser)]
#[command(
    name = "Dependency Analyzer Cop",
    version = "0.2.0",
    author = "tecnocrata",
    about = "Analyzes dependencies from project files",
    long_about = None,
    arg_required_else_help = true
)]
struct Cli {
    /// Sets a custom folder path
    #[arg(
        short = 'f',
        long = "folder",
        value_name = "PATH",
    )]
    path: String,

    /// Type of analysis to perform
    #[arg(
        short = 'a',
        long,
        value_name = "TYPE",
        default_value = "csharp:projects",
        requires = "path"
    )]
    analysis: String,

    /// Generate configuration file
    #[arg(
        short = 'g',
        long = "generate-config",
        value_name = "LANGUAGES",
        help = "Generates the default configuration file for the specified languages (comma-separated, e.g., 'csharp,javascript')"
    )]
    generate_config: Option<String>,

    /// List dependencies
    #[arg(
        short = 'l',
        long,
        help = "Displays all found projects"
    )]
    list: bool,

    /// Detect cycles in dependencies
    #[arg(
        short = 'c',
        long = "detect-cycles",
        help = "Detects cycles in project dependencies",
        requires = "path"
    )]
    detect_cycles: bool,

    /// Output file path for HTML
    #[arg(
        long = "output-html",
        value_name = "PATH",
        help = "Generates an HTML page with the specified output format",
        requires = "output"
    )]
    output_html: Option<String>,

    /// Output format (mermaid, graphviz, d3)
    #[arg(
        short,
        long,
        value_name = "FORMAT",
        help = "Selects output format ('d3', 'mermaid' or 'graphviz')",
        requires = "path"
    )]
    output: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Graphviz,
    D3,
}

// Main entry point of the application
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Get the current directory
    let current_dir = env::current_dir()?;

    let root_path = PathBuf::from(&cli.path);
    let complete_path = if root_path.is_relative() {
        current_dir.join(root_path)
    } else {
        root_path
    };
    // Canonicalize the path to resolve any '.' or '..'
    let root_path = complete_path.canonicalize()?;

    if let Some(languages) = cli.generate_config {
        generate_default_config(&root_path, &languages)?;
        println!("Configuration file generated at: {:?}", root_path.join("depscoprc.json"));
        return Ok(());
    }

    let config = load_config(&root_path);

    let analysis = cli.analysis.as_str();
    let layers: Vec<Node> = get_layers(&config);
    let layer_dependencies: NodeDependencies = get_layer_dependencies (&layers, &config.global.rules);

    let result = match analysis {
        "csharp:projects" => {
            let nodes = ProjectDependencyManager::collect_nodes(&root_path, &config)?;
            let project_dependencies = ProjectDependencyManager::find_dependencies(&nodes, &config)?;

            generate_output(&cli, &nodes, &project_dependencies, &layers, &layer_dependencies, &config)
        }
        "csharp:namespaces" => {
            let nodes = NamespaceDependencyManager::collect_nodes(&root_path, &config)?;
            let namespace_dependencies = NamespaceDependencyManager::find_dependencies(&root_path, &nodes, &config)?;

            generate_output(&cli, &nodes, &namespace_dependencies, &layers, &layer_dependencies, &config)
        }
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

fn get_layer_dependencies(layers: &[Node], rules: &HashMap<String, Vec<String>>) -> Vec<Vec<core::dependencies::EdgeInfo>> {
    // Precompute layer indices for quick lookup
    let layer_indices: HashMap<&String, usize> = layers.iter().enumerate()
        .map(|(index, layer)| (&layer.id, index))
        .collect();

    layers.iter().map(|layer| {
        rules.get(&layer.id).unwrap_or(&Vec::new()).iter().map(|layer_rule| {
            let to_layer_index = *layer_indices.get(layer_rule).unwrap();
            let to_layer = &layers[to_layer_index];
            let label = format!("{} -> {}", layer.name, to_layer.name);
            core::dependencies::EdgeInfo { to: to_layer_index, allowed: true, label }
        }).collect()
    }).collect()
}

fn get_layers(config: &config::types::Config) -> Vec<Node> {
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

fn generate_output(
    cli: &Cli,
    nodes: &[Node],
    dependencies: &NodeDependencies,
    layers: &[Node],
    layer_dependencies: &NodeDependencies,
    config: &Config
) -> Result<(), Box<dyn std::error::Error>> {
    // display the number of elements that nodes and dependencies have
    println!("Nodes: {}", nodes.len());
    println!("Dependencies: {}", dependencies.len());
    println!("Layers: {}", layers.len());
    println!("Layer Dependencies: {}", layer_dependencies.len());
    if cli.list {
        display_graph_information(&nodes, &dependencies);
        display_graph_information(&layers, &layer_dependencies)
    }

    if let Some(format) = &cli.output {
        if let Some(html_path) = &cli.output_html {
            generate_html_output(&nodes, &dependencies, &layers, &layer_dependencies, html_path, format, &config.global.toggles)?;
        } else {
            match format.as_str() {
                "mermaid" => generate_mermaid_diagram(&nodes, &dependencies),
                "graphviz" => generate_graphviz_diagram(&nodes, &dependencies, &layers, &layer_dependencies, &config.global.toggles),
                "d3" => eprintln!("D3 output is only available for HTML output."),
                _ => eprintln!("Invalid format. Use 'mermaid' or 'graphviz'."),
            }
        }
    }

    if cli.detect_cycles {
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

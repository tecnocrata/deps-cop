use std::path::PathBuf;
use std::env;
use clap::{App, Arg, AppSettings};
use std::fs::File;
use std::io::Write;
use chrono::Local;

mod projects;
use projects::Node;
use projects::{ProjectDependencyManager, ProjectDependencies};


// Main entry point of the application
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Dependency Analyzer Cop")
        .version("0.1.42")
        .mut_arg("version", |a| a.short('v'))  // It shows the version with -v
        .author("tecnocrata <")
        .about("Analyzes dependencies from C# project files for now")
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
                "d3" => eprintln!("D3 output is only available for HTML output."),
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
    let now = Local::now();
    
    writeln!(file, "<!DOCTYPE html>")?;
    writeln!(file, "<html lang=\"en\">")?;
    writeln!(file, "<head>")?;
    writeln!(file, "    <meta charset=\"UTF-8\">")?;
    writeln!(file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
    writeln!(file, "    <title>Dependencies Analyzer</title>")?;
    writeln!(file, "    <link rel=\"stylesheet\" href=\"https://unpkg.com/tailwindcss@2.2.19/dist/tailwind.min.css\"/>")?;
    writeln!(file, "    <link href=\"https://fonts.googleapis.com/css?family=Source+Sans+Pro:400,700\" rel=\"stylesheet\">")?;
    writeln!(file, "<style>")?;
    writeln!(file, "    body {{ font-family: 'Source Sans Pro', sans-serif; color: #4a5568; margin: 0; display: flex; flex-direction: column; min-height: 100vh; }}")?;
    writeln!(file, "    .header {{ background-color: #667eea; color: #fafafa; padding: 20px; text-align: center; flex-shrink: 0; }}")?;
    writeln!(file, "    .content {{ flex: 1; display: flex; flex-direction: column; padding: 0; overflow: hidden; }}")?;
    writeln!(file, "    .footer {{ background-color: #718096; color: #ffffff; text-align: center; padding: 10px; flex-shrink: 0; }}")?;
    writeln!(file, "    .rust-logo {{ height: 50px; }}")?;
    writeln!(file, "</style>")?;
    generate_header_content(&mut file, format)?;
    writeln!(file, "</head>")?;
    writeln!(file, "<body>")?;
    writeln!(file, "    <div class=\"header\">")?;
    writeln!(file, "        <h1>Dependencies Analyzer</h1>")?;
    writeln!(file, "        <p>This page was generated automatically.</p>")?;
    writeln!(file, "    </div>")?;
    writeln!(file, "<div class=\"content\">")?;
    generate_body_content(&mut file, format, nodes, node_dependencies)?;
    writeln!(file, "        </div>")?;
    writeln!(file, "    <div class=\"footer\">")?;
    writeln!(file, "        <p>Generated on: {}</p>", now.format("%Y-%m-%dT%H:%M:%SZ"))?;
    writeln!(file, "        <p>Everything was generated using Rust.</p>")?;
    writeln!(file, "        <img src=\"https://www.rust-lang.org/logos/rust-logo-blk.svg\" alt=\"Rust Logo\" class=\"rust-logo mx-auto\">")?;
    writeln!(file, "    </div>")?;
    generate_script_code(&mut file, format, nodes, node_dependencies)?;
    writeln!(file, "</body>")?;
    writeln!(file, "</html>")?;
    
    Ok(())
}

fn generate_header_content(file: &mut File, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    if format == "graphviz" {
        writeln!(file, "    <style>")?;
        writeln!(file, "        #graph-container {{ flex: 1; display: flex; flex-direction: column; width: 100%; overflow: hidden; border: 1px solid #ccc; }}")?;
        writeln!(file, "        #graph {{ flex: 1; width: 100%; height: 100% display: flex; }}")?;
        // writeln!(file, "        #graph svg {{ width: 100%; height: 100%; }}")?;
        writeln!(file, "    </style>")?;
        writeln!(file, "<script src=\"https://cdnjs.cloudflare.com/ajax/libs/viz.js/2.1.2/viz.js\"></script>")?;
        writeln!(file, "<script src=\"https://cdnjs.cloudflare.com/ajax/libs/viz.js/2.1.2/full.render.js\" integrity=\"sha512-1zKK2bG3QY2JaUPpfHZDUMe3dwBwFdCDwXQ01GrKSd+/l0hqPbF+aak66zYPUZtn+o2JYi1mjXAqy5mW04v3iA==\" crossorigin=\"anonymous\" referrerpolicy=\"no-referrer\"></script>")?;
        writeln!(file, "<script src=\"https://cdn.jsdelivr.net/npm/svg-pan-zoom@3.6.1/dist/svg-pan-zoom.min.js\"></script>")?;
    } else if format == "d3" {
        generate_style_content_d3(file)?;
    }
    Ok(())
}

fn generate_style_content_d3(file: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "<style>")?;
    // Define SVG size and overflow behavior
    writeln!(file, "    svg {{ cursor: grab; width: 100%; height: auto; max-height: 80vh; overflow: auto; }}")?;
    // Ensure consistent node styling
    writeln!(file, "    .node circle {{ fill: steelblue; stroke: #fff; stroke-width: 1.5px; }}")?;
    writeln!(file, "    .node text {{ font-size: 10px; font-family: 'Source Sans Pro', sans-serif; pointer-events: none; }}")?;
    // Style for links including arrow markers
    writeln!(file, "    .link {{ fill: none; stroke: #999; stroke-opacity: 0.6; marker-end: url(#arrow); }}")?;
    writeln!(file, "</style>")?;
    Ok(())
}

fn generate_script_code(file: &mut File, format: &str, nodes: &[Node], node_dependencies: &[Vec<usize>]) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        "graphviz" => generate_script_code_graphviz(file, nodes, node_dependencies)?,
        "d3" => generate_script_code_d3(file, nodes, node_dependencies)?,
        _ => (),
    }
    Ok(())
}

fn generate_script_code_graphviz(file: &mut File, nodes: &[Node], node_dependencies: &[Vec<usize>]) -> Result<(), Box<dyn std::error::Error>> {
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
    writeln!(file, "                var graph = document.getElementById('graph');")?;
    writeln!(file, "                graph.appendChild(element);")?;
    writeln!(file, "                var svg = graph.querySelector('svg');")?;
    writeln!(file, "                svg.setAttribute('preserveAspectRatio', 'none');")?;
    writeln!(file, "                svg.style.width = '100%';")?;
    writeln!(file, "                svg.style.height = '100%';")?;
    writeln!(file, "                svgPanZoom(svg, {{")?;
    writeln!(file, "                    zoomEnabled: true,")?;
    writeln!(file, "                    controlIconsEnabled: true,")?;
    writeln!(file, "                    fit: true,")?;
    writeln!(file, "                    center: true,")?;
    writeln!(file, "                    minZoom: 0.5,")?;
    writeln!(file, "                    maxZoom: 10")?;
    writeln!(file, "                }});")?;
    writeln!(file, "                function resizeSvg() {{")?;
    writeln!(file, "                    var container = document.getElementById('graph-container');")?;
    writeln!(file, "                    svg.style.width = container.clientWidth + 'px';")?;
    writeln!(file, "                    svg.style.height = container.clientHeight + 'px';")?;
    writeln!(file, "                }}")?;
    writeln!(file, "                window.addEventListener('resize', resizeSvg);")?;
    writeln!(file, "                resizeSvg();")?;
    writeln!(file, "            }})")?;
    writeln!(file, "            .catch(error => {{")?;
    writeln!(file, "                console.error('Error rendering graph:', error);")?;
    writeln!(file, "            }});")?;
    writeln!(file, "</script>")?;
    Ok(())
}


fn generate_script_code_d3(file: &mut File, nodes: &[Node], node_dependencies: &[Vec<usize>]) -> Result<(), Box<dyn std::error::Error>>  {
    writeln!(file, "<script src=\"https://d3js.org/d3.v6.min.js\"></script>")?;
    writeln!(file, "<script>")?;
    writeln!(file, "    const svg = d3.select('svg'),")?;
    writeln!(file, "          width = +svg.attr('width'),")?;
    writeln!(file, "          height = +svg.attr('height');")?;
    writeln!(file, "    const g = svg.append('g').attr('transform', 'translate(480, 300)');")?;
    writeln!(file, "    svg.call(d3.zoom().on('zoom', (event) => {{")?;
    writeln!(file, "        g.attr('transform', event.transform);")?;
    writeln!(file, "    }}));")?;

    writeln!(file, "    svg.append('defs').append('marker')")?;
    writeln!(file, "        .attr('id', 'arrow')")?;
    writeln!(file, "        .attr('viewBox', '0 -5 10 10')")?;
    writeln!(file, "        .attr('refX', 25)")?;
    writeln!(file, "        .attr('refY', 0)")?;
    writeln!(file, "        .attr('markerWidth', 6)")?;
    writeln!(file, "        .attr('markerHeight', 6)")?;
    writeln!(file, "        .attr('orient', 'auto')")?;
    writeln!(file, "        .append('path')")?;
    writeln!(file, "        .attr('d', 'M0,-5L10,0L0,5')")?;
    writeln!(file, "        .attr('fill', '#999');")?;

    // Generate the nodes data dynamically
    writeln!(file, "    const nodes = [")?;
    for node in nodes {
        writeln!(file, "        {{ id: '{}', name: '{}' }},", node.id, node.name)?;
    }
    writeln!(file, "    ];")?;

    // Generate the links data dynamically
    writeln!(file, "    const links = [")?;
    for (index, dependencies) in node_dependencies.iter().enumerate() {
        for &target_index in dependencies {
            writeln!(file, "        {{ source: '{}', target: '{}' }},", nodes[index].id, nodes[target_index].id)?;
        }
    }
    writeln!(file, "    ];")?;

    writeln!(file, r#"
        // Calculate incoming links for node size
        nodes.forEach(node => {{
            node.incomingLinks = links.filter(link => link.target === node.id).length;
        }});

        // Simulation
        const simulation = d3.forceSimulation(nodes)
            .force("link", d3.forceLink(links).id(d => d.id).distance(200))
            .force("charge", d3.forceManyBody().strength(-500))
            .force("center", d3.forceCenter(width / 2, height / 2));

        // Draw links
        const link = g.selectAll(".link")
            .data(links)
            .join("line")
            .classed("link", true)
            .attr("stroke-width", 2);

        // Draw nodes
        const node = g.selectAll(".node")
            .data(nodes)
            .join("g")
            .classed("node", true)
            .call(d3.drag()
                .on("start", dragstarted)
                .on("drag", dragged)
                .on("end", dragended));

        node.append("circle")
            .attr("r", d => 5 + d.incomingLinks * 2);

        node.append("text")
            .attr("x", 8)
            .attr("y", "0.31em")
            .text(d => d.name);

        simulation.on("tick", () => {{
            link.attr("x1", d => d.source.x)
                .attr("y1", d => d.source.y)
                .attr("x2", d => d.target.x)
                .attr("y2", d => d.target.y);

            node.attr("transform", d => `translate(${{d.x}},${{d.y}})`);
        }});

        function dragstarted(event, d) {{
            if (!event.active) simulation.alphaTarget(0.3).restart();
            d.fx = d.x;
            d.fy = d.y;
        }}

        function dragged(event, d) {{
            d.fx = event.x;
            d.fy = event.y;
        }}

        function dragended(event, d) {{
            if (!event.active) simulation.alphaTarget(0);
            d.fx = null;
            d.fy = null;
        }}
    "#)?;
    writeln!(file, "</script>")?;

    Ok(())
}

fn generate_body_content(file: &mut File, format: &str, nodes: &[Node], node_dependencies: &[Vec<usize>]) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        "graphviz" => generate_body_content_graphviz(file, nodes, node_dependencies)?,
        "d3" => generate_body_content_d3(file)?,
        _ => (),
    }
    Ok(())
}
fn generate_body_content_graphviz(file: &mut File, _nodes: &[Node], _node_dependencies: &[Vec<usize>]) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "            <div id=\"graph-container\">")?;
    writeln!(file, "                <div id=\"graph\"></div>")?;
    writeln!(file, "            </div>")?;
    Ok(())
}

fn generate_body_content_d3(file: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "<svg width=\"960\" height=\"600\"></svg>")?;
    // writeln!(file, "<svg viewBox=\"0 0 960 600\"></svg>")?;
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
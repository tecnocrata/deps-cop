# depscop - Dependency Analyzer

`depscop` is a Rust-based command-line tool designed to analyze dependencies in software projects. While it primarily focuses on C# projects, analyzing both project references (\*.csproj) and namespace dependencies, it's designed to be extensible to other languages and dependency types.

## Features

- **Multiple Analysis Types:**
  - C# Project Dependencies (\*.csproj files)
  - C# Namespace Dependencies
  - (Future support planned for JavaScript folder dependencies)
- **Flexible Configuration:**
  - Layer-based architecture validation
  - Customizable color schemes
  - Configurable dependency rules
  - Pattern-based project/namespace recognition (regex or wildcard)
- **Visualization Options:**
  - Interactive D3.js graphs
  - Mermaid diagrams
  - Graphviz diagrams
  - HTML output with pan and zoom capabilities
- **Analysis Tools:**
  - Dependency cycle detection
  - Valid/invalid dependency highlighting
  - Layer rule validation
- **Cross-Platform:** Works on Windows, macOS, and Linux

## Installation from Source Code

To install `depscop`, you need Rust's package manager, Cargo. You can install Rust and Cargo via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Once Rust is installed, you can clone this repository and build the tool locally:

```bash
git clone https://github.com/tecnocrata/deps-cop
cd deps-cop
cargo build --release
```

The executable will be available in `./target/release/`.

## Usage

### Basic Commands

```bash
# Analyze C# project dependencies
./depscop --folder ./src --analysis csharp:projects --list

# Analyze C# namespace dependencies
./depscop --folder ./src --analysis csharp:namespaces --output graphviz

# Generate interactive visualization
./depscop --folder ./src --output d3 --output-html dependencies.html

# Check for circular dependencies
./depscop --folder ./src --detect-cycles

# Generate default configuration
./depscop --generate-config csharp,javascript
```

### Configuration File (depscoprc.json)

The tool uses a configuration file to customize its behavior. Generate a default one using:

```bash
./depscop --generate-config csharp
```

Example configuration:

```json
{
  "global": {
    "layers": ["core", "io", "usecase"],
    "colors": {
      "core": "#FBFDB8",
      "io": "#A7D7FD",
      "usecase": "#FEA29C"
    },
    "rules": {
      "core": ["core"],
      "io": ["core", "io", "usecase"],
      "usecase": ["core", "usecase"]
    },
    "toggles": {
      "show_valid_dependencies": true,
      "show_invalid_dependencies": true,
      "show_recognized_nodes": true,
      "show_unrecognized_nodes": true
    }
  },
  "csharp": {
    "pattern": "regex",
    "case_sensitive": true,
    "exclude": {
      "folders": ["bin", "obj"],
      "projects": [],
      "namespaces": [],
      "files": []
    },
    "projects": {
      "core": ".*\\.Entities.*\\.csproj$",
      "io": ".*\\.IO.*\\.csproj$",
      "usecase": ".*\\.UseCases.*\\.csproj$"
    },
    "namespaces": {
      "core": ".*\\.Entities(\\..*)?$",
      "io": ".*\\.IO(\\..*)?$",
      "usecase": ".*\\.UseCases(\\..*)?$"
    }
  }
}
```

### Advanced Examples

```bash
# Generate HTML visualization with Graphviz
./depscop --folder ./src --output graphviz --output-html deps.html

# Analyze namespaces and detect cycles
./depscop --folder ./src --analysis csharp:namespaces --detect-cycles

# List projects with detailed dependency information
./depscop --folder ./src --analysis csharp:projects --list

# Generate interactive D3 visualization
./depscop --folder ./src --output d3 --output-html interactive.html
```

### Options

- `--folder <PATH>`: Specifies the root directory to search for project files.
- `--list`: Lists all detected projects.
- `--output <FORMAT>`: Selects the output format (`d3`, `mermaid`, or `graphviz`) for the dependency graph.
- `--output-html <PATH>`: Generates an HTML file at the specified path containing the visualized dependency graph. Requires `--output`.
- `--detect-cycles`: Checks and reports if there are any circular dependencies.
- `--analysis <TYPE>`: Specifies the analysis type (default: `csharp:projects`). Options include `csharp:projects` and `csharp:namespaces`.
- `--generate-config <LANGUAGES>`: Generates the default configuration file for the specified languages (comma-separated, e.g., `csharp,javascript`).

## Contributing

Contributions are welcome! Please feel free to submit pull requests, create issues for bugs and feature requests, and provide feedback to improve the tool.

## License

`depscop` is distributed under the Apache 2.0 License. See `LICENSE` in the repository for more information.

## Contact

For more details, contact the maintainer at [Enrique](mailto:your.enrique@ortuno.net).


# depscop - Dependency Analyzer

`depscop` is a Rust-based command-line tool designed to analyze and report on the dependencies between C# project files (`*.csproj`). It offers insights into project structures, visualizes dependencies via diagrams, and detects circular dependencies to help maintain clean and manageable project architectures.

## Features

- **Project Listing:** Displays all detected C# project files.
- **Dependency Visualization:** Generates dependency graphs in Mermaid or Graphviz format.
- **HTML Output:** Generates HTML pages incorporating the visual dependency graphs.
- **Cycle Detection:** Identifies and reports circular dependencies among projects.
- **Cross-Platform:** Runs on Windows, macOS, and Linux.

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

- **Listing Projects:**
  ```bash
  ./depscop --folder path/to/solution --list
  ```
- **Generating a Dependency Graph:**
  ```bash
  ./depscop --folder path/to/solution --output mermaid
  ```
- **Creating HTML Output:**
  ```bash
  ./depscop --folder path/to/solution --output graphviz --output-html path/to/output.html
  ```
- **Detecting Circular Dependencies:**
  ```bash
  ./depscop --folder path/to/solution --detect-cycles
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

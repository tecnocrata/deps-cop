# deps_cop - Project Dependency Analyzer

`deps_cop` is a Rust-based command-line tool designed to analyze and report on the dependencies between C# project files (`*.csproj`). It offers insights into project structures, visualizes dependencies via diagrams, and detects circular dependencies to help maintain clean and manageable project architectures.

## Features

- **Project Listing:** Displays all detected C# project files.
- **Dependency Visualization:** Generates dependency graphs in Mermaid or Graphviz format.
- **HTML Output:** Generates HTML pages incorporating the visual dependency graphs.
- **Cycle Detection:** Identifies and reports circular dependencies among projects.
- **Cross-Platform:** Runs on Windows, macOS, and Linux.

## Installation

To install `deps_cop`, you need Rust's package manager, Cargo. You can install Rust and Cargo via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Once Rust is installed, you can clone this repository and build the tool locally:

```bash
git clone https://github.com/yourgithub/deps_cop.git
cd deps_cop
cargo build --release
```

The executable will be available in `./target/release/`.

## Usage

### Basic Commands

- **Listing Projects:**
  ```bash
  ./deps_cop --folder path/to/solution --list
  ```
- **Generating a Dependency Graph:**
  ```bash
  ./deps_cop --folder path/to/solution --output mermaid
  ```
- **Creating HTML Output:**
  ```bash
  ./deps_cop --folder path/to/solution --output graphviz --output-html path/to/output.html
  ```
- **Detecting Circular Dependencies:**
  ```bash
  ./deps_cop --folder path/to/solution --detect-cycles
  ```

### Options

- `--folder <PATH>`: Specifies the root directory to search for project files. Defaults to the current directory if not provided.
- `--list`: Lists all detected projects.
- `--output <FORMAT>`: Selects the output format (`mermaid` or `graphviz`) for the dependency graph.
- `--output-html <PATH>`: Generates an HTML file at the specified path containing the visualized dependency graph. Requires `--output`.
- `--detect-cycles`: Checks and reports if there are any circular dependencies.

## Contributing

Contributions are welcome! Please feel free to submit pull requests, create issues for bugs and feature requests, and provide feedback to improve the tool.

## License

`deps_cop` is distributed under the MIT License. See `LICENSE` in the repository for more information.

## Contact

For more details, contact the maintainer at [Enrique](mailto:your.enrique@ortuno.net).

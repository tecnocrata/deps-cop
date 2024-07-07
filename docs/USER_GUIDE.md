
# depscop - User Guide

`depscop` is a command-line tool designed to analyze and report on the dependencies between C# project files (`*.csproj`). This guide will help you understand how to install, configure, and use `depscop` effectively.

## Table of Contents

1. [Installation](#installation)
2. [Configuration](#configuration)
3. [Usage](#usage)
    - [Basic Commands](#basic-commands)
    - [Options](#options)
4. [Examples](#examples)
5. [Troubleshooting](#troubleshooting)
6. [FAQ](#faq)
7. [Contact](#contact)

## Installation

### From Source Code

To install `depscop`, you need Rust's package manager, Cargo. You can install Rust and Cargo via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Once Rust is installed, you can clone the repository and build the tool locally:

```bash
git clone https://github.com/tecnocrata/deps-cop
cd deps-cop
cargo build --release
```

The executable will be available in `./target/release/`.

## Configuration

### Default Configuration File

`depscop` can generate a default configuration file for specific languages. To generate this file, run:

```bash
./depscop --folder path/to/solution --generate-config csharp
```

This will create a `depscoprc.json` file in the specified folder with default settings.

### Custom Configuration

You can customize the configuration file to suit your needs. Here is an example configuration:

```json
{
    "global": {
        "layers": ["Core", "Infrastructure", "Presentation"],
        "colors": {
            "Core": "blue",
            "Infrastructure": "green",
            "Presentation": "red"
        },
        "rules": {
            "Core": ["Infrastructure"],
            "Infrastructure": ["Presentation"]
        }
    }
}
```

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

- `--folder <PATH>`: Specifies the root directory to search for project files. Defaults to the current directory if not provided.
- `--list`: Lists all detected projects.
- `--output <FORMAT>`: Selects the output format (`d3`, `mermaid`, or `graphviz`) for the dependency graph.
- `--output-html <PATH>`: Generates an HTML file at the specified path containing the visualized dependency graph. Requires `--output`.
- `--detect-cycles`: Checks and reports if there are any circular dependencies.
- `--analysis <TYPE>`: Specifies the analysis type (default: `csharp:projects`). Options include `csharp:projects` and `csharp:namespaces`.
- `--generate-config <LANGUAGES>`: Generates the default configuration file for the specified languages (comma-separated, e.g., `csharp,javascript`).

## Examples

### Listing All Projects

To list all projects in a solution, use the following command:

```bash
./depscop --folder /path/to/solution --list
```

### Generating a Mermaid Dependency Graph

To generate a dependency graph in Mermaid format, use:

```bash
./depscop --folder /path/to/solution --output mermaid
```

### Creating an HTML Output with Graphviz

To generate an HTML page containing a Graphviz dependency graph, use:

```bash
./depscop --folder /path/to/solution --output graphviz --output-html /path/to/output.html
```

### Detecting Circular Dependencies

To detect and report circular dependencies, use:

```bash
./depscop --folder /path/to/solution --detect-cycles
```

## Troubleshooting

### Common Issues

- **No projects found:** Ensure the `--folder` path is correct and contains `.csproj` files.
- **Invalid output format:** Ensure you use a supported format (`d3`, `mermaid`, `graphviz`).
- **Configuration issues:** Ensure the `depscoprc.json` file is correctly formatted and located in the specified folder.

## FAQ

### What languages does depscop support?

Currently, `depscop` supports C# projects. Future versions may include support for other languages.

### How can I contribute to depscop?

See the [CONTRIBUTING.md](..\CONTRIBUTING.md) file for guidelines on how to contribute.

## Contact

For more details, contact the maintainer at [Enrique](mailto:your.enrique@ortuno.net).

Thank you for using `depscop`!

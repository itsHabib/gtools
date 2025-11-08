# gtools

**Graph algorithm toolkit: pathfinding, MST, connectivity analysis, and network simulation**

A collection of command-line utilities for graph analysis and optimization, written in Rust.

## 🔧 Tools

### [`gt-path`](apps/gt-path/) - Path Analysis & Simulation

Find shortest paths, check SLOs, and simulate network changes.

**Features:**
- Dijkstra's shortest path algorithm
- Bottleneck detection (slowest edge on path)
- SLO (Service Level Objective) checking
- What-if simulation (modify/drop edges)
- JSON and text output formats

**Example:**
```bash
gt-path path -g graph.json -f api -t db
gt-path slo -g graph.json -f api -t db --max-latency 100
gt-path simulate -g graph.json -f api -t db --override "auth:db:50"
```

[Full documentation →](apps/gt-path/README.md)

---

### [`gt-connect`](apps/gt-connect/) - Connectivity & MST Analysis

Compute minimum spanning trees and find critical components.

**Features:**
- Minimum Spanning Tree (Kruskal's algorithm)
- Bridge detection (critical edges)
- Articulation point detection (critical nodes)
- JSON and text output formats

**Example:**
```bash
gt-connect mst -g network.csv
gt-connect critical -g network.csv
gt-connect analyze -g network.csv  # MST + critical components
```

[Full documentation →](apps/gt-connect/README.md)

---

## 📦 Installation

### From Source

```bash
git clone https://github.com/itsHabib/gtools
cd gtools
cargo build --release
```

Binaries will be at:
- `target/release/gt-path`
- `target/release/gt-connect`

### Install Specific Tool

```bash
cargo install --path apps/gt-path
cargo install --path apps/gt-connect
```

## 🏗️ Architecture

This is a Cargo workspace with:

- **`apps/gt-path`** - Path analysis CLI (uses JSON input)
- **`apps/gt-connect`** - Connectivity analysis CLI (uses CSV input)
- **`crates/graphs`** - Shared graph library with core algorithms

Each tool is independent and can be built/installed separately.

## 📊 Input Formats

- **`gt-path`**: JSON format with named nodes and directed edges
- **`gt-connect`**: CSV format with integer node IDs and undirected edges

See individual tool READMEs for detailed format specifications.

## 🔄 Development

### Run Tests

```bash
cargo test --all
```

### Build Debug Version

```bash
cargo build
```

### Generate Documentation

```bash
cargo doc --open
```
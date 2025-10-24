# gt-connect

Graph connectivity and MST analysis tool for undirected graphs.

## Features

- **Minimum Spanning Tree (MST)**: Compute MST using Kruskal's algorithm
- **Bridge Detection**: Find critical edges whose removal disconnects the graph
- **Articulation Points**: Find critical nodes whose removal disconnects the graph
- **Multiple Output Formats**: Text (human-readable) and JSON (machine-readable)

## Installation

```bash
cargo build --release -p gt-connect
```

## Usage

### Minimum Spanning Tree

```bash
gt-connect mst -g graph.csv
gt-connect mst -g graph.csv --algo kruskal --format json
```

### Critical Components

```bash
gt-connect critical -g graph.csv
gt-connect critical -g graph.csv --format json
```

### Full Analysis

Run both MST and critical component analysis:

```bash
gt-connect analyze -g graph.csv
gt-connect analyze -g graph.csv --format json
```

## Input Format

CSV file with edges (undirected graph):

```csv
u,v,weight
0,1,1.0
1,2,2.0
2,0,3.0
```

- First row can be a header (will be auto-detected)
- Node IDs must be integers starting from 0
- Weights are floating-point numbers

## Output Formats

### Text (default)

Human-readable format:

```
Minimum Spanning Tree (kruskal)
  Total Weight: 3.00
  Edges: 2

Edges:
  0 -- 1 (weight: 1.00)
  1 -- 2 (weight: 2.00)
```

### JSON

Machine-readable format for scripting:

```json
{
  "mst": {
    "algorithm": "kruskal",
    "total_weight": 3.0,
    "num_edges": 2,
    "edges": [
      {"u": 0, "v": 1, "weight": 1.0},
      {"u": 1, "v": 2, "weight": 2.0}
    ]
  },
  "critical": {
    "num_bridges": 0,
    "num_articulation_points": 0,
    "bridges": [],
    "articulation_points": []
  }
}
```

## Examples

See `testdata/` directory for example graphs:
- `triangle.csv`: Simple 3-node cycle (no critical components)
- `network.csv`: Network with bridges and articulation points

## Algorithms

- **MST**: Kruskal's algorithm with Union-Find (DSU)
- **Bridges**: Tarjan's algorithm using DFS with low-link values
- **Articulation Points**: Tarjan's algorithm (variation for nodes)

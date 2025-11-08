# Graph Tools: Path - gt-path

**Graph path analysis and simulation tool**

`gt-path` is a command-line tool for analyzing network graphs and finding optimal paths between nodes.

> **Note:** This tool was previously [`gcheck`](https://github.com/itsHabib/gcheck) (now archived). It has been integrated here and renamed to `gt-path`.

## Features

- 🚀 **Fast shortest path finding** using Dijkstra's algorithm
- 🔍 **Bottleneck detection** - identify the slowest edge on any path
- 🧪 **Path simulation** - test "what-if" scenarios by modifying edge weights
- 📊 **Multiple output formats** - human-readable text or JSON for scripting
- ✅ **Graph validation** - catches invalid edges, self-loops, and missing nodes
- 🎯 **Exit codes** - proper error codes for CI/CD integration

## Installation

### From Source

```bash
git clone <repository-url>
cd gopt
cargo build --release
```

The binary will be at `target/release/gt-path`

## Usage

### Basic Path Analysis

Find the shortest path between two nodes:

```bash
gt-path path --graph graph.json --from api --to db
```

Output:
```
Shortest Path:
  Route: api → auth → db
  Total Cost: 8ms
  Bottleneck: api → auth (5ms)
```

### JSON Output

Get structured JSON output for scripting:

```bash
gt-path path --graph graph.json --from api --to db --format json
```

Output:
```json
{
  "from": "api",
  "to": "db",
  "path": ["api", "auth", "db"],
  "total_latency_ms": 8,
  "bottleneck": {
    "from": "api",
    "to": "auth",
    "latency_ms": 5
  }
}
```

### Short Flags

```bash
gt-path path -g graph.json -f api -t db
```

### SLO Checking

Check if a path meets a Service Level Objective (maximum latency):

```bash
gt-path slo --graph graph.json --from api --to db --max-latency 10
```

Output (when SLO is met):
```
SLO Check:
  Route: api → auth → db
  Actual Latency: 8ms
  Max Allowed: 10ms
  Status: ✓ PASS
  Bottleneck: api → auth (5ms)
```

Output (when SLO is violated):
```
SLO Check:
  Route: api → cache → db
  Actual Latency: 15ms
  Max Allowed: 10ms
  Status: ✗ FAIL
  Bottleneck: api → cache (12ms)
```

**Exit Codes:**
- Exit 0 if SLO is met
- Exit 3 if SLO is violated (path exists but too slow)
- Exit 2 if no path exists

### SLO Check with JSON

```bash
gt-path slo --graph graph.json --from api --to db --max-latency 10 --format json
```

Output:
```json
{
  "slo_met": true,
  "max_latency_ms": 10,
  "actual_latency_ms": 8,
  "path": {
    "from": "api",
    "to": "db",
    "path": ["api", "auth", "db"],
    "total_latency_ms": 8,
    "bottleneck": {
      "from": "api",
      "to": "auth",
      "latency_ms": 5
    }
  }
}
```

### Path Simulation

Simulate "what-if" scenarios by modifying edge weights or dropping edges:

```bash
# Override edge weight
gt-path simulate --graph graph.json --from api --to db --override "auth:db:100"
```

Output:
```
Simulation Results:

Original Path:
  Route: api → auth → db
  Latency: 8ms
  Bottleneck: api → auth (5ms)

Modified Path:
  Route: api → cache → db
  Latency: 9ms
  Bottleneck: api → cache (7ms)

Impact: +1ms (slower)
```

**Override edge weights:**
```bash
# Single override
gt-path simulate -g graph.json -f api -t db --override "auth:db:100"

# Multiple overrides (comma-separated)
gt-path simulate -g graph.json -f api -t db --override "auth:db:100,api:cache:50"
```

**Drop edges:**
```bash
# Drop a single edge
gt-path simulate -g graph.json -f api -t db --drop "auth:db"

# Drop multiple edges (comma-separated)
gt-path simulate -g graph.json -f api -t db --drop "auth:db,api:cache"
```

**Combine overrides and drops:**
```bash
gt-path simulate -g graph.json -f api -t db \
  --override "api:auth:10,cache:db:1" \
  --drop "auth:cache"
```

**JSON output for scripting:**
```bash
gt-path simulate -g graph.json -f api -t db --override "auth:db:100" --format json
```

Output:
```json
{
  "original": {
    "from": "api",
    "to": "db",
    "path": ["api", "auth", "db"],
    "total_latency_ms": 8,
    "bottleneck": {
      "from": "api",
      "to": "auth",
      "latency_ms": 5
    }
  },
  "modified": {
    "from": "api",
    "to": "db",
    "path": ["api", "cache", "db"],
    "total_latency_ms": 9,
    "bottleneck": {
      "from": "api",
      "to": "cache",
      "latency_ms": 7
    }
  },
  "latency_change_ms": 1
}
```

## Input Format

`gt-path` reads directed graphs in JSON format:

```json
{
  "nodes": ["api", "auth", "db", "cache"],
  "edges": [
    { "from": "api", "to": "auth", "latency_ms": 5.2 },
    { "from": "auth", "to": "db", "latency_ms": 3.1 },
    { "from": "api", "to": "cache", "latency_ms": 7.4 },
    { "from": "cache", "to": "db", "latency_ms": 2.3 }
  ]
}
```

### Field Descriptions

- `nodes` - Array of unique node names (strings)
- `edges` - Array of directed edges with:
  - `from` - Source node name
  - `to` - Destination node name  
  - `latency_ms` - Edge weight in milliseconds (float)

## Exit Codes

`gt-path` uses standard exit codes for automation:

- `0` - Success (path found, SLO met)
- `2` - No path exists between nodes
- `3` - SLO violated (path exists but exceeds max latency)
- `4` - Invalid input (bad file, invalid graph, missing node)

### Using Exit Codes in CI/CD

```bash
# Check if path meets SLO
gt-path slo -g graph.json -f api -t db --max-latency 100

case $? in
  0) echo "✓ SLO met" ;;
  2) echo "✗ No path - CRITICAL" ; exit 1 ;;
  3) echo "⚠ SLO violated - WARNING" ;;
  4) echo "✗ Invalid input" ; exit 1 ;;
esac
```

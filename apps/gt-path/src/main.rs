mod error;
mod graph;
mod io;
mod path;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::process;

/// Graph path analyzer - find shortest paths and bottlenecks in network graphs
#[derive(Parser)]
#[command(name = "gt-path")]
#[command(about = "Graph path analysis and simulation tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Find shortest path between two nodes
    Path {
        /// Path to graph JSON file
        #[arg(short, long)]
        graph: String,

        /// Source node name
        #[arg(short, long)]
        from: String,

        /// Destination node name
        #[arg(short, long)]
        to: String,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Check if path meets SLO (Service Level Objective)
    Slo {
        /// Path to graph JSON file
        #[arg(short, long)]
        graph: String,

        /// Source node name
        #[arg(short, long)]
        from: String,

        /// Destination node name
        #[arg(short, long)]
        to: String,

        /// Maximum allowed latency in milliseconds
        #[arg(short, long)]
        max_latency: u32,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Simulate path changes with modified edge weights
    Simulate {
        /// Path to graph JSON file
        #[arg(short, long)]
        graph: String,

        /// Source node name
        #[arg(short, long)]
        from: String,

        /// Destination node name
        #[arg(short, long)]
        to: String,

        /// Override edge weights: from:to:weight (e.g., "api:auth:100")
        #[arg(long = "override", value_delimiter = ',')]
        overrides: Vec<String>,

        /// Drop edges: from:to (e.g., "api:cache")
        #[arg(long, value_delimiter = ',')]
        drop: Vec<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    /// Human-readable text output
    Text,
    /// JSON output for scripting
    Json,
}

// Exit codes from spec
const EXIT_SUCCESS: i32 = 0;
const EXIT_NO_PATH: i32 = 2;
const EXIT_SLO_VIOLATED: i32 = 3;
const EXIT_INVALID_INPUT: i32 = 4;

fn main() {
    let cli = Cli::parse();

    let (result, exit_code) = match cli.command {
        Commands::Path {
            graph,
            from,
            to,
            format,
        } => (run_path(&graph, &from, &to, format), EXIT_SUCCESS),
        Commands::Slo {
            graph,
            from,
            to,
            max_latency,
            format,
        } => run_check_slo(&graph, &from, &to, max_latency, format),
        Commands::Simulate {
            graph,
            from,
            to,
            overrides,
            drop,
            format,
        } => (
            run_simulate(&graph, &from, &to, &overrides, &drop, format),
            EXIT_SUCCESS,
        ),
    };

    match result {
        Ok(()) => process::exit(exit_code),
        Err(e) => {
            eprintln!("Error: {:#}", e);

            let exit_code =
                if e.to_string().contains("No path") || e.to_string().contains("PathNotFound") {
                    EXIT_NO_PATH
                } else {
                    EXIT_INVALID_INPUT
                };

            process::exit(exit_code);
        }
    }
}

fn run_path(graph_file: &str, from: &str, to: &str, format: OutputFormat) -> Result<()> {
    let graph = graph::Graph::load_json(graph_file)
        .context(format!("Failed to load graph from {}", graph_file))?;

    let path = graph
        .shortest_path(from, to)
        .context(format!("Failed to find path from {} to {}", from, to))?;

    match format {
        OutputFormat::Text => print_text(&graph, &path),
        OutputFormat::Json => print_json(&graph, &path)?,
    }

    Ok(())
}

fn print_text(graph: &graph::Graph, path: &path::Path) {
    println!("Shortest Path:");
    println!("  Route: {}", graph.format_path(path));
    println!("  Total Cost: {}ms", path.cost);

    if let Some(bottleneck) = &path.bottleneck {
        let from_name = &graph.to_name[bottleneck.from.0 as usize];
        let to_name = &graph.to_name[bottleneck.to.0 as usize];
        println!(
            "  Bottleneck: {} → {} ({}ms)",
            from_name, to_name, bottleneck.latency_ms
        );
    }
}

fn print_json(graph: &graph::Graph, path: &path::Path) -> Result<()> {
    let output = graph.path_output(path);
    let json =
        serde_json::to_string_pretty(&output).context("Failed to serialize output to JSON")?;
    println!("{}", json);
    Ok(())
}

fn run_check_slo(
    graph_file: &str,
    from: &str,
    to: &str,
    max_latency: u32,
    format: OutputFormat,
) -> (Result<()>, i32) {
    let graph = match graph::Graph::load_json(graph_file)
        .context(format!("Failed to load graph from {}", graph_file))
    {
        Ok(g) => g,
        Err(e) => return (Err(e), EXIT_INVALID_INPUT),
    };

    let path = match graph
        .shortest_path(from, to)
        .context(format!("Failed to find path from {} to {}", from, to))
    {
        Ok(p) => p,
        Err(e) => return (Err(e), EXIT_NO_PATH),
    };

    let slo_met = path.cost <= max_latency;
    let exit_code = if slo_met {
        EXIT_SUCCESS
    } else {
        EXIT_SLO_VIOLATED
    };

    let result = match format {
        OutputFormat::Text => {
            print_slo_text(&graph, &path, max_latency, slo_met);
            Ok(())
        }
        OutputFormat::Json => print_slo_json(&graph, &path, max_latency, slo_met),
    };

    (result, exit_code)
}

fn print_slo_text(graph: &graph::Graph, path: &path::Path, max_latency: u32, slo_met: bool) {
    println!("SLO Check:");
    println!("  Route: {}", graph.format_path(path));
    println!("  Actual Latency: {}ms", path.cost);
    println!("  Max Allowed: {}ms", max_latency);
    println!("  Status: {}", if slo_met { "✓ PASS" } else { "✗ FAIL" });

    if let Some(bottleneck) = &path.bottleneck {
        let from_name = &graph.to_name[bottleneck.from.0 as usize];
        let to_name = &graph.to_name[bottleneck.to.0 as usize];
        println!(
            "  Bottleneck: {} → {} ({}ms)",
            from_name, to_name, bottleneck.latency_ms
        );
    }
}

fn print_slo_json(
    graph: &graph::Graph,
    path: &path::Path,
    max_latency: u32,
    slo_met: bool,
) -> Result<()> {
    use serde_json::json;

    let path_output = graph.path_output(path);
    let output = json!({
        "slo_met": slo_met,
        "max_latency_ms": max_latency,
        "actual_latency_ms": path.cost,
        "path": path_output,
    });

    let json =
        serde_json::to_string_pretty(&output).context("Failed to serialize output to JSON")?;
    println!("{}", json);
    Ok(())
}

fn run_simulate(
    graph_file: &str,
    from: &str,
    to: &str,
    overrides_raw: &[String],
    drop_raw: &[String],
    format: OutputFormat,
) -> Result<()> {
    let mut overrides = Vec::new();
    for override_str in overrides_raw {
        let parts: Vec<&str> = override_str.split(':').collect();
        if parts.len() != 3 {
            anyhow::bail!(
                "Invalid override format '{}'. Expected 'from:to:weight'",
                override_str
            );
        }
        let weight = parts[2].parse::<u32>().context(format!(
            "Invalid weight '{}' in override '{}'",
            parts[2], override_str
        ))?;
        overrides.push((parts[0].to_string(), parts[1].to_string(), weight));
    }

    let mut drops = Vec::new();
    for drop_str in drop_raw {
        let parts: Vec<&str> = drop_str.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid drop format '{}'. Expected 'from:to'", drop_str);
        }
        drops.push((parts[0].to_string(), parts[1].to_string()));
    }

    let graph = graph::Graph::load_json(graph_file)
        .context(format!("Failed to load graph from {}", graph_file))?;

    let original_path = graph
        .shortest_path(from, to)
        .context(format!("Failed to find path from {} to {}", from, to))?;

    let modified_graph = graph
        .with_modifications(&overrides, &drops)
        .context("Failed to apply modifications to graph")?;

    let new_path = modified_graph.shortest_path(from, to).context(format!(
        "Failed to find path from {} to {} in modified graph",
        from, to
    ))?;

    match format {
        OutputFormat::Text => {
            print_simulate_text(&graph, &modified_graph, &original_path, &new_path)
        }
        OutputFormat::Json => {
            print_simulate_json(&graph, &modified_graph, &original_path, &new_path)?
        }
    }

    Ok(())
}

fn print_simulate_text(
    original_graph: &graph::Graph,
    modified_graph: &graph::Graph,
    original_path: &path::Path,
    new_path: &path::Path,
) {
    println!("Simulation Results:");
    println!();
    println!("Original Path:");
    println!("  Route: {}", original_graph.format_path(original_path));
    println!("  Latency: {}ms", original_path.cost);

    if let Some(bottleneck) = &original_path.bottleneck {
        let from_name = &original_graph.to_name[bottleneck.from.0 as usize];
        let to_name = &original_graph.to_name[bottleneck.to.0 as usize];
        println!(
            "  Bottleneck: {} → {} ({}ms)",
            from_name, to_name, bottleneck.latency_ms
        );
    }

    println!();
    println!("Modified Path:");
    println!("  Route: {}", modified_graph.format_path(new_path));
    println!("  Latency: {}ms", new_path.cost);

    if let Some(bottleneck) = &new_path.bottleneck {
        let from_name = &modified_graph.to_name[bottleneck.from.0 as usize];
        let to_name = &modified_graph.to_name[bottleneck.to.0 as usize];
        println!(
            "  Bottleneck: {} → {} ({}ms)",
            from_name, to_name, bottleneck.latency_ms
        );
    }

    println!();
    let diff = new_path.cost as i64 - original_path.cost as i64;
    let change = if diff > 0 {
        format!("+{}ms (slower)", diff)
    } else if diff < 0 {
        format!("{}ms (faster)", diff)
    } else {
        "no change".to_string()
    };
    println!("Impact: {}", change);
}

fn print_simulate_json(
    original_graph: &graph::Graph,
    modified_graph: &graph::Graph,
    original_path: &path::Path,
    new_path: &path::Path,
) -> Result<()> {
    use serde_json::json;

    let original_output = original_graph.path_output(original_path);
    let new_output = modified_graph.path_output(new_path);

    let output = json!({
        "original": original_output,
        "modified": new_output,
        "latency_change_ms": new_path.cost as i64 - original_path.cost as i64,
    });

    let json =
        serde_json::to_string_pretty(&output).context("Failed to serialize output to JSON")?;
    println!("{}", json);
    Ok(())
}

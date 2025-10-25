use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use graphs::io::load_csv;
use graphs::mst::kruskal;
use serde::Serialize;
use std::process;

#[derive(Parser)]
#[command(name = "gt-connect")]
#[command(about = "Graph connectivity and MST analysis tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compute minimum spanning tree
    Mst {
        /// Path to graph CSV file (format: u,v,weight)
        #[arg(short, long)]
        graph: String,

        /// Algorithm to use
        #[arg(long, value_enum, default_value = "kruskal")]
        algo: MstAlgorithm,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Find critical components (bridges and articulation points)
    Critical {
        /// Path to graph CSV file (format: u,v,weight)
        #[arg(short, long)]
        graph: String,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Full connectivity analysis (MST + critical components)
    Analyze {
        /// Path to graph CSV file (format: u,v,weight)
        #[arg(short, long)]
        graph: String,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
}

#[derive(Clone, ValueEnum)]
enum MstAlgorithm {
    Kruskal,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Serialize)]
struct MstOutput {
    algorithm: String,
    total_weight: f32,
    num_edges: usize,
    edges: Vec<EdgeOutput>,
}

#[derive(Serialize)]
struct EdgeOutput {
    u: u32,
    v: u32,
    weight: f32,
}

#[derive(Serialize)]
struct CriticalOutput {
    num_bridges: usize,
    num_articulation_points: usize,
    bridges: Vec<(u32, u32)>,
    articulation_points: Vec<u32>,
}

#[derive(Serialize)]
struct AnalysisOutput {
    mst: MstOutput,
    critical: CriticalOutput,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Mst {
            graph,
            algo,
            format,
        } => run_mst(&graph, algo, format),
        Commands::Critical { graph, format } => run_critical(&graph, format),
        Commands::Analyze { graph, format } => run_analyze(&graph, format),
    };

    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        process::exit(1);
    }
}

fn run_mst(graph_file: &str, algo: MstAlgorithm, format: OutputFormat) -> Result<()> {
    let graph = load_csv(graph_file).context("Failed to load graph")?;

    let mst = match algo {
        MstAlgorithm::Kruskal => kruskal(&graph),
    };

    let output = MstOutput {
        algorithm: "kruskal".to_string(),
        total_weight: mst.total_weight,
        num_edges: mst.edges.len(),
        edges: mst
            .edges
            .iter()
            .map(|e| EdgeOutput {
                u: e.u.0,
                v: e.v.0,
                weight: e.weight,
            })
            .collect(),
    };

    match format {
        OutputFormat::Text => print_mst_text(&output),
        OutputFormat::Json => print_json(&output)?,
    }

    Ok(())
}

fn run_critical(graph_file: &str, format: OutputFormat) -> Result<()> {
    let graph = load_csv(graph_file).context("Failed to load graph")?;

    let (articulation_points, bridges) = graph.critical_components();

    let output = CriticalOutput {
        num_bridges: bridges.len(),
        num_articulation_points: articulation_points.len(),
        bridges: bridges.iter().map(|(u, v)| (u.0, v.0)).collect(),
        articulation_points: articulation_points.iter().map(|n| n.0).collect(),
    };

    match format {
        OutputFormat::Text => print_critical_text(&output),
        OutputFormat::Json => print_json(&output)?,
    }

    Ok(())
}

fn run_analyze(graph_file: &str, format: OutputFormat) -> Result<()> {
    let graph = load_csv(graph_file).context("Failed to load graph")?;

    let mst = kruskal(&graph);
    let (articulation_points, bridges) = graph.critical_components();

    let mst_output = MstOutput {
        algorithm: "kruskal".to_string(),
        total_weight: mst.total_weight,
        num_edges: mst.edges.len(),
        edges: mst
            .edges
            .iter()
            .map(|e| EdgeOutput {
                u: e.u.0,
                v: e.v.0,
                weight: e.weight,
            })
            .collect(),
    };

    let critical_output = CriticalOutput {
        num_bridges: bridges.len(),
        num_articulation_points: articulation_points.len(),
        bridges: bridges.iter().map(|(u, v)| (u.0, v.0)).collect(),
        articulation_points: articulation_points.iter().map(|n| n.0).collect(),
    };

    let output = AnalysisOutput {
        mst: mst_output,
        critical: critical_output,
    };

    match format {
        OutputFormat::Text => print_analysis_text(&output),
        OutputFormat::Json => print_json(&output)?,
    }

    Ok(())
}

fn print_mst_text(output: &MstOutput) {
    println!("Minimum Spanning Tree ({})", output.algorithm);
    println!("  Total Weight: {:.2}", output.total_weight);
    println!("  Edges: {}", output.num_edges);
    println!("\nEdges:");
    for edge in &output.edges {
        println!("  {} -- {} (weight: {:.2})", edge.u, edge.v, edge.weight);
    }
}

fn print_critical_text(output: &CriticalOutput) {
    println!("Critical Components Analysis");
    println!("  Bridges: {}", output.num_bridges);
    println!("  Articulation Points: {}", output.num_articulation_points);

    if !output.bridges.is_empty() {
        println!("\nBridges (critical edges):");
        for (u, v) in &output.bridges {
            println!("  {} -- {}", u, v);
        }
    }

    if !output.articulation_points.is_empty() {
        println!("\nArticulation Points (critical nodes):");
        for node in &output.articulation_points {
            println!("  {}", node);
        }
    }
}

fn print_analysis_text(output: &AnalysisOutput) {
    println!("=== Full Connectivity Analysis ===\n");
    print_mst_text(&output.mst);
    println!();
    print_critical_text(&output.critical);
}

fn print_json<T: Serialize>(output: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(output)?;
    println!("{}", json);
    Ok(())
}

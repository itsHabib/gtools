use crate::graph::{Edge, Graph, NodeId};
use csv::ReaderBuilder;
use std::fs::File;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during graph I/O operations.
#[derive(Error, Debug)]
pub enum IoError {
    #[error("Failed to read file: {0}")]
    FileError(#[from] std::io::Error),
    
    #[error("CSV parsing error: {0}")]
    CsvError(#[from] csv::Error),
    
    #[error("Invalid edge format: expected u,v,weight")]
    InvalidFormat,
    
    #[error("Invalid node ID: {0}")]
    InvalidNodeId(String),
    
    #[error("Invalid weight: {0}")]
    InvalidWeight(String),
}

/// Loads an undirected graph from a CSV file.
/// 
/// The CSV format expects three columns: u, v, weight where u and v are
/// node IDs (integers) and weight is a floating-point number. The file
/// may optionally have a header row (automatically detected).
/// 
/// Node IDs should be non-negative integers. The graph will be sized to
/// accommodate the maximum node ID found, so nodes don't need to be
/// contiguous (though this may waste memory for sparse graphs).
/// 
/// # Example CSV format
/// ```csv
/// u,v,weight
/// 0,1,1.5
/// 1,2,2.0
/// 2,0,1.0
/// ```
pub fn load_csv<P: AsRef<Path>>(path: P) -> Result<Graph, IoError> {
    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);
    
    let mut edges = Vec::new();
    let mut max_node = 0u32;
    
    for result in reader.records() {
        let record = result?;
        
        if record.len() < 3 {
            return Err(IoError::InvalidFormat);
        }
        
        // Skip header if first row looks like column names
        if record.get(0).unwrap_or("").to_lowercase() == "u" 
            || record.get(0).unwrap_or("").to_lowercase() == "from"
            || record.get(0).unwrap_or("").to_lowercase() == "source" {
            continue;
        }
        
        let u: u32 = record.get(0)
            .ok_or(IoError::InvalidFormat)?
            .trim()
            .parse()
            .map_err(|_| IoError::InvalidNodeId(record.get(0).unwrap().to_string()))?;
            
        let v: u32 = record.get(1)
            .ok_or(IoError::InvalidFormat)?
            .trim()
            .parse()
            .map_err(|_| IoError::InvalidNodeId(record.get(1).unwrap().to_string()))?;
            
        let weight: f32 = record.get(2)
            .ok_or(IoError::InvalidFormat)?
            .trim()
            .parse()
            .map_err(|_| IoError::InvalidWeight(record.get(2).unwrap().to_string()))?;
        
        max_node = max_node.max(u).max(v);
        edges.push((u, v, weight));
    }
    
    let num_nodes = (max_node + 1) as usize;
    let mut graph = Graph::new(num_nodes);
    
    for (u, v, weight) in edges {
        graph.add_edge(Edge {
            u: NodeId(u),
            v: NodeId(v),
            weight,
        });
    }
    
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_load_simple_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "0,1,1.0").unwrap();
        writeln!(file, "1,2,2.0").unwrap();
        writeln!(file, "2,0,3.0").unwrap();
        
        let graph = load_csv(file.path()).unwrap();
        assert_eq!(graph.size(), 3);
        assert_eq!(graph.edges().len(), 3);
    }
    
    #[test]
    fn test_load_with_header() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "u,v,weight").unwrap();
        writeln!(file, "0,1,1.0").unwrap();
        writeln!(file, "1,2,2.0").unwrap();
        
        let graph = load_csv(file.path()).unwrap();
        assert_eq!(graph.size(), 3);
        assert_eq!(graph.edges().len(), 2);
    }
}


//! Electrical Rule Checking (ERC) and topology validation command.

use crate::error::CliError;
use phonon_netlist::{elaborate_netlist, parse_netlist};
use std::path::Path;

/// Summary report of circuit topological and electrical rule validation.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationReport {
    pub title: String,
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub total_branches: usize,
    pub total_components: usize,
    pub is_valid: bool,
}

/// Executes topological Electrical Rule Checking (ERC) on a netlist file.
pub fn execute_validate(netlist_path: &Path) -> Result<ValidationReport, CliError> {
    let content = std::fs::read_to_string(netlist_path)?;
    let parsed = parse_netlist(&content)?;
    let elaborated = elaborate_netlist(&parsed)?;

    // Validate circuit graph topology
    let is_valid = match elaborated.graph.validate_topology() {
        Ok(()) => true,
        Err(e) => return Err(CliError::Core(e)),
    };

    Ok(ValidationReport {
        title: elaborated.title,
        total_nodes: elaborated.graph.total_nodes(),
        active_nodes: elaborated.graph.active_nodes(),
        total_branches: elaborated.graph.total_branches(),
        total_components: elaborated.graph.num_components(),
        is_valid,
    })
}

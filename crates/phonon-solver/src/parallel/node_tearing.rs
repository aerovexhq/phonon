//! Circuit partitioning and boundary tearing for Kron's Diakoptics.
//!
//! Decomposes large-scale circuit networks into disjoint subcircuits
//! interconnected through a minimal set of torn boundary nodes.

use phonon_core::{CircuitGraph, ComponentRecord, NodeId};
use std::collections::{HashSet, VecDeque};

/// A complete partitioned circuit network ready for parallel Diakoptics solution.
#[derive(Debug, Clone)]
pub struct PartitionedCircuit {
    /// The unified circuit graph.
    pub graph: CircuitGraph,
    /// Torn boundary nodes connecting the subcircuits.
    pub torn_nodes: Vec<NodeId>,
    /// MNA variable indices for each subcircuit partition.
    pub subcircuits: Vec<Vec<usize>>,
    /// MNA variable indices for the torn boundary variables.
    pub torn_indices: Vec<usize>,
}

impl PartitionedCircuit {
    /// Creates a partitioned circuit from an explicit partitioning of MNA variable indices.
    pub fn new(graph: CircuitGraph, torn_nodes: Vec<NodeId>, subcircuits: Vec<Vec<usize>>) -> Self {
        let active_nodes = graph.active_nodes();
        let mut torn_indices = Vec::new();
        for &tn in &torn_nodes {
            if !tn.is_ground() && tn.index() <= active_nodes {
                torn_indices.push(tn.index() - 1);
            }
        }
        Self {
            graph,
            torn_nodes,
            subcircuits,
            torn_indices,
        }
    }

    /// Automatically partitions a circuit graph into connected subcircuits given a set of torn boundary nodes.
    ///
    /// Uses breadth-first search (BFS) on component topological interconnections,
    /// stopping traversal at torn boundary nodes.
    pub fn from_graph_and_torn_nodes(graph: CircuitGraph, torn_nodes: Vec<NodeId>) -> Self {
        let active_nodes = graph.active_nodes();
        let total_branches = graph.total_branches();
        let total_dim = active_nodes + total_branches;

        let torn_set: HashSet<NodeId> = torn_nodes.iter().copied().collect();
        let mut torn_indices = Vec::new();
        for &tn in &torn_nodes {
            if !tn.is_ground() && tn.index() <= active_nodes {
                torn_indices.push(tn.index() - 1);
            }
        }

        // Build adjacency list for non-ground nodes, excluding traversal through torn nodes
        let mut adj = vec![Vec::new(); active_nodes + 1];
        for comp in graph.components() {
            match comp {
                ComponentRecord::Resistor { pos, neg, .. }
                | ComponentRecord::Capacitor { pos, neg, .. }
                | ComponentRecord::Inductor { pos, neg, .. }
                | ComponentRecord::VoltageSource { pos, neg, .. }
                | ComponentRecord::CurrentSource { pos, neg, .. }
                | ComponentRecord::Diode { pos, neg, .. }
                | ComponentRecord::TcadDiode { pos, neg, .. } => {
                    let p = pos.index();
                    let n = neg.index();
                    if p > 0 && n > 0 && p <= active_nodes && n <= active_nodes {
                        // Don't traverse through torn nodes
                        if !torn_set.contains(pos) && !torn_set.contains(neg) {
                            adj[p].push(n);
                            adj[n].push(p);
                        }
                    }
                }
                _ => {}
            }
        }

        // Find connected components among non-torn active nodes
        let mut visited = vec![false; active_nodes + 1];
        let mut subcircuit_node_sets: Vec<HashSet<usize>> = Vec::new();

        for start_idx in 1..=active_nodes {
            let start_node = NodeId::new(start_idx as u32);
            if torn_set.contains(&start_node) || visited[start_idx] {
                continue;
            }

            let mut comp_nodes = HashSet::new();
            let mut queue = VecDeque::new();
            queue.push_back(start_idx);
            visited[start_idx] = true;

            while let Some(curr) = queue.pop_front() {
                comp_nodes.insert(curr);
                for &neighbor in &adj[curr] {
                    if !visited[neighbor] {
                        visited[neighbor] = true;
                        queue.push_back(neighbor);
                    }
                }
            }

            if !comp_nodes.is_empty() {
                subcircuit_node_sets.push(comp_nodes);
            }
        }

        // Map node indices to MNA variables (0-based: node.index() - 1)
        let mut subcircuits: Vec<Vec<usize>> = subcircuit_node_sets
            .iter()
            .map(|node_set| {
                let mut vars: Vec<usize> = node_set.iter().map(|&idx| idx - 1).collect();
                vars.sort_unstable();
                vars
            })
            .collect();

        // Assign auxiliary branch variables (voltage sources, inductors) to the matching subcircuit
        for comp in graph.components() {
            let (pos, branch) = match comp {
                ComponentRecord::VoltageSource { pos, branch, .. }
                | ComponentRecord::Inductor { pos, branch, .. } => (*pos, *branch),
                _ => continue,
            };

            let br_mna_idx = active_nodes + branch.index();
            if br_mna_idx >= total_dim {
                continue;
            }

            // Find which subcircuit contains pos
            let mut assigned = false;
            for (sub_idx, node_set) in subcircuit_node_sets.iter().enumerate() {
                if node_set.contains(&pos.index()) {
                    subcircuits[sub_idx].push(br_mna_idx);
                    assigned = true;
                    break;
                }
            }

            if !assigned {
                // If connected between torn nodes, assign to torn indices
                torn_indices.push(br_mna_idx);
            }
        }

        Self {
            graph,
            torn_nodes,
            subcircuits,
            torn_indices,
        }
    }

    #[inline]
    pub fn subcircuit_count(&self) -> usize {
        self.subcircuits.len()
    }

    #[inline]
    pub fn torn_variable_count(&self) -> usize {
        self.torn_indices.len()
    }
}

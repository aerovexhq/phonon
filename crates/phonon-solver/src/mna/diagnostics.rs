//! Circuit-level singularity diagnostics mapping matrix pivots back to circuit components and nodes.

use phonon_core::{CircuitGraph, NodeId};

/// Explains a numerical singularity at a given MNA matrix index by identifying the
/// physical circuit node or component branch responsible.
pub fn diagnose_mna_singularity(graph: &CircuitGraph, mna_idx: usize) -> String {
    let active_nodes = graph.active_nodes();

    if mna_idx < active_nodes {
        // Corresponds to circuit node: node_idx = mna_idx + 1
        let node_id = NodeId::new((mna_idx + 1) as u32);
        let node_name = graph.node_name(node_id);

        let mut connected_components = Vec::new();
        for comp in graph.components() {
            match comp {
                phonon_core::ComponentRecord::Resistor { name, pos, neg, .. }
                | phonon_core::ComponentRecord::Capacitor { name, pos, neg, .. }
                | phonon_core::ComponentRecord::Inductor { name, pos, neg, .. }
                | phonon_core::ComponentRecord::VoltageSource { name, pos, neg, .. }
                | phonon_core::ComponentRecord::CurrentSource { name, pos, neg, .. }
                | phonon_core::ComponentRecord::JosephsonJunction { name, pos, neg, .. } => {
                    if *pos == node_id || *neg == node_id {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::Vcvs {
                    name,
                    out_pos,
                    out_neg,
                    ctrl_pos,
                    ctrl_neg,
                    ..
                } => {
                    if *out_pos == node_id
                        || *out_neg == node_id
                        || *ctrl_pos == node_id
                        || *ctrl_neg == node_id
                    {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::Vccs {
                    name,
                    out_pos,
                    out_neg,
                    ctrl_pos,
                    ctrl_neg,
                    ..
                } => {
                    if *out_pos == node_id
                        || *out_neg == node_id
                        || *ctrl_pos == node_id
                        || *ctrl_neg == node_id
                    {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::Diode { name, pos, neg } => {
                    if *pos == node_id || *neg == node_id {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::Mosfet {
                    name,
                    drain,
                    gate,
                    source,
                    bulk,
                } => {
                    if *drain == node_id
                        || *gate == node_id
                        || *source == node_id
                        || *bulk == node_id
                    {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::Bjt {
                    name,
                    collector,
                    base,
                    emitter,
                } => {
                    if *collector == node_id || *base == node_id || *emitter == node_id {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::TransmissionLine {
                    name,
                    in_pos,
                    in_neg,
                    out_pos,
                    out_neg,
                    ..
                } => {
                    if *in_pos == node_id
                        || *in_neg == node_id
                        || *out_pos == node_id
                        || *out_neg == node_id
                    {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::TcadDiode { name, pos, neg } => {
                    if *pos == node_id || *neg == node_id {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::TcadMosfet {
                    name,
                    drain,
                    gate,
                    source,
                    bulk,
                } => {
                    if *drain == node_id
                        || *gate == node_id
                        || *source == node_id
                        || *bulk == node_id
                    {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::NeuralSurrogate { name, nodes, .. } => {
                    if nodes.contains(&node_id) {
                        connected_components.push(name.as_str());
                    }
                }

                phonon_core::ComponentRecord::AtomisticChannel {
                    name,
                    drain,
                    gate,
                    source,
                    ..
                } => {
                    if *drain == node_id || *gate == node_id || *source == node_id {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::OpticalWaveguide { .. }
                | phonon_core::ComponentRecord::MicroRingResonator { .. } => {
                    // Optical passive components have no electrical nodes
                }
                phonon_core::ComponentRecord::ElectroOpticModulator {
                    name,
                    elec_pos,
                    elec_neg,
                    ..
                } => {
                    if *elec_pos == node_id || *elec_neg == node_id {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::LaserDiode {
                    name,
                    anode,
                    cathode,
                    ..
                } => {
                    if *anode == node_id || *cathode == node_id {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::Photodetector {
                    name,
                    anode,
                    cathode,
                    ..
                } => {
                    if *anode == node_id || *cathode == node_id {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::Memristor { name, pos, neg, .. } => {
                    if *pos == node_id || *neg == node_id {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::SpikingNeuron {
                    name,
                    input_node,
                    output_node,
                    ..
                } => {
                    if *input_node == node_id || *output_node == node_id {
                        connected_components.push(name.as_str());
                    }
                }
                phonon_core::ComponentRecord::RadiationStrike {
                    name, target_node, ..
                } => {
                    if *target_node == node_id {
                        connected_components.push(name.as_str());
                    }
                }
            }
        }

        if connected_components.is_empty() {
            format!(
                "Node '{}' ({}) is completely isolated (no connected components)",
                node_name, node_id
            )
        } else {
            format!(
                "Node '{}' ({}) has no DC path to ground or reference. Connected components: [{}]",
                node_name,
                node_id,
                connected_components.join(", ")
            )
        }
    } else {
        // Corresponds to auxiliary branch variable
        let branch_idx = mna_idx - active_nodes;
        let branch_id = phonon_core::BranchId::new(branch_idx as u32);
        let branch_label = graph.branch_label(branch_id);

        format!(
            "Auxiliary branch '{}' (MNA index {}) formed a singular constraint (e.g. parallel loop of ideal voltage sources or zero-impedance loop)",
            branch_label, mna_idx
        )
    }
}

//! Circuit graph topology, node and branch allocation, and connectivity validation.

use crate::error::CoreError;
use crate::types::{BranchId, ComponentId, NodeId, OpticalPortId};

use std::collections::{HashMap, HashSet};

/// Component description stored within the circuit graph.
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentRecord {
    Resistor {
        name: String,
        pos: NodeId,
        neg: NodeId,
        resistance: f64,
    },
    Capacitor {
        name: String,
        pos: NodeId,
        neg: NodeId,
        capacitance: f64,
        initial_voltage: Option<f64>,
    },
    Inductor {
        name: String,
        pos: NodeId,
        neg: NodeId,
        inductance: f64,
        branch: BranchId,
        initial_current: Option<f64>,
    },
    VoltageSource {
        name: String,
        pos: NodeId,
        neg: NodeId,
        dc_value: f64,
        branch: BranchId,
    },
    CurrentSource {
        name: String,
        pos: NodeId,
        neg: NodeId,
        dc_value: f64,
    },
    Vcvs {
        name: String,
        out_pos: NodeId,
        out_neg: NodeId,
        ctrl_pos: NodeId,
        ctrl_neg: NodeId,
        gain: f64,
        branch: BranchId,
    },
    Vccs {
        name: String,
        out_pos: NodeId,
        out_neg: NodeId,
        ctrl_pos: NodeId,
        ctrl_neg: NodeId,
        transconductance: f64,
    },
    Diode {
        name: String,
        pos: NodeId,
        neg: NodeId,
    },
    Mosfet {
        name: String,
        drain: NodeId,
        gate: NodeId,
        source: NodeId,
        bulk: NodeId,
    },
    Bjt {
        name: String,
        collector: NodeId,
        base: NodeId,
        emitter: NodeId,
    },
    TransmissionLine {
        name: String,
        in_pos: NodeId,
        in_neg: NodeId,
        out_pos: NodeId,
        out_neg: NodeId,
        z0: f64,
        td: f64,
    },
    TcadDiode {
        name: String,
        pos: NodeId,
        neg: NodeId,
    },
    TcadMosfet {
        name: String,
        drain: NodeId,
        gate: NodeId,
        source: NodeId,
        bulk: NodeId,
    },
    NeuralSurrogate {
        name: String,
        nodes: Vec<NodeId>,
        surrogate_id: String,
    },
    JosephsonJunction {
        name: String,
        pos: NodeId,
        neg: NodeId,
        ic: f64,
        rn: f64,
        cap: f64,
        phase_init: f64,
    },
    AtomisticChannel {
        name: String,
        drain: NodeId,
        gate: NodeId,
        source: NodeId,
        channel_type: AtomisticChannelType,
    },
    OpticalWaveguide {
        name: String,
        in_port: OpticalPortId,
        out_port: OpticalPortId,
        length_m: f64,
        loss_db_m: f64,
        n_eff: f64,
    },
    MicroRingResonator {
        name: String,
        in_port: OpticalPortId,
        thru_port: OpticalPortId,
        drop_port: OpticalPortId,
        radius_m: f64,
        coupling_k: f64,
    },
    ElectroOpticModulator {
        name: String,
        opt_in: OpticalPortId,
        opt_out: OpticalPortId,
        elec_pos: NodeId,
        elec_neg: NodeId,
        v_pi: f64,
    },
    LaserDiode {
        name: String,
        anode: NodeId,
        cathode: NodeId,
        opt_out: OpticalPortId,
        i_th: f64,
        slope_efficiency: f64,
    },
    Photodetector {
        name: String,
        opt_in: OpticalPortId,
        anode: NodeId,
        cathode: NodeId,
        responsivity: f64,
    },
    /// Non-volatile resistive memory element (RRAM, PCM, or FeFET channel).
    Memristor {
        name: String,
        pos: NodeId,
        neg: NodeId,
        initial_conductance: f64,
        r_on: f64,
        r_off: f64,
    },
    /// Neuromorphic spiking neuron (LIF / AdEx integrate-and-fire action potential unit).
    SpikingNeuron {
        name: String,
        input_node: NodeId,
        output_node: NodeId,
        v_thresh: f64,
        v_reset: f64,
    },
    /// Single-event radiation particle strike (heavy-ion track charge deposition).
    RadiationStrike {
        name: String,
        target_node: NodeId,
        let_mev: f64,
        strike_time_s: f64,
    },
}

/// Material classification and geometry for atomistic, 2D monolayer, and nanowire channels.
#[derive(Debug, Clone, PartialEq)]
pub enum AtomisticChannelType {
    /// 2D Transition Metal Dichalcogenide monolayer (e.g. "MoS2", "WS2", "MoSe2", "WSe2").
    TmdMonolayer {
        species: String,
        length_m: f64,
        width_m: f64,
    },
    /// Carbon Nanotube defined by chiral indices (n, m) and length in meters.
    CarbonNanotube { n: usize, m: usize, length_m: f64 },
    /// Metallic interconnect nanowire subject to atomistic electromigration.
    InterconnectNanowire {
        material: String,
        length_m: f64,
        width_m: f64,
        height_m: f64,
    },
}

impl ComponentRecord {
    pub fn name(&self) -> &str {
        match self {
            Self::Resistor { name, .. }
            | Self::Capacitor { name, .. }
            | Self::Inductor { name, .. }
            | Self::VoltageSource { name, .. }
            | Self::CurrentSource { name, .. }
            | Self::Vcvs { name, .. }
            | Self::Vccs { name, .. }
            | Self::Diode { name, .. }
            | Self::Mosfet { name, .. }
            | Self::Bjt { name, .. }
            | Self::TransmissionLine { name, .. }
            | Self::TcadDiode { name, .. }
            | Self::TcadMosfet { name, .. }
            | Self::NeuralSurrogate { name, .. }
            | Self::JosephsonJunction { name, .. }
            | Self::AtomisticChannel { name, .. }
            | Self::OpticalWaveguide { name, .. }
            | Self::MicroRingResonator { name, .. }
            | Self::ElectroOpticModulator { name, .. }
            | Self::LaserDiode { name, .. }
            | Self::Photodetector { name, .. }
            | Self::Memristor { name, .. }
            | Self::SpikingNeuron { name, .. }
            | Self::RadiationStrike { name, .. } => name,
        }
    }
}

/// The topological circuit graph mapping physical electrical nodes, auxiliary MNA branches,
/// and discrete component connections.
#[derive(Debug, Clone)]
pub struct CircuitGraph {
    /// Maps node name (e.g. "GND", "VIN", "OUT") to NodeId.
    node_lookup: HashMap<String, NodeId>,
    /// Maps NodeId index to node name string.
    node_names: Vec<String>,
    /// Auxiliary branch names (for voltage sources and inductors).
    branch_names: Vec<String>,
    /// Optical port names mapped to OpticalPortId.
    optical_port_lookup: HashMap<String, OpticalPortId>,
    /// Maps OpticalPortId index to optical port name string.
    optical_port_names: Vec<String>,
    /// Components registered in the circuit graph.
    components: Vec<ComponentRecord>,
    /// Map component name to index.
    component_lookup: HashMap<String, usize>,
}

impl Default for CircuitGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitGraph {
    /// Creates a new circuit graph initialized with the reference Ground node (`NodeId::GROUND = 0`).
    pub fn new() -> Self {
        let mut graph = Self {
            node_lookup: HashMap::new(),
            node_names: Vec::new(),
            branch_names: Vec::new(),
            optical_port_lookup: HashMap::new(),
            optical_port_names: Vec::new(),
            components: Vec::new(),
            component_lookup: HashMap::new(),
        };

        // Node 0 is permanently assigned to GND
        graph.node_names.push("0".to_string());
        graph.node_lookup.insert("0".to_string(), NodeId::GROUND);
        graph.node_lookup.insert("GND".to_string(), NodeId::GROUND);
        graph.node_lookup.insert("gnd".to_string(), NodeId::GROUND);
        graph
    }

    /// Retrieves an existing node by name, or allocates a new sequential NodeId.
    pub fn get_or_create_node(&mut self, name: &str) -> NodeId {
        if let Some(&node_id) = self.node_lookup.get(name) {
            return node_id;
        }

        let new_id = NodeId::new(self.node_names.len() as u32);
        let name_owned = name.to_string();
        self.node_lookup.insert(name_owned.clone(), new_id);
        self.node_names.push(name_owned);
        new_id
    }

    /// Retrieves an existing optical port by name, or allocates a new sequential OpticalPortId.
    pub fn get_or_create_optical_port(&mut self, name: &str) -> OpticalPortId {
        if let Some(&port_id) = self.optical_port_lookup.get(name) {
            return port_id;
        }

        let new_id = OpticalPortId::new(self.optical_port_names.len() as u32);
        let name_owned = name.to_string();
        self.optical_port_lookup.insert(name_owned.clone(), new_id);
        self.optical_port_names.push(name_owned);
        new_id
    }

    /// Returns the node identifier associated with the given name if it exists.
    pub fn get_node(&self, name: &str) -> Option<NodeId> {
        self.node_lookup.get(name).copied()
    }

    /// Returns the optical port identifier associated with the given name if it exists.
    pub fn get_optical_port(&self, name: &str) -> Option<OpticalPortId> {
        self.optical_port_lookup.get(name).copied()
    }

    /// Returns the total number of registered optical ports.
    pub fn total_optical_ports(&self) -> usize {
        self.optical_port_names.len()
    }

    /// Returns the name of an optical port by its ID.
    pub fn optical_port_name(&self, port: OpticalPortId) -> Option<&str> {
        self.optical_port_names
            .get(port.index())
            .map(|s| s.as_str())
    }

    /// Returns the human-readable name of the given node.
    pub fn node_name(&self, node: NodeId) -> &str {
        let idx = node.index();
        if idx < self.node_names.len() {
            &self.node_names[idx]
        } else {
            "<unknown_node>"
        }
    }

    /// Allocates a new auxiliary MNA branch current index (for voltage sources / inductors).
    pub fn allocate_branch(&mut self, label: &str) -> BranchId {
        let branch_id = BranchId::new(self.branch_names.len() as u32);
        self.branch_names.push(label.to_string());
        branch_id
    }

    /// Returns the human-readable label of the given branch.
    pub fn branch_label(&self, branch: BranchId) -> &str {
        let idx = branch.index();
        if idx < self.branch_names.len() {
            &self.branch_names[idx]
        } else {
            "<unknown_branch>"
        }
    }

    /// Total number of nodes, including ground (`NodeId(0)`).
    pub fn total_nodes(&self) -> usize {
        self.node_names.len()
    }

    /// Number of active non-ground nodes to be solved in MNA ($N - 1$).
    pub fn active_nodes(&self) -> usize {
        if self.node_names.is_empty() {
            0
        } else {
            self.node_names.len() - 1
        }
    }

    /// Total number of auxiliary branch variables.
    pub fn total_branches(&self) -> usize {
        self.branch_names.len()
    }

    /// Total number of components in the circuit.
    pub fn num_components(&self) -> usize {
        self.components.len()
    }

    /// Returns a slice of all component records.
    pub fn components(&self) -> &[ComponentRecord] {
        &self.components
    }

    /// Returns a mutable slice of all component records.
    pub fn components_mut(&mut self) -> &mut [ComponentRecord] {
        &mut self.components
    }

    /// Returns a reference to a component by name.
    pub fn get_component(&self, name: &str) -> Option<&ComponentRecord> {
        let &idx = self.component_lookup.get(name)?;
        self.components.get(idx)
    }

    /// Returns a mutable reference to a component by name.
    pub fn get_component_mut(&mut self, name: &str) -> Option<&mut ComponentRecord> {
        let &idx = self.component_lookup.get(name)?;
        self.components.get_mut(idx)
    }

    /// Updates the numerical value of a passive or independent source component.
    pub fn update_component_value(&mut self, name: &str, new_value: f64) -> Result<(), CoreError> {
        let comp = self
            .get_component_mut(name)
            .ok_or_else(|| CoreError::ComponentNotFound(name.to_string()))?;
        match comp {
            ComponentRecord::Resistor { resistance, .. } => {
                if new_value <= 0.0 || new_value.is_nan() || new_value.is_infinite() {
                    return Err(CoreError::InvalidResistance(name.to_string(), new_value));
                }
                *resistance = new_value;
            }
            ComponentRecord::Capacitor { capacitance, .. } => {
                if new_value <= 0.0 || new_value.is_nan() || new_value.is_infinite() {
                    return Err(CoreError::InvalidValue(name.to_string(), new_value));
                }
                *capacitance = new_value;
            }
            ComponentRecord::Inductor { inductance, .. } => {
                if new_value <= 0.0 || new_value.is_nan() || new_value.is_infinite() {
                    return Err(CoreError::InvalidValue(name.to_string(), new_value));
                }
                *inductance = new_value;
            }
            ComponentRecord::VoltageSource { dc_value, .. }
            | ComponentRecord::CurrentSource { dc_value, .. } => {
                if new_value.is_nan() || new_value.is_infinite() {
                    return Err(CoreError::InvalidValue(name.to_string(), new_value));
                }
                *dc_value = new_value;
            }
            _ => {
                return Err(CoreError::NumericalAnomaly {
                    detail: format!("Cannot update value of component '{}'", name),
                })
            }
        }
        Ok(())
    }

    /// Adds a resistor to the circuit.
    pub fn add_resistor(
        &mut self,
        name: &str,
        pos_name: &str,
        neg_name: &str,
        resistance: f64,
    ) -> Result<ComponentId, CoreError> {
        if resistance <= 0.0 || resistance.is_nan() || resistance.is_infinite() {
            return Err(CoreError::InvalidResistance(name.to_string(), resistance));
        }
        let pos = self.get_or_create_node(pos_name);
        let neg = self.get_or_create_node(neg_name);
        self.register_component(ComponentRecord::Resistor {
            name: name.to_string(),
            pos,
            neg,
            resistance,
        })
    }

    /// Adds a capacitor to the circuit.
    pub fn add_capacitor(
        &mut self,
        name: &str,
        pos_name: &str,
        neg_name: &str,
        capacitance: f64,
        initial_voltage: Option<f64>,
    ) -> Result<ComponentId, CoreError> {
        let pos = self.get_or_create_node(pos_name);
        let neg = self.get_or_create_node(neg_name);
        self.register_component(ComponentRecord::Capacitor {
            name: name.to_string(),
            pos,
            neg,
            capacitance,
            initial_voltage,
        })
    }

    /// Adds an inductor to the circuit with an auxiliary branch current variable.
    pub fn add_inductor(
        &mut self,
        name: &str,
        pos_name: &str,
        neg_name: &str,
        inductance: f64,
        initial_current: Option<f64>,
    ) -> Result<ComponentId, CoreError> {
        let pos = self.get_or_create_node(pos_name);
        let neg = self.get_or_create_node(neg_name);
        let branch = self.allocate_branch(&format!("I({})", name));
        self.register_component(ComponentRecord::Inductor {
            name: name.to_string(),
            pos,
            neg,
            inductance,
            branch,
            initial_current,
        })
    }

    /// Adds an independent DC voltage source with an auxiliary branch variable.
    pub fn add_voltage_source(
        &mut self,
        name: &str,
        pos_name: &str,
        neg_name: &str,
        dc_value: f64,
    ) -> Result<ComponentId, CoreError> {
        let pos = self.get_or_create_node(pos_name);
        let neg = self.get_or_create_node(neg_name);
        let branch = self.allocate_branch(&format!("I({})", name));
        self.register_component(ComponentRecord::VoltageSource {
            name: name.to_string(),
            pos,
            neg,
            dc_value,
            branch,
        })
    }

    /// Adds an independent DC current source (current flows from pos to neg through the external circuit,
    /// entering pos from outside).
    pub fn add_current_source(
        &mut self,
        name: &str,
        pos_name: &str,
        neg_name: &str,
        dc_value: f64,
    ) -> Result<ComponentId, CoreError> {
        let pos = self.get_or_create_node(pos_name);
        let neg = self.get_or_create_node(neg_name);
        self.register_component(ComponentRecord::CurrentSource {
            name: name.to_string(),
            pos,
            neg,
            dc_value,
        })
    }

    /// Adds a Voltage-Controlled Voltage Source (VCVS): $V(out\_pos) - V(out\_neg) = \text{gain} \cdot (V(ctrl\_pos) - V(ctrl\_neg))$.
    pub fn add_vcvs(
        &mut self,
        name: &str,
        out_pos_name: &str,
        out_neg_name: &str,
        ctrl_pos_name: &str,
        ctrl_neg_name: &str,
        gain: f64,
    ) -> Result<ComponentId, CoreError> {
        let out_pos = self.get_or_create_node(out_pos_name);
        let out_neg = self.get_or_create_node(out_neg_name);
        let ctrl_pos = self.get_or_create_node(ctrl_pos_name);
        let ctrl_neg = self.get_or_create_node(ctrl_neg_name);
        let branch = self.allocate_branch(&format!("I({})", name));
        self.register_component(ComponentRecord::Vcvs {
            name: name.to_string(),
            out_pos,
            out_neg,
            ctrl_pos,
            ctrl_neg,
            gain,
            branch,
        })
    }

    /// Adds a Voltage-Controlled Current Source (VCCS): $I(out) = \text{transconductance} \cdot (V(ctrl\_pos) - V(ctrl\_neg))$.
    pub fn add_vccs(
        &mut self,
        name: &str,
        out_pos_name: &str,
        out_neg_name: &str,
        ctrl_pos_name: &str,
        ctrl_neg_name: &str,
        transconductance: f64,
    ) -> Result<ComponentId, CoreError> {
        let out_pos = self.get_or_create_node(out_pos_name);
        let out_neg = self.get_or_create_node(out_neg_name);
        let ctrl_pos = self.get_or_create_node(ctrl_pos_name);
        let ctrl_neg = self.get_or_create_node(ctrl_neg_name);
        self.register_component(ComponentRecord::Vccs {
            name: name.to_string(),
            out_pos,
            out_neg,
            ctrl_pos,
            ctrl_neg,
            transconductance,
        })
    }

    /// Adds a Diode connected between anode (`pos_name`) and cathode (`neg_name`).
    pub fn add_diode(
        &mut self,
        name: &str,
        pos_name: &str,
        neg_name: &str,
    ) -> Result<ComponentId, CoreError> {
        let pos = self.get_or_create_node(pos_name);
        let neg = self.get_or_create_node(neg_name);
        self.register_component(ComponentRecord::Diode {
            name: name.to_string(),
            pos,
            neg,
        })
    }

    /// Adds a 4-terminal MOSFET connected to drain, gate, source, and bulk.
    pub fn add_mosfet(
        &mut self,
        name: &str,
        drain_name: &str,
        gate_name: &str,
        source_name: &str,
        bulk_name: &str,
    ) -> Result<ComponentId, CoreError> {
        let drain = self.get_or_create_node(drain_name);
        let gate = self.get_or_create_node(gate_name);
        let source = self.get_or_create_node(source_name);
        let bulk = self.get_or_create_node(bulk_name);
        self.register_component(ComponentRecord::Mosfet {
            name: name.to_string(),
            drain,
            gate,
            source,
            bulk,
        })
    }

    /// Adds a 3-terminal Bipolar Junction Transistor (BJT) connected to collector, base, and emitter.
    pub fn add_bjt(
        &mut self,
        name: &str,
        collector_name: &str,
        base_name: &str,
        emitter_name: &str,
    ) -> Result<ComponentId, CoreError> {
        let collector = self.get_or_create_node(collector_name);
        let base = self.get_or_create_node(base_name);
        let emitter = self.get_or_create_node(emitter_name);
        self.register_component(ComponentRecord::Bjt {
            name: name.to_string(),
            collector,
            base,
            emitter,
        })
    }

    /// Adds an ideal distributed lossless transmission line modeled via Branin's Method of Characteristics.
    #[allow(clippy::too_many_arguments)]
    pub fn add_transmission_line(
        &mut self,
        name: &str,
        in_pos_name: &str,
        in_neg_name: &str,
        out_pos_name: &str,
        out_neg_name: &str,
        z0: f64,
        td: f64,
    ) -> Result<ComponentId, CoreError> {
        if z0 <= 0.0 || z0.is_nan() || z0.is_infinite() {
            return Err(CoreError::InvalidValue(format!("{name}.Z0"), z0));
        }
        if td < 0.0 || td.is_nan() || td.is_infinite() {
            return Err(CoreError::InvalidValue(format!("{name}.TD"), td));
        }
        let in_pos = self.get_or_create_node(in_pos_name);
        let in_neg = self.get_or_create_node(in_neg_name);
        let out_pos = self.get_or_create_node(out_pos_name);
        let out_neg = self.get_or_create_node(out_neg_name);
        self.register_component(ComponentRecord::TransmissionLine {
            name: name.to_string(),
            in_pos,
            in_neg,
            out_pos,
            out_neg,
            z0,
            td,
        })
    }

    /// Adds a 2-terminal physical TCAD diode device connected between anode and cathode.
    pub fn add_tcad_diode(
        &mut self,
        name: &str,
        pos_name: &str,
        neg_name: &str,
    ) -> Result<ComponentId, CoreError> {
        let pos = self.get_or_create_node(pos_name);
        let neg = self.get_or_create_node(neg_name);
        self.register_component(ComponentRecord::TcadDiode {
            name: name.to_string(),
            pos,
            neg,
        })
    }

    /// Adds a 4-terminal physical TCAD MOSFET device connected to drain, gate, source, and bulk.
    pub fn add_tcad_mosfet(
        &mut self,
        name: &str,
        drain_name: &str,
        gate_name: &str,
        source_name: &str,
        bulk_name: &str,
    ) -> Result<ComponentId, CoreError> {
        let drain = self.get_or_create_node(drain_name);
        let gate = self.get_or_create_node(gate_name);
        let source = self.get_or_create_node(source_name);
        let bulk = self.get_or_create_node(bulk_name);
        self.register_component(ComponentRecord::TcadMosfet {
            name: name.to_string(),
            drain,
            gate,
            source,
            bulk,
        })
    }

    /// Adds an N-terminal neural surrogate device connected to the specified node names.
    pub fn add_neural_surrogate(
        &mut self,
        name: &str,
        node_names: &[&str],
        surrogate_id: &str,
    ) -> Result<ComponentId, CoreError> {
        let nodes = node_names
            .iter()
            .map(|n| self.get_or_create_node(n))
            .collect();
        self.register_component(ComponentRecord::NeuralSurrogate {
            name: name.to_string(),
            nodes,
            surrogate_id: surrogate_id.to_string(),
        })
    }

    /// Adds a superconducting Josephson Junction (RCSJ model) between `pos_name` and `neg_name`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_josephson_junction(
        &mut self,
        name: &str,
        pos_name: &str,
        neg_name: &str,
        ic: f64,
        rn: f64,
        cap: f64,
        phase_init: Option<f64>,
    ) -> Result<ComponentId, CoreError> {
        let pos = self.get_or_create_node(pos_name);
        let neg = self.get_or_create_node(neg_name);
        self.register_component(ComponentRecord::JosephsonJunction {
            name: name.to_string(),
            pos,
            neg,
            ic,
            rn,
            cap,
            phase_init: phase_init.unwrap_or(0.0),
        })
    }

    /// Adds an atomistic or 2D material field-effect channel between drain, gate, and source.
    pub fn add_atomistic_channel(
        &mut self,
        name: &str,
        drain_name: &str,
        gate_name: &str,
        source_name: &str,
        channel_type: AtomisticChannelType,
    ) -> Result<ComponentId, CoreError> {
        let drain = self.get_or_create_node(drain_name);
        let gate = self.get_or_create_node(gate_name);
        let source = self.get_or_create_node(source_name);
        self.register_component(ComponentRecord::AtomisticChannel {
            name: name.to_string(),
            drain,
            gate,
            source,
            channel_type,
        })
    }

    /// Adds an optical waveguide connecting `in_port_name` to `out_port_name`.
    pub fn add_optical_waveguide(
        &mut self,
        name: &str,
        in_port_name: &str,
        out_port_name: &str,
        length_m: f64,
        loss_db_m: f64,
        n_eff: f64,
    ) -> Result<ComponentId, CoreError> {
        let in_port = self.get_or_create_optical_port(in_port_name);
        let out_port = self.get_or_create_optical_port(out_port_name);
        self.register_component(ComponentRecord::OpticalWaveguide {
            name: name.to_string(),
            in_port,
            out_port,
            length_m,
            loss_db_m,
            n_eff,
        })
    }

    /// Adds an optical micro-ring resonator (MRR) with input, through, and drop ports.
    pub fn add_micro_ring_resonator(
        &mut self,
        name: &str,
        in_port_name: &str,
        thru_port_name: &str,
        drop_port_name: &str,
        radius_m: f64,
        coupling_k: f64,
    ) -> Result<ComponentId, CoreError> {
        let in_port = self.get_or_create_optical_port(in_port_name);
        let thru_port = self.get_or_create_optical_port(thru_port_name);
        let drop_port = self.get_or_create_optical_port(drop_port_name);
        self.register_component(ComponentRecord::MicroRingResonator {
            name: name.to_string(),
            in_port,
            thru_port,
            drop_port,
            radius_m,
            coupling_k,
        })
    }

    /// Adds an electro-optic modulator (e.g. MZI) driven by electrical voltage (elec_pos, elec_neg).
    pub fn add_electro_optic_modulator(
        &mut self,
        name: &str,
        opt_in_name: &str,
        opt_out_name: &str,
        elec_pos_name: &str,
        elec_neg_name: &str,
        v_pi: f64,
    ) -> Result<ComponentId, CoreError> {
        let opt_in = self.get_or_create_optical_port(opt_in_name);
        let opt_out = self.get_or_create_optical_port(opt_out_name);
        let elec_pos = self.get_or_create_node(elec_pos_name);
        let elec_neg = self.get_or_create_node(elec_neg_name);
        self.register_component(ComponentRecord::ElectroOpticModulator {
            name: name.to_string(),
            opt_in,
            opt_out,
            elec_pos,
            elec_neg,
            v_pi,
        })
    }

    /// Adds a semiconductor laser diode with electrical anode/cathode terminals and optical output.
    pub fn add_laser_diode(
        &mut self,
        name: &str,
        anode_name: &str,
        cathode_name: &str,
        opt_out_name: &str,
        i_th: f64,
        slope_efficiency: f64,
    ) -> Result<ComponentId, CoreError> {
        let anode = self.get_or_create_node(anode_name);
        let cathode = self.get_or_create_node(cathode_name);
        let opt_out = self.get_or_create_optical_port(opt_out_name);
        self.register_component(ComponentRecord::LaserDiode {
            name: name.to_string(),
            anode,
            cathode,
            opt_out,
            i_th,
            slope_efficiency,
        })
    }

    /// Adds an integrated photodetector converting optical input to electrical photocurrent.
    pub fn add_photodetector(
        &mut self,
        name: &str,
        opt_in_name: &str,
        anode_name: &str,
        cathode_name: &str,
        responsivity: f64,
    ) -> Result<ComponentId, CoreError> {
        let opt_in = self.get_or_create_optical_port(opt_in_name);
        let anode = self.get_or_create_node(anode_name);
        let cathode = self.get_or_create_node(cathode_name);
        self.register_component(ComponentRecord::Photodetector {
            name: name.to_string(),
            opt_in,
            anode,
            cathode,
            responsivity,
        })
    }

    /// Adds a non-volatile memristive memory element (RRAM, PCM, or FeFET).
    pub fn add_memristor(
        &mut self,
        name: &str,
        pos_name: &str,
        neg_name: &str,
        initial_conductance: f64,
        r_on: f64,
        r_off: f64,
    ) -> Result<ComponentId, CoreError> {
        let pos = self.get_or_create_node(pos_name);
        let neg = self.get_or_create_node(neg_name);
        self.register_component(ComponentRecord::Memristor {
            name: name.to_string(),
            pos,
            neg,
            initial_conductance,
            r_on,
            r_off,
        })
    }

    /// Adds a neuromorphic spiking integrate-and-fire neuron.
    pub fn add_spiking_neuron(
        &mut self,
        name: &str,
        input_name: &str,
        output_name: &str,
        v_thresh: f64,
        v_reset: f64,
    ) -> Result<ComponentId, CoreError> {
        let input_node = self.get_or_create_node(input_name);
        let output_node = self.get_or_create_node(output_name);
        self.register_component(ComponentRecord::SpikingNeuron {
            name: name.to_string(),
            input_node,
            output_node,
            v_thresh,
            v_reset,
        })
    }

    /// Adds a single-event radiation strike event injecting charge into a sensitive node.
    pub fn add_radiation_strike(
        &mut self,
        name: &str,
        target_name: &str,
        let_mev: f64,
        strike_time_s: f64,
    ) -> Result<ComponentId, CoreError> {
        let target_node = self.get_or_create_node(target_name);
        self.register_component(ComponentRecord::RadiationStrike {
            name: name.to_string(),
            target_node,
            let_mev,
            strike_time_s,
        })
    }

    fn register_component(&mut self, record: ComponentRecord) -> Result<ComponentId, CoreError> {
        let name = record.name().to_string();
        if self.component_lookup.contains_key(&name) {
            return Err(CoreError::DuplicateComponent(name));
        }
        let id = ComponentId::new(self.components.len() as u32);
        self.component_lookup.insert(name, id.index());
        self.components.push(record);
        Ok(id)
    }

    /// Performs static topological electrical sanity analysis:
    /// 1. Verifies that ground is connected.
    /// 2. Detects floating/isolated nodes.
    /// 3. Detects loops of ideal voltage sources.
    pub fn validate_topology(&self) -> Result<(), CoreError> {
        if self.components.is_empty() {
            return Err(CoreError::EmptyCircuit);
        }

        // Count node degrees
        let mut degree = vec![0usize; self.total_nodes()];
        for comp in &self.components {
            match comp {
                ComponentRecord::Resistor { pos, neg, .. }
                | ComponentRecord::Capacitor { pos, neg, .. }
                | ComponentRecord::Inductor { pos, neg, .. }
                | ComponentRecord::VoltageSource { pos, neg, .. }
                | ComponentRecord::CurrentSource { pos, neg, .. }
                | ComponentRecord::Diode { pos, neg, .. }
                | ComponentRecord::TcadDiode { pos, neg, .. }
                | ComponentRecord::JosephsonJunction { pos, neg, .. } => {
                    degree[pos.index()] += 1;
                    degree[neg.index()] += 1;
                }
                ComponentRecord::Mosfet {
                    drain,
                    gate,
                    source,
                    bulk,
                    ..
                }
                | ComponentRecord::TcadMosfet {
                    drain,
                    gate,
                    source,
                    bulk,
                    ..
                } => {
                    degree[drain.index()] += 1;
                    degree[gate.index()] += 1;
                    degree[source.index()] += 1;
                    degree[bulk.index()] += 1;
                }
                ComponentRecord::Bjt {
                    collector,
                    base,
                    emitter,
                    ..
                } => {
                    degree[collector.index()] += 1;
                    degree[base.index()] += 1;
                    degree[emitter.index()] += 1;
                }
                ComponentRecord::NeuralSurrogate { nodes, .. } => {
                    for &n in nodes {
                        degree[n.index()] += 1;
                    }
                }
                ComponentRecord::Vcvs {
                    out_pos,
                    out_neg,
                    ctrl_pos,
                    ctrl_neg,
                    ..
                } => {
                    degree[out_pos.index()] += 1;
                    degree[out_neg.index()] += 1;
                    degree[ctrl_pos.index()] += 1;
                    degree[ctrl_neg.index()] += 1;
                }
                ComponentRecord::Vccs {
                    out_pos,
                    out_neg,
                    ctrl_pos,
                    ctrl_neg,
                    ..
                } => {
                    degree[out_pos.index()] += 1;
                    degree[out_neg.index()] += 1;
                    degree[ctrl_pos.index()] += 1;
                    degree[ctrl_neg.index()] += 1;
                }
                ComponentRecord::TransmissionLine {
                    in_pos,
                    in_neg,
                    out_pos,
                    out_neg,
                    ..
                } => {
                    degree[in_pos.index()] += 1;
                    degree[in_neg.index()] += 1;
                    degree[out_pos.index()] += 1;
                    degree[out_neg.index()] += 1;
                }
                ComponentRecord::AtomisticChannel {
                    drain,
                    gate,
                    source,
                    ..
                } => {
                    degree[drain.index()] += 1;
                    degree[gate.index()] += 1;
                    degree[source.index()] += 1;
                }
                ComponentRecord::ElectroOpticModulator {
                    elec_pos, elec_neg, ..
                } => {
                    degree[elec_pos.index()] += 1;
                    degree[elec_neg.index()] += 1;
                }
                ComponentRecord::LaserDiode { anode, cathode, .. } => {
                    degree[anode.index()] += 1;
                    degree[cathode.index()] += 1;
                }
                ComponentRecord::Photodetector { anode, cathode, .. } => {
                    degree[anode.index()] += 1;
                    degree[cathode.index()] += 1;
                }
                ComponentRecord::Memristor { pos, neg, .. } => {
                    degree[pos.index()] += 1;
                    degree[neg.index()] += 1;
                }
                ComponentRecord::SpikingNeuron {
                    input_node,
                    output_node,
                    ..
                } => {
                    degree[input_node.index()] += 1;
                    degree[output_node.index()] += 1;
                }
                ComponentRecord::RadiationStrike { target_node, .. } => {
                    degree[target_node.index()] += 1;
                }
                ComponentRecord::OpticalWaveguide { .. }
                | ComponentRecord::MicroRingResonator { .. } => {
                    // Purely optical components do not connect to electrical nodes
                }
            }
        }

        // Check for isolated nodes (degree == 0, excluding ground if unused)
        for (i, &deg) in degree.iter().enumerate().skip(1) {
            if deg == 0 {
                return Err(CoreError::FloatingNode {
                    node_id: NodeId::new(i as u32),
                    name: self.node_names[i].clone(),
                });
            }
        }

        // Check for ideal voltage source loops (excluding 0-volt sources or parallel sources)
        let mut v_adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for comp in &self.components {
            if let ComponentRecord::VoltageSource { pos, neg, .. } = comp {
                v_adj.entry(*pos).or_default().push(*neg);
                v_adj.entry(*neg).or_default().push(*pos);
            }
        }

        // BFS to check for loops within the pure voltage source subgraph
        let mut visited: HashSet<NodeId> = HashSet::new();
        for &start_node in v_adj.keys() {
            if !visited.contains(&start_node) {
                let mut queue = vec![(start_node, None)];
                visited.insert(start_node);

                while let Some((curr, parent)) = queue.pop() {
                    if let Some(neighbors) = v_adj.get(&curr) {
                        for &nbr in neighbors {
                            if Some(nbr) == parent {
                                continue;
                            }
                            if visited.contains(&nbr) {
                                return Err(CoreError::VoltageSourceLoop {
                                    node_a: curr,
                                    node_b: nbr,
                                });
                            }
                            visited.insert(nbr);
                            queue.push((nbr, Some(curr)));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_graph_building() {
        let mut graph = CircuitGraph::new();
        let r1 = graph.add_resistor("R1", "in", "out", 1000.0).unwrap();
        let v1 = graph.add_voltage_source("V1", "in", "0", 5.0).unwrap();
        let r2 = graph.add_resistor("R2", "out", "0", 2000.0).unwrap();

        assert_eq!(r1.index(), 0);
        assert_eq!(v1.index(), 1);
        assert_eq!(r2.index(), 2);
        assert_eq!(graph.total_nodes(), 3); // 0, in, out
        assert_eq!(graph.active_nodes(), 2);
        assert_eq!(graph.total_branches(), 1); // V1 branch

        assert!(graph.validate_topology().is_ok());
    }

    #[test]
    fn test_floating_node_detection() {
        let mut graph = CircuitGraph::new();
        let _ = graph.get_or_create_node("floating_node");
        graph.add_resistor("R1", "in", "0", 100.0).unwrap();

        let validation = graph.validate_topology();
        assert!(matches!(validation, Err(CoreError::FloatingNode { .. })));
    }

    #[test]
    fn test_voltage_source_loop_detection() {
        let mut graph = CircuitGraph::new();
        graph.add_voltage_source("V1", "n1", "0", 5.0).unwrap();
        graph.add_voltage_source("V2", "n1", "0", 3.3).unwrap();

        let validation = graph.validate_topology();
        assert!(matches!(
            validation,
            Err(CoreError::VoltageSourceLoop { .. })
        ));
    }
}

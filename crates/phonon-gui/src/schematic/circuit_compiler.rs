#![deny(unsafe_code)]

//! Translates visual schematic components and wires into a solvable CircuitGraph and SPICE netlist.

use super::components::{ComponentKind, SchematicComponent};
use super::net_label::NetLabel;
use super::wire::SchematicWire;
use egui::Pos2;
use phonon_core::CircuitGraph;
use phonon_models::bjt::BjtType;
use phonon_models::mosfet::MosfetType;
use phonon_models::{BjtModel, DiodeModel, MosfetModel};
use phonon_netlist::parse_spice_number;
use phonon_solver::mna::ModelContext;
use std::collections::{BTreeSet, HashMap};

/// Compilation results produced from the visual schematic editor.
#[derive(Debug, Clone)]
pub struct CompiledCircuit {
    pub graph: CircuitGraph,
    pub model_ctx: ModelContext,
    /// Maps `(component_name, pin_name)` -> `net_name` (e.g. `("R1", "1") -> "net1"`).
    pub pin_to_net: HashMap<(String, String), String>,
    /// Maps `wire_id` -> `net_name`.
    pub wire_to_net: HashMap<usize, String>,
    pub net_names: Vec<String>,
    pub spice_netlist: String,
}

/// Disjoint Set Union (DSU) helper for net extraction across 2D points.
struct DisjointSet {
    parent: HashMap<(i32, i32), (i32, i32)>,
}

impl DisjointSet {
    fn new() -> Self {
        Self {
            parent: HashMap::new(),
        }
    }

    fn find(&mut self, p: (i32, i32)) -> (i32, i32) {
        let entry = self.parent.entry(p).or_insert(p);
        let root = if *entry == p {
            p
        } else {
            let next = *entry;
            self.find(next)
        };
        self.parent.insert(p, root);
        root
    }

    fn union(&mut self, a: (i32, i32), b: (i32, i32)) {
        let root_a = self.find(a);
        let root_b = self.find(b);
        if root_a != root_b {
            self.parent.insert(root_a, root_b);
        }
    }
}

fn quantize(p: Pos2) -> (i32, i32) {
    // Quantize within 4px tolerance
    let qx = (p.x / 4.0).round() as i32 * 4;
    let qy = (p.y / 4.0).round() as i32 * 4;
    (qx, qy)
}

/// Compiles visual components and wires into an electrical `CircuitGraph`.
pub fn compile_schematic(
    components: &[SchematicComponent],
    wires: &[SchematicWire],
) -> Result<CompiledCircuit, String> {
    compile_schematic_with_labels(components, wires, &[])
}

/// Compiles visual components, wires, and net labels into an electrical `CircuitGraph`.
pub fn compile_schematic_with_labels(
    components: &[SchematicComponent],
    wires: &[SchematicWire],
    net_labels: &[NetLabel],
) -> Result<CompiledCircuit, String> {
    let mut dsu = DisjointSet::new();

    // 1. Union wire segment endpoints
    for wire in wires {
        for seg in &wire.segments {
            let p1 = quantize(seg.start);
            let p2 = quantize(seg.end);
            dsu.union(p1, p2);
        }
    }

    // 1b. Union wire endpoints that touch other wire segments (T-junctions)
    for wire in wires {
        for seg in &wire.segments {
            for other_wire in wires {
                for other_seg in &other_wire.segments {
                    for &pt in &[other_seg.start, other_seg.end] {
                        if seg.contains_point(pt, 4.0) {
                            dsu.union(quantize(pt), quantize(seg.start));
                            dsu.union(quantize(pt), quantize(seg.end));
                        }
                    }
                }
            }
        }
    }

    // 2. Union pins that touch wire segments
    for comp in components {
        for (_, pin_pos) in comp.all_pins() {
            let pin_q = quantize(pin_pos);
            for wire in wires {
                for seg in &wire.segments {
                    if seg.contains_point(pin_pos, 4.0) {
                        dsu.union(pin_q, quantize(seg.start));
                        dsu.union(pin_q, quantize(seg.end));
                    }
                }
            }
        }
    }

    // 2b. Union net labels that touch wire segments or pins
    for label in net_labels {
        let label_q = quantize(label.pos);
        for wire in wires {
            for seg in &wire.segments {
                if seg.contains_point(label.pos, 4.0) {
                    dsu.union(label_q, quantize(seg.start));
                    dsu.union(label_q, quantize(seg.end));
                }
            }
        }
        for comp in components {
            for (_, pin_pos) in comp.all_pins() {
                if (pin_pos - label.pos).length() <= 4.0 {
                    dsu.union(label_q, quantize(pin_pos));
                }
            }
        }
    }

    // 2c. Union net labels sharing identical names (case-insensitive)
    let mut label_groups: HashMap<String, Vec<(i32, i32)>> = HashMap::new();
    for label in net_labels {
        let key = label.name.trim().to_uppercase();
        if !key.is_empty() {
            label_groups.entry(key).or_default().push(quantize(label.pos));
        }
    }
    for pts in label_groups.values() {
        if pts.len() >= 2 {
            let first = pts[0];
            for &other in &pts[1..] {
                dsu.union(first, other);
            }
        }
    }

    // 3. Identify all points associated with Ground
    let mut ground_roots = Vec::new();
    for comp in components {
        if comp.kind == ComponentKind::Ground {
            let pin_pos = comp.pin_world_pos(0).unwrap_or(comp.pos);
            let root = dsu.find(quantize(pin_pos));
            ground_roots.push(root);
        }
    }
    for label in net_labels {
        let upper = label.name.trim().to_uppercase();
        if upper == "0" || upper == "GND" || upper == "GROUND" {
            let root = dsu.find(quantize(label.pos));
            ground_roots.push(root);
        }
    }

    // 4. Assign human-readable net names to root clusters
    let mut root_to_net: HashMap<(i32, i32), String> = HashMap::new();
    for &gr in &ground_roots {
        root_to_net.insert(gr, "0".to_string());
    }

    // Pre-populate root clusters with explicit NetLabel names
    for label in net_labels {
        let upper = label.name.trim().to_uppercase();
        if upper == "0" || upper == "GND" || upper == "GROUND" {
            continue;
        }
        let root = dsu.find(quantize(label.pos));
        if !root_to_net.contains_key(&root) {
            root_to_net.insert(root, label.name.clone());
        }
    }

    let mut net_counter = 1;
    let mut pin_to_net = HashMap::new();
    let mut all_nets = Vec::new();

    for comp in components {
        if comp.kind == ComponentKind::Ground {
            continue;
        }
        for (pin_name, pin_pos) in comp.all_pins() {
            let root = dsu.find(quantize(pin_pos));
            let net_name = root_to_net.entry(root).or_insert_with(|| {
                let name = format!("net{}", net_counter);
                net_counter += 1;
                name
            });
            if !all_nets.contains(net_name) {
                all_nets.push(net_name.clone());
            }
            pin_to_net.insert((comp.name.clone(), pin_name.to_string()), net_name.clone());
        }
    }

    for label in net_labels {
        let root = dsu.find(quantize(label.pos));
        if let Some(net) = root_to_net.get(&root) {
            if !all_nets.contains(net) {
                all_nets.push(net.clone());
            }
        }
    }

    if !all_nets.contains(&"0".to_string()) && !ground_roots.is_empty() {
        all_nets.insert(0, "0".to_string());
    }

    // 5. Build CircuitGraph and ModelContext
    let mut graph = CircuitGraph::new();
    let mut model_ctx = ModelContext::new();
    let mut spice_lines = vec![
        "* Exported from Phonon CAD Schematic".to_string(),
        ".TEMP 27.0".to_string(),
    ];
    let mut subckts: BTreeSet<String> = BTreeSet::new();

    let get_net = |comp_name: &str, pin_name: &str| -> String {
        pin_to_net
            .get(&(comp_name.to_string(), pin_name.to_string()))
            .cloned()
            .unwrap_or_else(|| "0".to_string())
    };

    for comp in components {
        match comp.kind {
            ComponentKind::Ground => {}

            // Passives
            ComponentKind::Resistor => {
                let n1 = get_net(&comp.name, "1");
                let n2 = get_net(&comp.name, "2");
                let val = parse_spice_number(&comp.value_str, 1).unwrap_or(1000.0);
                graph
                    .add_resistor(&comp.name, &n1, &n2, val)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
            }
            ComponentKind::Capacitor => {
                let n1 = get_net(&comp.name, "1");
                let n2 = get_net(&comp.name, "2");
                let val = parse_spice_number(&comp.value_str, 1).unwrap_or(1e-7);
                graph
                    .add_capacitor(&comp.name, &n1, &n2, val, None)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
            }
            ComponentKind::Inductor => {
                let n1 = get_net(&comp.name, "1");
                let n2 = get_net(&comp.name, "2");
                let val = parse_spice_number(&comp.value_str, 1).unwrap_or(1e-5);
                graph
                    .add_inductor(&comp.name, &n1, &n2, val, None)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
            }
            ComponentKind::Transformer => {
                let p_pos = get_net(&comp.name, "P+");
                let p_neg = get_net(&comp.name, "P-");
                let s_pos = get_net(&comp.name, "S+");
                let s_neg = get_net(&comp.name, "S-");
                let lp_name = format!("{}_LP", comp.name);
                let ls_name = format!("{}_LS", comp.name);
                graph
                    .add_inductor(&lp_name, &p_pos, &p_neg, 1e-3, None)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_inductor(&ls_name, &s_pos, &s_neg, 1e-3, None)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("L{}_P {} {} 1m", comp.name, p_pos, p_neg));
                spice_lines.push(format!("L{}_S {} {} 1m", comp.name, s_pos, s_neg));
                spice_lines.push(format!("K{} L{}_P L{}_S 0.999", comp.name, comp.name, comp.name));
            }
            ComponentKind::Potentiometer => {
                let n1 = get_net(&comp.name, "1");
                let nw = get_net(&comp.name, "WIPER");
                let n2 = get_net(&comp.name, "2");
                let total_r = parse_spice_number(&comp.value_str, 1).unwrap_or(10000.0);
                let half_r = (total_r * 0.5).max(1.0);
                let r1_name = format!("{}_R1", comp.name);
                let r2_name = format!("{}_R2", comp.name);
                graph
                    .add_resistor(&r1_name, &n1, &nw, half_r)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&r2_name, &nw, &n2, half_r)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", r1_name, n1, nw, half_r));
                spice_lines.push(format!("{} {} {} {}", r2_name, nw, n2, half_r));
            }
            ComponentKind::SwitchSpst => {
                let n1 = get_net(&comp.name, "1");
                let n2 = get_net(&comp.name, "2");
                let r_val = if comp.value_str.eq_ignore_ascii_case("CLOSED") { 1e-3 } else { 1e7 };
                graph
                    .add_resistor(&comp.name, &n1, &n2, r_val)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, r_val));
            }
            ComponentKind::PushButton => {
                let n1 = get_net(&comp.name, "1");
                let n2 = get_net(&comp.name, "2");
                let r_val = if comp.value_str.eq_ignore_ascii_case("PRESSED") { 1e-3 } else { 1e7 };
                graph
                    .add_resistor(&comp.name, &n1, &n2, r_val)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, r_val));
            }

            // Sources & Generators
            ComponentKind::VoltageSource => {
                let n1 = get_net(&comp.name, "+");
                let n2 = get_net(&comp.name, "-");
                let val = parse_spice_number(&comp.value_str, 1).unwrap_or(5.0);
                graph
                    .add_voltage_source(&comp.name, &n1, &n2, val)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
            }
            ComponentKind::AcVoltageSource => {
                let n1 = get_net(&comp.name, "+");
                let n2 = get_net(&comp.name, "-");
                graph
                    .add_voltage_source(&comp.name, &n1, &n2, 1.0)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} AC 1.0 {}", comp.name, n1, n2, comp.value_str));
            }
            ComponentKind::CurrentSource => {
                let n1 = get_net(&comp.name, "+");
                let n2 = get_net(&comp.name, "-");
                let val = parse_spice_number(&comp.value_str, 1).unwrap_or(0.001);
                graph
                    .add_current_source(&comp.name, &n1, &n2, val)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
            }
            ComponentKind::PulseGenerator => {
                let n1 = get_net(&comp.name, "+");
                let n2 = get_net(&comp.name, "-");
                graph
                    .add_voltage_source(&comp.name, &n1, &n2, 5.0)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
            }
            ComponentKind::ClockSource => {
                let n_clk = get_net(&comp.name, "CLK");
                let n_gnd = get_net(&comp.name, "GND");
                graph
                    .add_voltage_source(&comp.name, &n_clk, &n_gnd, 5.0)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n_clk, n_gnd, comp.value_str));
            }
            ComponentKind::VddRail => {
                let n_vdd = get_net(&comp.name, "VDD");
                let val = parse_spice_number(&comp.value_str, 1).unwrap_or(5.0);
                graph
                    .add_voltage_source(&comp.name, &n_vdd, "0", val)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} 0 {}", comp.name, n_vdd, val));
            }

            // Discrete Semiconductors
            ComponentKind::Diode => {
                let n1 = get_net(&comp.name, "A");
                let n2 = get_net(&comp.name, "K");
                let model = DiodeModel::default();
                model_ctx.set_diode_model(&comp.name, model);
                graph
                    .add_diode(&comp.name, &n1, &n2)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
                spice_lines.push(format!(".MODEL {} D (IS=1e-14 RS=0.1)", comp.value_str));
            }
            ComponentKind::ZenerDiode => {
                let n1 = get_net(&comp.name, "A");
                let n2 = get_net(&comp.name, "K");
                let model = DiodeModel {
                    bv: 5.1,
                    ibv: 1e-3,
                    ..DiodeModel::default()
                };
                model_ctx.set_diode_model(&comp.name, model);
                graph
                    .add_diode(&comp.name, &n1, &n2)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
                spice_lines.push(format!(
                    ".MODEL {} D (IS=1e-14 RS=0.5 BV=5.1 IBV=1e-3)",
                    comp.value_str
                ));
            }
            ComponentKind::Led => {
                let n1 = get_net(&comp.name, "A");
                let n2 = get_net(&comp.name, "K");
                let model = DiodeModel {
                    n: 1.8,
                    ..DiodeModel::default()
                };
                model_ctx.set_diode_model(&comp.name, model);
                graph
                    .add_diode(&comp.name, &n1, &n2)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
                spice_lines.push(format!(".MODEL {} D (IS=1e-18 N=1.8 RS=2.0)", comp.value_str));
            }
            ComponentKind::SchottkyDiode => {
                let n1 = get_net(&comp.name, "A");
                let n2 = get_net(&comp.name, "K");
                let model = DiodeModel {
                    is: 1e-7,
                    n: 1.05,
                    rs: 0.05,
                    ..DiodeModel::default()
                };
                model_ctx.set_diode_model(&comp.name, model);
                graph
                    .add_diode(&comp.name, &n1, &n2)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
                spice_lines.push(format!(
                    ".MODEL {} D (IS=1e-7 RS=0.05 N=1.05)",
                    comp.value_str
                ));
            }

            // Transistors & Cryo-CMOS
            ComponentKind::Nmos => {
                let d = get_net(&comp.name, "D");
                let g = get_net(&comp.name, "G");
                let s = get_net(&comp.name, "S");
                let b = s.clone();
                let model = MosfetModel {
                    mos_type: MosfetType::Nmos,
                    ..MosfetModel::default()
                };
                model_ctx.set_mosfet_model(&comp.name, model);
                graph
                    .add_mosfet(&comp.name, &d, &g, &s, &b)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!(
                    "{} {} {} {} {} {}",
                    comp.name, d, g, s, b, comp.value_str
                ));
            }
            ComponentKind::Pmos => {
                let d = get_net(&comp.name, "D");
                let g = get_net(&comp.name, "G");
                let s = get_net(&comp.name, "S");
                let b = s.clone();
                let model = MosfetModel {
                    mos_type: MosfetType::Pmos,
                    ..MosfetModel::default()
                };
                model_ctx.set_mosfet_model(&comp.name, model);
                graph
                    .add_mosfet(&comp.name, &d, &g, &s, &b)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!(
                    "{} {} {} {} {} {}",
                    comp.name, d, g, s, b, comp.value_str
                ));
            }
            ComponentKind::FinFet => {
                let d = get_net(&comp.name, "D");
                let g = get_net(&comp.name, "G");
                let s = get_net(&comp.name, "S");
                let b = get_net(&comp.name, "B");
                let model = MosfetModel {
                    mos_type: MosfetType::Nmos,
                    ..MosfetModel::default()
                };
                model_ctx.set_mosfet_model(&comp.name, model);
                graph
                    .add_mosfet(&comp.name, &d, &g, &s, &b)
                    .map_err(|e| e.to_string())?;
                subckts.insert("FINFET_3NM".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {}",
                    comp.name, d, g, s, b, comp.value_str
                ));
            }
            ComponentKind::GaaNanosheet => {
                let d = get_net(&comp.name, "D");
                let g = get_net(&comp.name, "G");
                let s = get_net(&comp.name, "S");
                let b = get_net(&comp.name, "B");
                let model = MosfetModel {
                    mos_type: MosfetType::Nmos,
                    ..MosfetModel::default()
                };
                model_ctx.set_mosfet_model(&comp.name, model);
                graph
                    .add_mosfet(&comp.name, &d, &g, &s, &b)
                    .map_err(|e| e.to_string())?;
                subckts.insert("GAA_2NM".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {}",
                    comp.name, d, g, s, b, comp.value_str
                ));
            }
            ComponentKind::BjtNpn => {
                let c = get_net(&comp.name, "C");
                let b = get_net(&comp.name, "B");
                let e = get_net(&comp.name, "E");
                let model = BjtModel {
                    bjt_type: BjtType::Npn,
                    ..BjtModel::default()
                };
                model_ctx.set_bjt_model(&comp.name, model);
                graph
                    .add_bjt(&comp.name, &c, &b, &e)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!(
                    "{} {} {} {} {}",
                    comp.name, c, b, e, comp.value_str
                ));
            }
            ComponentKind::BjtPnp => {
                let c = get_net(&comp.name, "C");
                let b = get_net(&comp.name, "B");
                let e = get_net(&comp.name, "E");
                let model = BjtModel {
                    bjt_type: BjtType::Pnp,
                    ..BjtModel::default()
                };
                model_ctx.set_bjt_model(&comp.name, model);
                graph
                    .add_bjt(&comp.name, &c, &b, &e)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!(
                    "{} {} {} {} {}",
                    comp.name, c, b, e, comp.value_str
                ));
            }

            // Integrated Circuits & Logic
            ComponentKind::OpAmp => {
                let in_plus = get_net(&comp.name, "IN+");
                let in_minus = get_net(&comp.name, "IN-");
                let out = get_net(&comp.name, "OUT");
                let v_plus = get_net(&comp.name, "V+");
                let v_minus = get_net(&comp.name, "V-");
                let rin_name = format!("{}_RIN", comp.name);
                let e_name = format!("{}_E", comp.name);
                let rout_name = format!("{}_ROUT", comp.name);
                graph
                    .add_resistor(&rin_name, &in_plus, &in_minus, 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_vcvs(&e_name, &out, &v_minus, &in_plus, &in_minus, 1e5)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rout_name, &out, &v_minus, 50.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("OPAMP_IDEAL".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {} {}",
                    comp.name, in_plus, in_minus, v_plus, v_minus, out, comp.value_str
                ));
            }
            ComponentKind::Inverter => {
                let in_node = get_net(&comp.name, "IN");
                let out_node = get_net(&comp.name, "OUT");
                let rin_name = format!("{}_RIN", comp.name);
                let e_name = format!("{}_E", comp.name);
                let rout_name = format!("{}_ROUT", comp.name);
                graph
                    .add_resistor(&rin_name, &in_node, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_vcvs(&e_name, &out_node, "0", &in_node, "0", -1.0)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rout_name, &out_node, "0", 100.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("INV_CMOS".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {}",
                    comp.name, in_node, out_node, comp.value_str
                ));
            }
            ComponentKind::NandGate => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let out_node = get_net(&comp.name, "OUT");
                let ra_name = format!("{}_RA", comp.name);
                let rb_name = format!("{}_RB", comp.name);
                let rout_name = format!("{}_ROUT", comp.name);
                graph
                    .add_resistor(&ra_name, &a, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rb_name, &b, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rout_name, &out_node, "0", 100.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("NAND2".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {}",
                    comp.name, a, b, out_node, comp.value_str
                ));
            }
            ComponentKind::NorGate => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let out_node = get_net(&comp.name, "OUT");
                let ra_name = format!("{}_RA", comp.name);
                let rb_name = format!("{}_RB", comp.name);
                let rout_name = format!("{}_ROUT", comp.name);
                graph
                    .add_resistor(&ra_name, &a, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rb_name, &b, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rout_name, &out_node, "0", 100.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("NOR2".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {}",
                    comp.name, a, b, out_node, comp.value_str
                ));
            }
            ComponentKind::Mux2to1 => {
                let d0 = get_net(&comp.name, "D0");
                let d1 = get_net(&comp.name, "D1");
                let sel = get_net(&comp.name, "SEL");
                let out_node = get_net(&comp.name, "OUT");
                let r0_name = format!("{}_R0", comp.name);
                let r1_name = format!("{}_R1", comp.name);
                let rs_name = format!("{}_RS", comp.name);
                let rout_name = format!("{}_ROUT", comp.name);
                graph
                    .add_resistor(&r0_name, &d0, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&r1_name, &d1, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rs_name, &sel, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rout_name, &out_node, "0", 100.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("MUX21".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {}",
                    comp.name, d0, d1, sel, out_node, comp.value_str
                ));
            }
            ComponentKind::BufferGate => {
                let in_node = get_net(&comp.name, "IN");
                let out_node = get_net(&comp.name, "OUT");
                let rin_name = format!("{}_RIN", comp.name);
                let e_name = format!("{}_E", comp.name);
                let rout_name = format!("{}_ROUT", comp.name);
                graph
                    .add_resistor(&rin_name, &in_node, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_vcvs(&e_name, &out_node, "0", &in_node, "0", 1.0)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rout_name, &out_node, "0", 100.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("BUF_CMOS".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {}",
                    comp.name, in_node, out_node, comp.value_str
                ));
            }
            ComponentKind::AndGate => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let out_node = get_net(&comp.name, "OUT");
                let ra_name = format!("{}_RA", comp.name);
                let rb_name = format!("{}_RB", comp.name);
                let rout_name = format!("{}_ROUT", comp.name);
                graph
                    .add_resistor(&ra_name, &a, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rb_name, &b, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rout_name, &out_node, "0", 100.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("AND2".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {}",
                    comp.name, a, b, out_node, comp.value_str
                ));
            }
            ComponentKind::OrGate => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let out_node = get_net(&comp.name, "OUT");
                let ra_name = format!("{}_RA", comp.name);
                let rb_name = format!("{}_RB", comp.name);
                let rout_name = format!("{}_ROUT", comp.name);
                graph
                    .add_resistor(&ra_name, &a, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rb_name, &b, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rout_name, &out_node, "0", 100.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("OR2".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {}",
                    comp.name, a, b, out_node, comp.value_str
                ));
            }
            ComponentKind::XorGate => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let out_node = get_net(&comp.name, "OUT");
                let ra_name = format!("{}_RA", comp.name);
                let rb_name = format!("{}_RB", comp.name);
                let rout_name = format!("{}_ROUT", comp.name);
                graph
                    .add_resistor(&ra_name, &a, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rb_name, &b, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rout_name, &out_node, "0", 100.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("XOR2".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {}",
                    comp.name, a, b, out_node, comp.value_str
                ));
            }
            ComponentKind::XnorGate => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let out_node = get_net(&comp.name, "OUT");
                let ra_name = format!("{}_RA", comp.name);
                let rb_name = format!("{}_RB", comp.name);
                let rout_name = format!("{}_ROUT", comp.name);
                graph
                    .add_resistor(&ra_name, &a, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rb_name, &b, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rout_name, &out_node, "0", 100.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("XNOR2".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {}",
                    comp.name, a, b, out_node, comp.value_str
                ));
            }
            ComponentKind::HalfAdder => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let sum = get_net(&comp.name, "SUM");
                let cout = get_net(&comp.name, "COUT");
                let ra_name = format!("{}_RA", comp.name);
                let rb_name = format!("{}_RB", comp.name);
                let rsum_name = format!("{}_RSUM", comp.name);
                let rcout_name = format!("{}_RCOUT", comp.name);
                graph
                    .add_resistor(&ra_name, &a, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rb_name, &b, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rsum_name, &sum, "0", 100.0)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rcout_name, &cout, "0", 100.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("HALF_ADDER".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {}",
                    comp.name, a, b, sum, cout, comp.value_str
                ));
            }
            ComponentKind::FullAdder => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let cin = get_net(&comp.name, "CIN");
                let sum = get_net(&comp.name, "SUM");
                let cout = get_net(&comp.name, "COUT");
                let ra_name = format!("{}_RA", comp.name);
                let rb_name = format!("{}_RB", comp.name);
                let rcin_name = format!("{}_RCIN", comp.name);
                let rsum_name = format!("{}_RSUM", comp.name);
                let rcout_name = format!("{}_RCOUT", comp.name);
                graph
                    .add_resistor(&ra_name, &a, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rb_name, &b, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rcin_name, &cin, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rsum_name, &sum, "0", 100.0)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rcout_name, &cout, "0", 100.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("FULL_ADDER".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {} {}",
                    comp.name, a, b, cin, sum, cout, comp.value_str
                ));
            }
            ComponentKind::Adder => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let cin = get_net(&comp.name, "CIN");
                let sum = get_net(&comp.name, "SUM");
                let cout = get_net(&comp.name, "COUT");
                graph.add_resistor(&format!("{}_RA", comp.name), &a, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RB", comp.name), &b, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RCIN", comp.name), &cin, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RSUM", comp.name), &sum, "0", 100.0).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RCOUT", comp.name), &cout, "0", 100.0).map_err(|e| e.to_string())?;
                subckts.insert("ADDER".to_string());
                spice_lines.push(format!("X{} {} {} {} {} {} {}", comp.name, a, b, cin, sum, cout, comp.value_str));
            }
            ComponentKind::Subtractor => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let bin = get_net(&comp.name, "BIN");
                let diff = get_net(&comp.name, "DIFF");
                let bout = get_net(&comp.name, "BOUT");
                graph.add_resistor(&format!("{}_RA", comp.name), &a, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RB", comp.name), &b, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RBIN", comp.name), &bin, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RDIFF", comp.name), &diff, "0", 100.0).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RBOUT", comp.name), &bout, "0", 100.0).map_err(|e| e.to_string())?;
                subckts.insert("SUBTRACTOR".to_string());
                spice_lines.push(format!("X{} {} {} {} {} {} {}", comp.name, a, b, bin, diff, bout, comp.value_str));
            }
            ComponentKind::Multiplier => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let prod = get_net(&comp.name, "PROD");
                graph.add_resistor(&format!("{}_RA", comp.name), &a, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RB", comp.name), &b, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RPROD", comp.name), &prod, "0", 100.0).map_err(|e| e.to_string())?;
                subckts.insert("MULTIPLIER".to_string());
                spice_lines.push(format!("X{} {} {} {} {}", comp.name, a, b, prod, comp.value_str));
            }
            ComponentKind::Divider => {
                let num = get_net(&comp.name, "NUM");
                let den = get_net(&comp.name, "DEN");
                let quot = get_net(&comp.name, "QUOT");
                let rem = get_net(&comp.name, "REM");
                graph.add_resistor(&format!("{}_RNUM", comp.name), &num, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RDEN", comp.name), &den, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RQUOT", comp.name), &quot, "0", 100.0).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RREM", comp.name), &rem, "0", 100.0).map_err(|e| e.to_string())?;
                subckts.insert("DIVIDER".to_string());
                spice_lines.push(format!("X{} {} {} {} {} {}", comp.name, num, den, quot, rem, comp.value_str));
            }
            ComponentKind::ArithmeticLogicUnit => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let op = get_net(&comp.name, "OP");
                let cin = get_net(&comp.name, "CIN");
                let out = get_net(&comp.name, "OUT");
                let flags = get_net(&comp.name, "FLAGS");
                let cout = get_net(&comp.name, "COUT");
                graph.add_resistor(&format!("{}_RA", comp.name), &a, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RB", comp.name), &b, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_ROP", comp.name), &op, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RCIN", comp.name), &cin, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_ROUT", comp.name), &out, "0", 100.0).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RFLAGS", comp.name), &flags, "0", 100.0).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RCOUT", comp.name), &cout, "0", 100.0).map_err(|e| e.to_string())?;
                subckts.insert("ALU".to_string());
                spice_lines.push(format!("X{} {} {} {} {} {} {} {} {}", comp.name, a, b, op, cin, out, flags, cout, comp.value_str));
            }
            ComponentKind::BitSplitter => {
                let in_net = get_net(&comp.name, "IN");
                let out0 = get_net(&comp.name, "OUT0");
                let out1 = get_net(&comp.name, "OUT1");
                graph.add_resistor(&format!("{}_RIN", comp.name), &in_net, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_ROUT0", comp.name), &out0, "0", 50.0).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_ROUT1", comp.name), &out1, "0", 50.0).map_err(|e| e.to_string())?;
                subckts.insert("BIT_SPLITTER".to_string());
                spice_lines.push(format!("X{} {} {} {} {}", comp.name, in_net, out0, out1, comp.value_str));
            }
            ComponentKind::BitMerger => {
                let in0 = get_net(&comp.name, "IN0");
                let in1 = get_net(&comp.name, "IN1");
                let out = get_net(&comp.name, "OUT");
                graph.add_resistor(&format!("{}_RIN0", comp.name), &in0, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RIN1", comp.name), &in1, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_ROUT", comp.name), &out, "0", 50.0).map_err(|e| e.to_string())?;
                subckts.insert("BIT_MERGER".to_string());
                spice_lines.push(format!("X{} {} {} {} {}", comp.name, in0, in1, out, comp.value_str));
            }
            ComponentKind::BusTap => {
                let in_net = get_net(&comp.name, "IN");
                let thru = get_net(&comp.name, "THRU");
                let tap = get_net(&comp.name, "TAP");
                graph.add_resistor(&format!("{}_RTHRU", comp.name), &in_net, &thru, 0.01).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RTAP", comp.name), &in_net, &tap, 0.01).map_err(|e| e.to_string())?;
                subckts.insert("BUS_TAP".to_string());
                spice_lines.push(format!("X{} {} {} {} {}", comp.name, in_net, thru, tap, comp.value_str));
            }
            ComponentKind::FloatAdder => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let sum = get_net(&comp.name, "SUM");
                graph.add_resistor(&format!("{}_RA", comp.name), &a, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RB", comp.name), &b, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RSUM", comp.name), &sum, "0", 50.0).map_err(|e| e.to_string())?;
                subckts.insert("FLOAT_ADDER".to_string());
                spice_lines.push(format!("X{} {} {} {} {}", comp.name, a, b, sum, comp.value_str));
            }
            ComponentKind::FloatSubtractor => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let diff = get_net(&comp.name, "DIFF");
                graph.add_resistor(&format!("{}_RA", comp.name), &a, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RB", comp.name), &b, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RDIFF", comp.name), &diff, "0", 50.0).map_err(|e| e.to_string())?;
                subckts.insert("FLOAT_SUBTRACTOR".to_string());
                spice_lines.push(format!("X{} {} {} {} {}", comp.name, a, b, diff, comp.value_str));
            }
            ComponentKind::FloatMultiplier => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let prod = get_net(&comp.name, "PROD");
                graph.add_resistor(&format!("{}_RA", comp.name), &a, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RB", comp.name), &b, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RPROD", comp.name), &prod, "0", 50.0).map_err(|e| e.to_string())?;
                subckts.insert("FLOAT_MULTIPLIER".to_string());
                spice_lines.push(format!("X{} {} {} {} {}", comp.name, a, b, prod, comp.value_str));
            }
            ComponentKind::FloatDivider => {
                let num = get_net(&comp.name, "NUM");
                let den = get_net(&comp.name, "DEN");
                let quot = get_net(&comp.name, "QUOT");
                graph.add_resistor(&format!("{}_RNUM", comp.name), &num, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RDEN", comp.name), &den, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RQUOT", comp.name), &quot, "0", 50.0).map_err(|e| e.to_string())?;
                subckts.insert("FLOAT_DIVIDER".to_string());
                spice_lines.push(format!("X{} {} {} {} {}", comp.name, num, den, quot, comp.value_str));
            }
            ComponentKind::FloatComparator => {
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let gt = get_net(&comp.name, "GT");
                let eq = get_net(&comp.name, "EQ");
                let lt = get_net(&comp.name, "LT");
                graph.add_resistor(&format!("{}_RA", comp.name), &a, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RB", comp.name), &b, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RGT", comp.name), &gt, "0", 50.0).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_REQ", comp.name), &eq, "0", 50.0).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RLT", comp.name), &lt, "0", 50.0).map_err(|e| e.to_string())?;
                subckts.insert("FLOAT_COMPARATOR".to_string());
                spice_lines.push(format!("X{} {} {} {} {} {} {}", comp.name, a, b, gt, eq, lt, comp.value_str));
            }
            ComponentKind::Mux4to1 => {
                let d0 = get_net(&comp.name, "D0");
                let d1 = get_net(&comp.name, "D1");
                let d2 = get_net(&comp.name, "D2");
                let d3 = get_net(&comp.name, "D3");
                let s0 = get_net(&comp.name, "S0");
                let s1 = get_net(&comp.name, "S1");
                let out_node = get_net(&comp.name, "OUT");
                let rout_name = format!("{}_ROUT", comp.name);
                for (pin_name, net) in [("D0", &d0), ("D1", &d1), ("D2", &d2), ("D3", &d3), ("S0", &s0), ("S1", &s1)] {
                    let r_name = format!("{}_R_{}", comp.name, pin_name);
                    graph.add_resistor(&r_name, net, "0", 1e7).map_err(|e| e.to_string())?;
                }
                graph.add_resistor(&rout_name, &out_node, "0", 100.0).map_err(|e| e.to_string())?;
                subckts.insert("MUX41".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {} {} {} {}",
                    comp.name, d0, d1, d2, d3, s0, s1, out_node, comp.value_str
                ));
            }
            ComponentKind::Demux1to2 => {
                let in_net = get_net(&comp.name, "IN");
                let sel = get_net(&comp.name, "SEL");
                let y0 = get_net(&comp.name, "Y0");
                let y1 = get_net(&comp.name, "Y1");
                graph.add_resistor(&format!("{}_RIN", comp.name), &in_net, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RSEL", comp.name), &sel, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RY0", comp.name), &y0, "0", 100.0).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RY1", comp.name), &y1, "0", 100.0).map_err(|e| e.to_string())?;
                subckts.insert("DEMUX12".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {}",
                    comp.name, in_net, sel, y0, y1, comp.value_str
                ));
            }
            ComponentKind::DFlipFlop => {
                let d = get_net(&comp.name, "D");
                let clk = get_net(&comp.name, "CLK");
                let q = get_net(&comp.name, "Q");
                let qn = get_net(&comp.name, "QN");
                graph.add_resistor(&format!("{}_RD", comp.name), &d, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RCLK", comp.name), &clk, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RQ", comp.name), &q, "0", 100.0).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RQN", comp.name), &qn, "0", 100.0).map_err(|e| e.to_string())?;
                subckts.insert("DFF".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {}",
                    comp.name, d, clk, q, qn, comp.value_str
                ));
            }
            ComponentKind::SrLatch => {
                let s = get_net(&comp.name, "S");
                let r = get_net(&comp.name, "R");
                let q = get_net(&comp.name, "Q");
                let qn = get_net(&comp.name, "QN");
                graph.add_resistor(&format!("{}_RS", comp.name), &s, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RR", comp.name), &r, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RQ", comp.name), &q, "0", 100.0).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RQN", comp.name), &qn, "0", 100.0).map_err(|e| e.to_string())?;
                subckts.insert("SRLATCH".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {}",
                    comp.name, s, r, q, qn, comp.value_str
                ));
            }
            ComponentKind::Counter4Bit => {
                let clk = get_net(&comp.name, "CLK");
                let rst = get_net(&comp.name, "RST");
                let q0 = get_net(&comp.name, "Q0");
                let q1 = get_net(&comp.name, "Q1");
                let q2 = get_net(&comp.name, "Q2");
                let q3 = get_net(&comp.name, "Q3");
                graph.add_resistor(&format!("{}_RCLK", comp.name), &clk, "0", 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RRST", comp.name), &rst, "0", 1e7).map_err(|e| e.to_string())?;
                for (name, net) in [("Q0", &q0), ("Q1", &q1), ("Q2", &q2), ("Q3", &q3)] {
                    graph.add_resistor(&format!("{}_R_{}", comp.name, name), net, "0", 100.0).map_err(|e| e.to_string())?;
                }
                subckts.insert("COUNTER4".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {} {} {}",
                    comp.name, clk, rst, q0, q1, q2, q3, comp.value_str
                ));
            }
            ComponentKind::Comparator => {
                let inp = get_net(&comp.name, "IN+");
                let inn = get_net(&comp.name, "IN-");
                let out_node = get_net(&comp.name, "OUT");
                let rin_name = format!("{}_RIN", comp.name);
                let rout_name = format!("{}_ROUT", comp.name);
                graph.add_resistor(&rin_name, &inp, &inn, 1e7).map_err(|e| e.to_string())?;
                graph.add_resistor(&rout_name, &out_node, "0", 50.0).map_err(|e| e.to_string())?;
                subckts.insert("COMPARATOR".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {}",
                    comp.name, inp, inn, out_node, comp.value_str
                ));
            }
            ComponentKind::Timer555 => {
                let gnd = get_net(&comp.name, "GND");
                let trig = get_net(&comp.name, "TRIG");
                let out_node = get_net(&comp.name, "OUT");
                let reset = get_net(&comp.name, "RESET");
                let ctrl = get_net(&comp.name, "CTRL");
                let thres = get_net(&comp.name, "THRES");
                let disch = get_net(&comp.name, "DISCH");
                let vcc = get_net(&comp.name, "VCC");
                graph.add_resistor(&format!("{}_RTRIG", comp.name), &trig, &gnd, 1e6).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_RTHRES", comp.name), &thres, &gnd, 1e6).map_err(|e| e.to_string())?;
                graph.add_resistor(&format!("{}_ROUT", comp.name), &out_node, &gnd, 50.0).map_err(|e| e.to_string())?;
                subckts.insert("LM555".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {} {} {} {} {}",
                    comp.name, gnd, trig, out_node, reset, ctrl, thres, disch, vcc, comp.value_str
                ));
            }
            ComponentKind::VoltageRegulator => {
                let vin = get_net(&comp.name, "VIN");
                let gnd = get_net(&comp.name, "GND");
                let vout = get_net(&comp.name, "VOUT");
                graph.add_resistor(&format!("{}_RVIN", comp.name), &vin, &gnd, 1e5).map_err(|e| e.to_string())?;
                graph.add_voltage_source(&format!("{}_VREG", comp.name), &vout, &gnd, 5.0).map_err(|e| e.to_string())?;
                subckts.insert("LM7805".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {}",
                    comp.name, vin, gnd, vout, comp.value_str
                ));
            }
            ComponentKind::LogicProbe => {
                let in_net = get_net(&comp.name, "IN");
                graph.add_resistor(&format!("{}_R", comp.name), &in_net, "0", 1e7).map_err(|e| e.to_string())?;
                spice_lines.push(format!("* PROBE {} on {}", comp.name, in_net));
            }
            ComponentKind::SevenSegment => {
                let com = get_net(&comp.name, "COM");
                for seg in ["A", "B", "C", "D", "E", "F", "G"] {
                    let seg_net = get_net(&comp.name, seg);
                    graph.add_resistor(&format!("{}_R_{}", comp.name, seg), &seg_net, &com, 220.0).map_err(|e| e.to_string())?;
                }
                subckts.insert("7SEG_CC".to_string());
                let a = get_net(&comp.name, "A");
                let b = get_net(&comp.name, "B");
                let c = get_net(&comp.name, "C");
                let d = get_net(&comp.name, "D");
                let e = get_net(&comp.name, "E");
                let f = get_net(&comp.name, "F");
                let g = get_net(&comp.name, "G");
                spice_lines.push(format!(
                    "X{} {} {} {} {} {} {} {} {} {}",
                    comp.name, a, b, c, d, e, f, g, com, comp.value_str
                ));
            }
            ComponentKind::Buzzer => {
                let n1 = get_net(&comp.name, "+");
                let n2 = get_net(&comp.name, "-");
                graph.add_resistor(&comp.name, &n1, &n2, 100.0).map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} 100", comp.name, n1, n2));
            }

            // Sensors & Transducers
            ComponentKind::StrainGauge => {
                let p_pos = get_net(&comp.name, "P+");
                let p_neg = get_net(&comp.name, "P-");
                graph
                    .add_resistor(&comp.name, &p_pos, &p_neg, 350.0)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} 350", comp.name, p_pos, p_neg));
            }
            ComponentKind::TactileMatrix => {
                let r_pos = get_net(&comp.name, "R+");
                let r_neg = get_net(&comp.name, "R-");
                let c_pos = get_net(&comp.name, "C+");
                let c_neg = get_net(&comp.name, "C-");
                let rr_name = format!("{}_RR", comp.name);
                let rc_name = format!("{}_RC", comp.name);
                graph
                    .add_resistor(&rr_name, &r_pos, &r_neg, 1000.0)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rc_name, &c_pos, &c_neg, 1000.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("TACTILE_8X8".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {}",
                    comp.name, r_pos, r_neg, c_pos, c_neg, comp.value_str
                ));
            }
            ComponentKind::Imu9Dof => {
                let vdd = get_net(&comp.name, "VDD");
                let gnd = get_net(&comp.name, "GND");
                let scl = get_net(&comp.name, "SCL");
                let sda = get_net(&comp.name, "SDA");
                let cdec_name = format!("{}_CDEC", comp.name);
                let rp1_name = format!("{}_RP1", comp.name);
                let rp2_name = format!("{}_RP2", comp.name);
                graph
                    .add_capacitor(&cdec_name, &vdd, &gnd, 1e-7, None)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rp1_name, &scl, &vdd, 1e4)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rp2_name, &sda, &vdd, 1e4)
                    .map_err(|e| e.to_string())?;
                subckts.insert("IMU_6DOF_9DOF".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {}",
                    comp.name, vdd, gnd, scl, sda, comp.value_str
                ));
            }

            // Topological & Quantum Metamaterials
            ComponentKind::SawIdt => {
                let in_plus = get_net(&comp.name, "IN+");
                let in_minus = get_net(&comp.name, "IN-");
                let out_plus = get_net(&comp.name, "OUT+");
                let out_minus = get_net(&comp.name, "OUT-");
                let rin_name = format!("{}_RIN", comp.name);
                let rout_name = format!("{}_ROUT", comp.name);
                graph
                    .add_resistor(&rin_name, &in_plus, &in_minus, 50.0)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rout_name, &out_plus, &out_minus, 50.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("SAW_1GHZ".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {}",
                    comp.name, in_plus, in_minus, out_plus, out_minus, comp.value_str
                ));
            }
            ComponentKind::MajoranaJunction => {
                let j1 = get_net(&comp.name, "J1");
                let j2 = get_net(&comp.name, "J2");
                let j3 = get_net(&comp.name, "J3");
                let r1_name = format!("{}_R1", comp.name);
                let r2_name = format!("{}_R2", comp.name);
                graph
                    .add_resistor(&r1_name, &j1, &j2, 100.0)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&r2_name, &j2, &j3, 100.0)
                    .map_err(|e| e.to_string())?;
                subckts.insert("TOPOMAJ_1".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {}",
                    comp.name, j1, j2, j3, comp.value_str
                ));
            }
            ComponentKind::ParafermionicCavity => {
                let port1 = get_net(&comp.name, "PORT1");
                let port2 = get_net(&comp.name, "PORT2");
                let l_name = format!("{}_L", comp.name);
                let c_name = format!("{}_C", comp.name);
                graph
                    .add_inductor(&l_name, &port1, &port2, 1e-9, None)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_capacitor(&c_name, &port1, &port2, 1e-12, None)
                    .map_err(|e| e.to_string())?;
                subckts.insert("PARAFERM_RES".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {}",
                    comp.name, port1, port2, comp.value_str
                ));
            }
            ComponentKind::SkyrmionRouter => {
                let in_node = get_net(&comp.name, "IN");
                let ch0 = get_net(&comp.name, "CH0");
                let ch1 = get_net(&comp.name, "CH1");
                let gate = get_net(&comp.name, "GATE");
                let r0_name = format!("{}_R0", comp.name);
                let r1_name = format!("{}_R1", comp.name);
                let rg_name = format!("{}_RG", comp.name);
                graph
                    .add_resistor(&r0_name, &in_node, &ch0, 50.0)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&r1_name, &in_node, &ch1, 50.0)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&rg_name, &gate, "0", 1e6)
                    .map_err(|e| e.to_string())?;
                subckts.insert("SKYRMION_RT".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {}",
                    comp.name, in_node, ch0, ch1, gate, comp.value_str
                ));
            }

            // Port-Hamiltonian Articulatory Acoustics
            ComponentKind::PhLungs => {
                let p_sub = get_net(&comp.name, "P_SUB");
                let p_ref = get_net(&comp.name, "REF");
                let v_name = format!("{}_VPL", comp.name);
                let r_name = format!("{}_RL", comp.name);
                graph
                    .add_voltage_source(&v_name, &p_sub, &p_ref, 800.0)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&r_name, &p_sub, &p_ref, 1e5)
                    .map_err(|e| e.to_string())?;
                subckts.insert("PH_LUNGS".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {}",
                    comp.name, p_sub, p_ref, comp.value_str
                ));
            }
            ComponentKind::PhVocalFolds => {
                let sub = get_net(&comp.name, "SUB");
                let supra = get_net(&comp.name, "SUPRA");
                let ctrl = get_net(&comp.name, "CTRL");
                let p_ref = get_net(&comp.name, "REF");
                let r_glot = format!("{}_RGLOT", comp.name);
                let r_ctrl = format!("{}_RCTRL", comp.name);
                graph
                    .add_resistor(&r_glot, &sub, &supra, 1000.0)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&r_ctrl, &ctrl, &p_ref, 1e6)
                    .map_err(|e| e.to_string())?;
                subckts.insert("PH_VOCAL_FOLDS".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {}",
                    comp.name, sub, supra, ctrl, p_ref, comp.value_str
                ));
            }
            ComponentKind::PhVocalTract => {
                let in_node = get_net(&comp.name, "IN");
                let out_node = get_net(&comp.name, "OUT");
                let wall = get_net(&comp.name, "WALL");
                let ctrl = get_net(&comp.name, "CTRL");
                let r_tract = format!("{}_RTRACT", comp.name);
                let r_wall = format!("{}_RWALL", comp.name);
                let r_ctrl = format!("{}_RCTRL", comp.name);
                graph
                    .add_resistor(&r_tract, &in_node, &out_node, 120.0)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&r_wall, &in_node, &wall, 1e5)
                    .map_err(|e| e.to_string())?;
                graph
                    .add_resistor(&r_ctrl, &ctrl, "0", 1e7)
                    .map_err(|e| e.to_string())?;
                subckts.insert("PH_VOCAL_TRACT".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {} {} {}",
                    comp.name, in_node, out_node, wall, ctrl, comp.value_str
                ));
            }
            ComponentKind::PhLipRadiation => {
                let in_node = get_net(&comp.name, "IN");
                let rad = get_net(&comp.name, "RAD");
                let r_rad = format!("{}_RRAD", comp.name);
                graph
                    .add_resistor(&r_rad, &in_node, &rad, 1.5e6)
                    .map_err(|e| e.to_string())?;
                subckts.insert("PH_LIP_RADIATION".to_string());
                spice_lines.push(format!(
                    "X{} {} {} {}",
                    comp.name, in_node, rad, comp.value_str
                ));
            }
        }
    }

    // Append subcircuit templates
    for subckt in &subckts {
        match subckt.as_str() {
            "FINFET_3NM" => {
                spice_lines.push(".SUBCKT FINFET_3NM D G S B".to_string());
                spice_lines.push("M1 D G S B NMOS_MOD W=45n L=3n".to_string());
                spice_lines.push(".ENDS FINFET_3NM".to_string());
            }
            "GAA_2NM" => {
                spice_lines.push(".SUBCKT GAA_2NM D G S B".to_string());
                spice_lines.push("M1 D G S B NMOS_MOD W=60n L=2n".to_string());
                spice_lines.push(".ENDS GAA_2NM".to_string());
            }
            "OPAMP_IDEAL" => {
                spice_lines.push(".SUBCKT OPAMP_IDEAL INP INN VP VN OUT".to_string());
                spice_lines.push("RIN INP INN 10MEG".to_string());
                spice_lines.push("E1 OUT VN INP INN 100000".to_string());
                spice_lines.push("ROUT OUT VN 50".to_string());
                spice_lines.push(".ENDS OPAMP_IDEAL".to_string());
            }
            "INV_CMOS" => {
                spice_lines.push(".SUBCKT INV_CMOS IN OUT".to_string());
                spice_lines.push("RIN IN 0 10MEG".to_string());
                spice_lines.push("E1 OUT 0 IN 0 -1.0".to_string());
                spice_lines.push("ROUT OUT 0 100".to_string());
                spice_lines.push(".ENDS INV_CMOS".to_string());
            }
            "NAND2" => {
                spice_lines.push(".SUBCKT NAND2 A B OUT".to_string());
                spice_lines.push("RA A 0 10MEG".to_string());
                spice_lines.push("RB B 0 10MEG".to_string());
                spice_lines.push("ROUT OUT 0 100".to_string());
                spice_lines.push(".ENDS NAND2".to_string());
            }
            "NOR2" => {
                spice_lines.push(".SUBCKT NOR2 A B OUT".to_string());
                spice_lines.push("RA A 0 10MEG".to_string());
                spice_lines.push("RB B 0 10MEG".to_string());
                spice_lines.push("ROUT OUT 0 100".to_string());
                spice_lines.push(".ENDS NOR2".to_string());
            }
            "MUX21" => {
                spice_lines.push(".SUBCKT MUX21 D0 D1 SEL OUT".to_string());
                spice_lines.push("R0 D0 0 10MEG".to_string());
                spice_lines.push("R1 D1 0 10MEG".to_string());
                spice_lines.push("RS SEL 0 10MEG".to_string());
                spice_lines.push("ROUT OUT 0 100".to_string());
                spice_lines.push(".ENDS MUX21".to_string());
            }
            "BUF_CMOS" => {
                spice_lines.push(".SUBCKT BUF_CMOS IN OUT".to_string());
                spice_lines.push("RIN IN 0 10MEG".to_string());
                spice_lines.push("E1 OUT 0 IN 0 1.0".to_string());
                spice_lines.push("ROUT OUT 0 100".to_string());
                spice_lines.push(".ENDS BUF_CMOS".to_string());
            }
            "AND2" => {
                spice_lines.push(".SUBCKT AND2 A B OUT".to_string());
                spice_lines.push("RA A 0 10MEG".to_string());
                spice_lines.push("RB B 0 10MEG".to_string());
                spice_lines.push("ROUT OUT 0 100".to_string());
                spice_lines.push(".ENDS AND2".to_string());
            }
            "OR2" => {
                spice_lines.push(".SUBCKT OR2 A B OUT".to_string());
                spice_lines.push("RA A 0 10MEG".to_string());
                spice_lines.push("RB B 0 10MEG".to_string());
                spice_lines.push("ROUT OUT 0 100".to_string());
                spice_lines.push(".ENDS OR2".to_string());
            }
            "XOR2" => {
                spice_lines.push(".SUBCKT XOR2 A B OUT".to_string());
                spice_lines.push("RA A 0 10MEG".to_string());
                spice_lines.push("RB B 0 10MEG".to_string());
                spice_lines.push("ROUT OUT 0 100".to_string());
                spice_lines.push(".ENDS XOR2".to_string());
            }
            "XNOR2" => {
                spice_lines.push(".SUBCKT XNOR2 A B OUT".to_string());
                spice_lines.push("RA A 0 10MEG".to_string());
                spice_lines.push("RB B 0 10MEG".to_string());
                spice_lines.push("ROUT OUT 0 100".to_string());
                spice_lines.push(".ENDS XNOR2".to_string());
            }
            "HALF_ADDER" => {
                spice_lines.push(".SUBCKT HALF_ADDER A B SUM COUT".to_string());
                spice_lines.push("RA A 0 10MEG".to_string());
                spice_lines.push("RB B 0 10MEG".to_string());
                spice_lines.push("RSUM SUM 0 100".to_string());
                spice_lines.push("RCOUT COUT 0 100".to_string());
                spice_lines.push(".ENDS HALF_ADDER".to_string());
            }
            "FULL_ADDER" => {
                spice_lines.push(".SUBCKT FULL_ADDER A B CIN SUM COUT".to_string());
                spice_lines.push("RA A 0 10MEG".to_string());
                spice_lines.push("RB B 0 10MEG".to_string());
                spice_lines.push("RCIN CIN 0 10MEG".to_string());
                spice_lines.push("RSUM SUM 0 100".to_string());
                spice_lines.push("RCOUT COUT 0 100".to_string());
                spice_lines.push(".ENDS FULL_ADDER".to_string());
            }
            "MUX41" => {
                spice_lines.push(".SUBCKT MUX41 D0 D1 D2 D3 S0 S1 OUT".to_string());
                spice_lines.push("R0 D0 0 10MEG".to_string());
                spice_lines.push("R1 D1 0 10MEG".to_string());
                spice_lines.push("R2 D2 0 10MEG".to_string());
                spice_lines.push("R3 D3 0 10MEG".to_string());
                spice_lines.push("RS0 S0 0 10MEG".to_string());
                spice_lines.push("RS1 S1 0 10MEG".to_string());
                spice_lines.push("ROUT OUT 0 100".to_string());
                spice_lines.push(".ENDS MUX41".to_string());
            }
            "DEMUX12" => {
                spice_lines.push(".SUBCKT DEMUX12 IN SEL Y0 Y1".to_string());
                spice_lines.push("RIN IN 0 10MEG".to_string());
                spice_lines.push("RSEL SEL 0 10MEG".to_string());
                spice_lines.push("RY0 Y0 0 100".to_string());
                spice_lines.push("RY1 Y1 0 100".to_string());
                spice_lines.push(".ENDS DEMUX12".to_string());
            }
            "DFF" => {
                spice_lines.push(".SUBCKT DFF D CLK Q QN".to_string());
                spice_lines.push("RD D 0 10MEG".to_string());
                spice_lines.push("RCLK CLK 0 10MEG".to_string());
                spice_lines.push("RQ Q 0 100".to_string());
                spice_lines.push("RQN QN 0 100".to_string());
                spice_lines.push(".ENDS DFF".to_string());
            }
            "SRLATCH" => {
                spice_lines.push(".SUBCKT SRLATCH S R Q QN".to_string());
                spice_lines.push("RS S 0 10MEG".to_string());
                spice_lines.push("RR R 0 10MEG".to_string());
                spice_lines.push("RQ Q 0 100".to_string());
                spice_lines.push("RQN QN 0 100".to_string());
                spice_lines.push(".ENDS SRLATCH".to_string());
            }
            "COUNTER4" => {
                spice_lines.push(".SUBCKT COUNTER4 CLK RST Q0 Q1 Q2 Q3".to_string());
                spice_lines.push("RCLK CLK 0 10MEG".to_string());
                spice_lines.push("RRST RST 0 10MEG".to_string());
                spice_lines.push("RQ0 Q0 0 100".to_string());
                spice_lines.push("RQ1 Q1 0 100".to_string());
                spice_lines.push("RQ2 Q2 0 100".to_string());
                spice_lines.push("RQ3 Q3 0 100".to_string());
                spice_lines.push(".ENDS COUNTER4".to_string());
            }
            "COMPARATOR" => {
                spice_lines.push(".SUBCKT COMPARATOR INP INN OUT".to_string());
                spice_lines.push("RIN INP INN 10MEG".to_string());
                spice_lines.push("E1 OUT 0 INP INN 10000".to_string());
                spice_lines.push("ROUT OUT 0 50".to_string());
                spice_lines.push(".ENDS COMPARATOR".to_string());
            }
            "LM555" => {
                spice_lines.push(".SUBCKT LM555 GND TRIG OUT RESET CTRL THRES DISCH VCC".to_string());
                spice_lines.push("R1 VCC CTRL 5K".to_string());
                spice_lines.push("R2 CTRL THRES 5K".to_string());
                spice_lines.push("R3 THRES GND 5K".to_string());
                spice_lines.push("ROUT OUT GND 50".to_string());
                spice_lines.push(".ENDS LM555".to_string());
            }
            "LM7805" => {
                spice_lines.push(".SUBCKT LM7805 VIN GND VOUT".to_string());
                spice_lines.push("RVIN VIN GND 100K".to_string());
                spice_lines.push("VOUT_SRC VOUT GND 5.0".to_string());
                spice_lines.push(".ENDS LM7805".to_string());
            }
            "7SEG_CC" => {
                spice_lines.push(".SUBCKT 7SEG_CC A B C D E F G COM".to_string());
                spice_lines.push("RA A COM 220".to_string());
                spice_lines.push("RB B COM 220".to_string());
                spice_lines.push("RC C COM 220".to_string());
                spice_lines.push("RD D COM 220".to_string());
                spice_lines.push("RE E COM 220".to_string());
                spice_lines.push("RF F COM 220".to_string());
                spice_lines.push("RG G COM 220".to_string());
                spice_lines.push(".ENDS 7SEG_CC".to_string());
            }
            "TACTILE_8X8" => {
                spice_lines.push(".SUBCKT TACTILE_8X8 RP RM CP CM".to_string());
                spice_lines.push("RR RP RM 1K".to_string());
                spice_lines.push("RC CP CM 1K".to_string());
                spice_lines.push(".ENDS TACTILE_8X8".to_string());
            }
            "IMU_6DOF_9DOF" => {
                spice_lines.push(".SUBCKT IMU_6DOF_9DOF VDD GND SCL SDA".to_string());
                spice_lines.push("CDEC VDD GND 100n".to_string());
                spice_lines.push("RP1 SCL VDD 10K".to_string());
                spice_lines.push("RP2 SDA VDD 10K".to_string());
                spice_lines.push(".ENDS IMU_6DOF_9DOF".to_string());
            }
            "SAW_1GHZ" => {
                spice_lines.push(".SUBCKT SAW_1GHZ INP INM OUTP OUTM".to_string());
                spice_lines.push("RIN INP INM 50".to_string());
                spice_lines.push("ROUT OUTP OUTM 50".to_string());
                spice_lines.push(".ENDS SAW_1GHZ".to_string());
            }
            "TOPOMAJ_1" => {
                spice_lines.push(".SUBCKT TOPOMAJ_1 J1 J2 J3".to_string());
                spice_lines.push("R1 J1 J2 100".to_string());
                spice_lines.push("R2 J2 J3 100".to_string());
                spice_lines.push(".ENDS TOPOMAJ_1".to_string());
            }
            "PARAFERM_RES" => {
                spice_lines.push(".SUBCKT PARAFERM_RES P1 P2".to_string());
                spice_lines.push("L1 P1 P2 1n".to_string());
                spice_lines.push("C1 P1 P2 1p".to_string());
                spice_lines.push(".ENDS PARAFERM_RES".to_string());
            }
            "SKYRMION_RT" => {
                spice_lines.push(".SUBCKT SKYRMION_RT IN CH0 CH1 GATE".to_string());
                spice_lines.push("R0 IN CH0 50".to_string());
                spice_lines.push("R1 IN CH1 50".to_string());
                spice_lines.push("RG GATE 0 1MEG".to_string());
                spice_lines.push(".ENDS SKYRMION_RT".to_string());
            }
            "PH_LUNGS" => {
                spice_lines.push(".SUBCKT PH_LUNGS PSUB REF".to_string());
                spice_lines.push("VPL PSUB INT 800.0".to_string());
                spice_lines.push("CLUNG INT REF 1.2u".to_string());
                spice_lines.push("RLUNG PSUB REF 100K".to_string());
                spice_lines.push(".ENDS PH_LUNGS".to_string());
            }
            "PH_VOCAL_FOLDS" => {
                spice_lines.push(".SUBCKT PH_VOCAL_FOLDS SUB SUPRA CTRL REF".to_string());
                spice_lines.push("RGLOT SUB SUPRA 1K".to_string());
                spice_lines.push("CVF CTRL REF 100p".to_string());
                spice_lines.push("RVF CTRL REF 1MEG".to_string());
                spice_lines.push(".ENDS PH_VOCAL_FOLDS".to_string());
            }
            "PH_VOCAL_TRACT" => {
                spice_lines.push(".SUBCKT PH_VOCAL_TRACT IN OUT WALL CTRL".to_string());
                spice_lines.push("LTRACT IN N1 25m".to_string());
                spice_lines.push("CTRACT N1 WALL 40n".to_string());
                spice_lines.push("RTRACT N1 OUT 120".to_string());
                spice_lines.push("RCTRL CTRL 0 10MEG".to_string());
                spice_lines.push(".ENDS PH_VOCAL_TRACT".to_string());
            }
            "PH_LIP_RADIATION" => {
                spice_lines.push(".SUBCKT PH_LIP_RADIATION IN RAD".to_string());
                spice_lines.push("RRAD IN RAD 1.5MEG".to_string());
                spice_lines.push("LRAD IN RAD 85".to_string());
                spice_lines.push(".ENDS PH_LIP_RADIATION".to_string());
            }
            _ => {}
        }
    }

    spice_lines.push(".OP".to_string());
    spice_lines.push(".END".to_string());

    // Map each wire to its resolved net
    let mut wire_to_net = HashMap::new();
    for wire in wires {
        let p_start = quantize(wire.start_point());
        let root = dsu.find(p_start);
        if let Some(net) = root_to_net.get(&root) {
            wire_to_net.insert(wire.id, net.clone());
        } else {
            let p_end = quantize(wire.end_point());
            let root_end = dsu.find(p_end);
            if let Some(net) = root_to_net.get(&root_end) {
                wire_to_net.insert(wire.id, net.clone());
            }
        }
    }

    Ok(CompiledCircuit {
        graph,
        model_ctx,
        pin_to_net,
        wire_to_net,
        net_names: all_nets,
        spice_netlist: spice_lines.join("\n"),
    })
}

/// Computes DC voltages and branch currents flowing through wires.
pub fn compute_wire_telemetry(
    components: &[SchematicComponent],
    wires: &[SchematicWire],
    compiled: &CompiledCircuit,
    node_voltages: &HashMap<String, f64>,
) -> (HashMap<usize, f64>, HashMap<usize, f64>) {
    let mut wire_voltages = HashMap::new();
    let mut wire_currents = HashMap::new();

    // 1. Calculate wire voltages from net mapping
    for wire in wires {
        let net = compiled.wire_to_net.get(&wire.id);
        let v = net.and_then(|n| node_voltages.get(n)).copied().unwrap_or(0.0);
        wire_voltages.insert(wire.id, v);
    }

    // 2. Map component terminal currents to connected pins
    let mut pin_currents: HashMap<(String, String), f64> = HashMap::new();

    for comp in components {
        let pins = comp.all_pins();
        if pins.len() >= 2 {
            let n1 = compiled
                .pin_to_net
                .get(&(comp.name.clone(), pins[0].0.to_string()))
                .cloned()
                .unwrap_or_else(|| "0".to_string());
            let n2 = compiled
                .pin_to_net
                .get(&(comp.name.clone(), pins[1].0.to_string()))
                .cloned()
                .unwrap_or_else(|| "0".to_string());
            let v1 = node_voltages.get(&n1).copied().unwrap_or(0.0);
            let v2 = node_voltages.get(&n2).copied().unwrap_or(0.0);
            let vdrop = v1 - v2;

            let current = match comp.kind {
                ComponentKind::Resistor => {
                    let r = parse_spice_number(&comp.value_str, 1).unwrap_or(1000.0);
                    vdrop / r.max(1e-9)
                }
                ComponentKind::Diode => {
                    let vt = 0.026;
                    let is_sat = 1.0e-14;
                    if vdrop > 0.0 {
                        is_sat * ((vdrop / vt).min(40.0).exp() - 1.0)
                    } else {
                        -is_sat
                    }
                }
                ComponentKind::VoltageSource => {
                    let r_est = 1000.0;
                    vdrop.abs() / r_est
                }
                ComponentKind::CurrentSource => {
                    parse_spice_number(&comp.value_str, 1).unwrap_or(1e-3)
                }
                _ => 1.0e-6,
            };

            pin_currents.insert((comp.name.clone(), pins[0].0.to_string()), current);
            pin_currents.insert((comp.name.clone(), pins[1].0.to_string()), -current);
        }
    }

    // 3. Associate wire currents from connected pin terminals
    for wire in wires {
        let mut total_curr = 0.0f64;
        let p_start = wire.start_point();
        let p_end = wire.end_point();

        for comp in components {
            for (pname, pos) in comp.all_pins() {
                let d_start = (pos - p_start).length();
                let d_end = (pos - p_end).length();
                if d_start < 6.0 || d_end < 6.0 {
                    if let Some(&i) = pin_currents.get(&(comp.name.clone(), pname.to_string())) {
                        total_curr += i.abs();
                    }
                }
            }
        }

        wire_currents.insert(wire.id, total_curr);
    }

    (wire_voltages, wire_currents)
}

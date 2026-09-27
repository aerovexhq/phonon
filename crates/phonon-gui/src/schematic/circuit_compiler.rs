//! Translates visual schematic components and wires into a solvable CircuitGraph and SPICE netlist.

use super::components::{ComponentKind, SchematicComponent};
use super::wire::SchematicWire;
use egui::Pos2;
use phonon_core::CircuitGraph;
use phonon_models::bjt::BjtType;
use phonon_models::mosfet::MosfetType;
use phonon_models::{BjtModel, DiodeModel, MosfetModel};
use phonon_netlist::parse_spice_number;
use phonon_solver::mna::ModelContext;
use std::collections::HashMap;

/// Compilation results produced from the visual schematic editor.
#[derive(Debug, Clone)]
pub struct CompiledCircuit {
    pub graph: CircuitGraph,
    pub model_ctx: ModelContext,
    /// Maps `(component_name, pin_name)` -> `net_name` (e.g. `("R1", "1") -> "net1"`).
    pub pin_to_net: HashMap<(String, String), String>,
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
    let mut dsu = DisjointSet::new();

    // 1. Union wire segment endpoints
    for wire in wires {
        for seg in &wire.segments {
            let p1 = quantize(seg.start);
            let p2 = quantize(seg.end);
            dsu.union(p1, p2);
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

    // 3. Identify all points associated with Ground
    let mut ground_roots = Vec::new();
    for comp in components {
        if comp.kind == ComponentKind::Ground {
            let pin_pos = comp.pin_world_pos(0).unwrap_or(comp.pos);
            let root = dsu.find(quantize(pin_pos));
            ground_roots.push(root);
        }
    }

    // 4. Assign human-readable net names to root clusters
    let mut root_to_net: HashMap<(i32, i32), String> = HashMap::new();
    for &gr in &ground_roots {
        root_to_net.insert(gr, "0".to_string());
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
                all_nets.push(name.clone());
                name
            });
            pin_to_net.insert((comp.name.clone(), pin_name.to_string()), net_name.clone());
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

    for comp in components {
        match comp.kind {
            ComponentKind::Ground => {}
            ComponentKind::Resistor => {
                let n1 = pin_to_net
                    .get(&(comp.name.clone(), "1".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let n2 = pin_to_net
                    .get(&(comp.name.clone(), "2".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let val = parse_spice_number(&comp.value_str, 1).unwrap_or(1000.0);
                graph
                    .add_resistor(&comp.name, &n1, &n2, val)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
            }
            ComponentKind::Capacitor => {
                let n1 = pin_to_net
                    .get(&(comp.name.clone(), "1".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let n2 = pin_to_net
                    .get(&(comp.name.clone(), "2".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let val = parse_spice_number(&comp.value_str, 1).unwrap_or(1e-7);
                graph
                    .add_capacitor(&comp.name, &n1, &n2, val, None)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
            }
            ComponentKind::Inductor => {
                let n1 = pin_to_net
                    .get(&(comp.name.clone(), "1".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let n2 = pin_to_net
                    .get(&(comp.name.clone(), "2".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let val = parse_spice_number(&comp.value_str, 1).unwrap_or(1e-5);
                graph
                    .add_inductor(&comp.name, &n1, &n2, val, None)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
            }
            ComponentKind::VoltageSource => {
                let n1 = pin_to_net
                    .get(&(comp.name.clone(), "+".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let n2 = pin_to_net
                    .get(&(comp.name.clone(), "-".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let val = parse_spice_number(&comp.value_str, 1).unwrap_or(5.0);
                graph
                    .add_voltage_source(&comp.name, &n1, &n2, val)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
            }
            ComponentKind::CurrentSource => {
                let n1 = pin_to_net
                    .get(&(comp.name.clone(), "+".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let n2 = pin_to_net
                    .get(&(comp.name.clone(), "-".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let val = parse_spice_number(&comp.value_str, 1).unwrap_or(0.001);
                graph
                    .add_current_source(&comp.name, &n1, &n2, val)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
            }
            ComponentKind::Diode => {
                let n1 = pin_to_net
                    .get(&(comp.name.clone(), "A".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let n2 = pin_to_net
                    .get(&(comp.name.clone(), "K".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let model = DiodeModel::default();
                model_ctx.set_diode_model(&comp.name, model);
                graph
                    .add_diode(&comp.name, &n1, &n2)
                    .map_err(|e| e.to_string())?;
                spice_lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
                spice_lines.push(format!(".MODEL {} D (IS=1e-14 RS=0.1)", comp.value_str));
            }
            ComponentKind::Nmos => {
                let d = pin_to_net
                    .get(&(comp.name.clone(), "D".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let g = pin_to_net
                    .get(&(comp.name.clone(), "G".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let s = pin_to_net
                    .get(&(comp.name.clone(), "S".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let b = s.clone(); // Bulk defaults to source
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
                let d = pin_to_net
                    .get(&(comp.name.clone(), "D".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let g = pin_to_net
                    .get(&(comp.name.clone(), "G".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let s = pin_to_net
                    .get(&(comp.name.clone(), "S".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
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
            ComponentKind::BjtNpn => {
                let c = pin_to_net
                    .get(&(comp.name.clone(), "C".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let b = pin_to_net
                    .get(&(comp.name.clone(), "B".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let e = pin_to_net
                    .get(&(comp.name.clone(), "E".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
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
                let c = pin_to_net
                    .get(&(comp.name.clone(), "C".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let b = pin_to_net
                    .get(&(comp.name.clone(), "B".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
                let e = pin_to_net
                    .get(&(comp.name.clone(), "E".to_string()))
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
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
        }
    }

    spice_lines.push(".OP".to_string());
    spice_lines.push(".END".to_string());

    Ok(CompiledCircuit {
        graph,
        model_ctx,
        pin_to_net,
        net_names: all_nets,
        spice_netlist: spice_lines.join("\n"),
    })
}

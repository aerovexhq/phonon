#![deny(unsafe_code)]

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
use std::collections::{BTreeSet, HashMap};

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
            _ => {}
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

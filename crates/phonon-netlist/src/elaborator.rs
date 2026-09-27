//! Netlist elaboration, subcircuit hierarchical flattening, and model instantiation.

use crate::ast::*;
use crate::error::NetlistError;
use phonon_core::{CircuitGraph, DigitalNodeId};
use phonon_models::mixed_signal::{A2dBridge, D2aBridge};
use phonon_models::{bjt::BjtType, mosfet::MosfetType, BjtModel, DiodeModel, MosfetModel};
use phonon_solver::mna::ModelContext;
use std::collections::HashMap;

/// Simulation analysis plan specified by SPICE directives.
#[derive(Debug, Clone, PartialEq)]
pub enum SimulationPlan {
    /// Operating point DC solve (.OP)
    Op,
    /// DC sweep of a voltage or current source (.DC)
    Dc {
        source_name: String,
        start: f64,
        stop: f64,
        step: f64,
    },
    /// Transient time-domain simulation (.TRAN)
    Tran { tstep: f64, tstop: f64 },
}

/// Elaborated circuit ready for MNA matrix assembly and simulation.
#[derive(Debug, Clone)]
pub struct ElaboratedCircuit {
    pub title: String,
    pub graph: CircuitGraph,
    pub model_ctx: ModelContext,
    pub plan: SimulationPlan,
    pub a2d_bridges: Vec<A2dBridge>,
    pub d2a_bridges: Vec<D2aBridge>,
}

/// Elaborates a parsed SPICE netlist: flattens hierarchical subcircuits,
/// instantiates semiconductor models, and builds the `CircuitGraph` and `ModelContext`.
pub fn elaborate_netlist(netlist: &ParsedNetlist) -> Result<ElaboratedCircuit, NetlistError> {
    let mut model_ctx = ModelContext::new();

    // 1. Process .TEMP directive
    for dir in &netlist.directives {
        if let Directive::Temp { temp_c } = dir {
            model_ctx.temperature_kelvin = temp_c + 273.15;
        }
    }

    // 2. Parse and build base models from .MODEL cards
    let mut diode_models: HashMap<String, DiodeModel> = HashMap::new();
    let mut mosfet_models: HashMap<String, MosfetModel> = HashMap::new();
    let mut bjt_models: HashMap<String, BjtModel> = HashMap::new();

    for (name, card) in &netlist.models {
        let type_upper = card.model_type.to_ascii_uppercase();
        match type_upper.as_str() {
            "D" => {
                let mut d = DiodeModel::default();
                for (k, &v) in &card.params {
                    match k.to_ascii_uppercase().as_str() {
                        "IS" => d.is = v,
                        "N" => d.n = v,
                        "RS" => d.rs = v,
                        "CJO" | "CJ" => d.cj0 = v,
                        "VJ" => d.vj = v,
                        "M" => d.m = v,
                        "TT" => d.tt = v,
                        "BV" => d.bv = v,
                        "IBV" => d.ibv = v,
                        "XTI" => d.xti = v,
                        "EG" => d.eg = v,
                        "FC" => d.fc = v,
                        _ => {}
                    }
                }
                diode_models.insert(name.clone(), d);
                model_ctx.set_diode_model(name, d);
            }
            "NMOS" | "PMOS" => {
                let mos_type = if type_upper == "PMOS" {
                    MosfetType::Pmos
                } else {
                    MosfetType::Nmos
                };
                let mut m = MosfetModel {
                    mos_type,
                    ..MosfetModel::default()
                };
                for (k, &v) in &card.params {
                    match k.to_ascii_uppercase().as_str() {
                        "VTO" | "VTH0" => m.vth0 = v,
                        "W" => m.w = v,
                        "L" => m.l = v,
                        "TOX" => m.tox = v,
                        "UO" | "MU0" => m.mu0 = v,
                        "VSAT" => m.vsat = v,
                        "LAMBDA" => m.lambda = v,
                        "GAMMA" => m.gamma = v,
                        "PHI" => m.phi_s = v,
                        "ETA" | "ETADIBL" => m.eta_dibl = v,
                        "NFACTOR" | "SUBTH_N" => m.subthreshold_n = v,
                        _ => {}
                    }
                }
                mosfet_models.insert(name.clone(), m);
                model_ctx.set_mosfet_model(name, m);
            }
            "NPN" | "PNP" => {
                let bjt_type = if type_upper == "PNP" {
                    BjtType::Pnp
                } else {
                    BjtType::Npn
                };
                let mut b = BjtModel {
                    bjt_type,
                    ..BjtModel::default()
                };
                for (k, &v) in &card.params {
                    match k.to_ascii_uppercase().as_str() {
                        "IS" => b.is = v,
                        "BF" => b.bf = v,
                        "BR" => b.br = v,
                        "VAF" => b.vaf = v,
                        "VAR" => b.var = v,
                        "IKF" => b.ikf = v,
                        "IKR" => b.ikr = v,
                        "NF" => b.nf = v,
                        "NR" => b.nr = v,
                        _ => {}
                    }
                }
                bjt_models.insert(name.clone(), b);
                model_ctx.set_bjt_model(name, b);
            }
            _ => {}
        }
    }

    // 3. Flatten hierarchical subcircuits
    let pin_map = HashMap::new();
    let flattened_components =
        flatten_components(&netlist.components, &netlist.subcircuits, None, &pin_map, 0)?;

    // 4. Construct CircuitGraph and Mixed-Signal Bridges
    let mut graph = CircuitGraph::new();
    let mut a2d_bridges = Vec::new();
    let mut d2a_bridges = Vec::new();
    let mut dig_name_to_id: HashMap<String, DigitalNodeId> = HashMap::new();
    let mut get_or_create_dnode = |name: &str| -> DigitalNodeId {
        let next_id = dig_name_to_id.len() as u32;
        *dig_name_to_id
            .entry(name.to_string())
            .or_insert(DigitalNodeId(next_id))
    };

    for comp in flattened_components {
        match comp {
            ComponentAst::Resistor {
                name,
                pos,
                neg,
                resistance,
            } => {
                graph.add_resistor(&name, &pos, &neg, resistance)?;
            }
            ComponentAst::Capacitor {
                name,
                pos,
                neg,
                capacitance,
                initial_voltage,
            } => {
                graph.add_capacitor(&name, &pos, &neg, capacitance, initial_voltage)?;
            }
            ComponentAst::Inductor {
                name,
                pos,
                neg,
                inductance,
                initial_current,
            } => {
                graph.add_inductor(&name, &pos, &neg, inductance, initial_current)?;
            }
            ComponentAst::VoltageSource {
                name,
                pos,
                neg,
                dc_value,
            } => {
                graph.add_voltage_source(&name, &pos, &neg, dc_value)?;
            }
            ComponentAst::CurrentSource {
                name,
                pos,
                neg,
                dc_value,
            } => {
                graph.add_current_source(&name, &pos, &neg, dc_value)?;
            }
            ComponentAst::Vcvs {
                name,
                out_pos,
                out_neg,
                ctrl_pos,
                ctrl_neg,
                gain,
            } => {
                graph.add_vcvs(&name, &out_pos, &out_neg, &ctrl_pos, &ctrl_neg, gain)?;
            }
            ComponentAst::Vccs {
                name,
                out_pos,
                out_neg,
                ctrl_pos,
                ctrl_neg,
                transconductance,
            } => {
                graph.add_vccs(
                    &name,
                    &out_pos,
                    &out_neg,
                    &ctrl_pos,
                    &ctrl_neg,
                    transconductance,
                )?;
            }
            ComponentAst::Diode {
                name,
                pos,
                neg,
                model_name,
            } => {
                let model =
                    diode_models
                        .get(&model_name)
                        .ok_or_else(|| NetlistError::UndefinedModel {
                            line: 0,
                            component_name: name.clone(),
                            model_name: model_name.clone(),
                        })?;
                model_ctx.set_diode_model(&name, *model);
                graph.add_diode(&name, &pos, &neg)?;
            }
            ComponentAst::Mosfet {
                name,
                drain,
                gate,
                source,
                bulk,
                model_name,
                w,
                l,
            } => {
                let base_model =
                    mosfet_models
                        .get(&model_name)
                        .ok_or_else(|| NetlistError::UndefinedModel {
                            line: 0,
                            component_name: name.clone(),
                            model_name: model_name.clone(),
                        })?;
                let mut inst_model = *base_model;
                if let Some(width) = w {
                    inst_model.w = width;
                }
                if let Some(length) = l {
                    inst_model.l = length;
                }
                model_ctx.set_mosfet_model(&name, inst_model);
                graph.add_mosfet(&name, &drain, &gate, &source, &bulk)?;
            }
            ComponentAst::Bjt {
                name,
                collector,
                base,
                emitter,
                model_name,
            } => {
                let model =
                    bjt_models
                        .get(&model_name)
                        .ok_or_else(|| NetlistError::UndefinedModel {
                            line: 0,
                            component_name: name.clone(),
                            model_name: model_name.clone(),
                        })?;
                model_ctx.set_bjt_model(&name, *model);
                graph.add_bjt(&name, &collector, &base, &emitter)?;
            }
            ComponentAst::TransmissionLine {
                name,
                in_pos,
                in_neg,
                out_pos,
                out_neg,
                z0,
                td,
            } => {
                graph.add_transmission_line(&name, &in_pos, &in_neg, &out_pos, &out_neg, z0, td)?;
            }
            ComponentAst::A2dBridge {
                in_pos,
                in_neg,
                out_dig,
                vth_low,
                vth_high,
                r_in,
                ..
            } => {
                let p = graph.get_or_create_node(&in_pos);
                let n = graph.get_or_create_node(&in_neg);
                let d_id = get_or_create_dnode(&out_dig);
                let mut bridge = A2dBridge::new(p, n, d_id, vth_low, vth_high);
                if let Some(r) = r_in {
                    bridge = bridge.with_impedance(r, 0.0);
                }
                a2d_bridges.push(bridge);
            }
            ComponentAst::D2aBridge {
                in_dig,
                out_pos,
                out_neg,
                v_low,
                v_high,
                rise_time,
                fall_time,
                r_out,
                ..
            } => {
                let d_id = get_or_create_dnode(&in_dig);
                let p = graph.get_or_create_node(&out_pos);
                let n = graph.get_or_create_node(&out_neg);
                let bridge = D2aBridge::new(d_id, p, n, v_low, v_high)
                    .with_timings(rise_time, fall_time, r_out);
                d2a_bridges.push(bridge);
            }
            ComponentAst::SubcircuitInstance { name, .. } => {
                // Should have been completely expanded during flattening
                return Err(NetlistError::SyntaxError {
                    line: 0,
                    message: format!(
                        "Unresolved subcircuit instance '{}' during graph assembly",
                        name
                    ),
                });
            }
        }
    }

    // 5. Determine simulation plan
    let mut plan = SimulationPlan::Op;
    for dir in &netlist.directives {
        match dir {
            Directive::Op => {
                plan = SimulationPlan::Op;
            }
            Directive::Dc {
                source_name,
                start,
                stop,
                step,
            } => {
                plan = SimulationPlan::Dc {
                    source_name: source_name.clone(),
                    start: *start,
                    stop: *stop,
                    step: *step,
                };
            }
            Directive::Tran { tstep, tstop } => {
                plan = SimulationPlan::Tran {
                    tstep: *tstep,
                    tstop: *tstop,
                };
            }
            Directive::Temp { .. } => {}
        }
    }

    Ok(ElaboratedCircuit {
        title: netlist.title.clone(),
        graph,
        model_ctx,
        plan,
        a2d_bridges,
        d2a_bridges,
    })
}

/// Recursively flattens subcircuit instances into concrete primitives.
fn flatten_components(
    components: &[ComponentAst],
    subcircuits: &HashMap<String, SubcircuitDef>,
    prefix: Option<&str>,
    pin_map: &HashMap<String, String>,
    depth: usize,
) -> Result<Vec<ComponentAst>, NetlistError> {
    const MAX_RECURSION_DEPTH: usize = 64;
    let mut flattened = Vec::new();

    for comp in components {
        match comp {
            ComponentAst::SubcircuitInstance {
                name,
                pins,
                subcircuit_name,
            } => {
                if depth >= MAX_RECURSION_DEPTH {
                    return Err(NetlistError::RecursionDepthExceeded {
                        subcircuit_name: subcircuit_name.clone(),
                        max_depth: MAX_RECURSION_DEPTH,
                    });
                }

                let subckt_def = subcircuits
                    .get(subcircuit_name)
                    .or_else(|| subcircuits.get(&subcircuit_name.to_ascii_uppercase()))
                    .ok_or_else(|| NetlistError::UndefinedSubcircuit {
                        line: 0,
                        instance_name: name.clone(),
                        subcircuit_name: subcircuit_name.clone(),
                    })?;

                if pins.len() != subckt_def.pins.len() {
                    return Err(NetlistError::PinCountMismatch {
                        line: 0,
                        instance_name: name.clone(),
                        expected: subckt_def.pins.len(),
                        found: pins.len(),
                    });
                }

                let inst_prefix = match prefix {
                    Some(p) => format!("{p}.{name}"),
                    None => name.clone(),
                };

                let mut child_pin_map = HashMap::new();
                for (formal, actual) in subckt_def.pins.iter().zip(pins.iter()) {
                    let mapped_actual = remap_node(actual, prefix, pin_map);
                    child_pin_map.insert(formal.to_ascii_uppercase(), mapped_actual);
                }

                let expanded = flatten_components(
                    &subckt_def.components,
                    subcircuits,
                    Some(&inst_prefix),
                    &child_pin_map,
                    depth + 1,
                )?;
                flattened.extend(expanded);
            }
            ComponentAst::Resistor {
                name,
                pos,
                neg,
                resistance,
            } => {
                flattened.push(ComponentAst::Resistor {
                    name: rename_comp(name, prefix),
                    pos: remap_node(pos, prefix, pin_map),
                    neg: remap_node(neg, prefix, pin_map),
                    resistance: *resistance,
                });
            }
            ComponentAst::Capacitor {
                name,
                pos,
                neg,
                capacitance,
                initial_voltage,
            } => {
                flattened.push(ComponentAst::Capacitor {
                    name: rename_comp(name, prefix),
                    pos: remap_node(pos, prefix, pin_map),
                    neg: remap_node(neg, prefix, pin_map),
                    capacitance: *capacitance,
                    initial_voltage: *initial_voltage,
                });
            }
            ComponentAst::Inductor {
                name,
                pos,
                neg,
                inductance,
                initial_current,
            } => {
                flattened.push(ComponentAst::Inductor {
                    name: rename_comp(name, prefix),
                    pos: remap_node(pos, prefix, pin_map),
                    neg: remap_node(neg, prefix, pin_map),
                    inductance: *inductance,
                    initial_current: *initial_current,
                });
            }
            ComponentAst::VoltageSource {
                name,
                pos,
                neg,
                dc_value,
            } => {
                flattened.push(ComponentAst::VoltageSource {
                    name: rename_comp(name, prefix),
                    pos: remap_node(pos, prefix, pin_map),
                    neg: remap_node(neg, prefix, pin_map),
                    dc_value: *dc_value,
                });
            }
            ComponentAst::CurrentSource {
                name,
                pos,
                neg,
                dc_value,
            } => {
                flattened.push(ComponentAst::CurrentSource {
                    name: rename_comp(name, prefix),
                    pos: remap_node(pos, prefix, pin_map),
                    neg: remap_node(neg, prefix, pin_map),
                    dc_value: *dc_value,
                });
            }
            ComponentAst::Vcvs {
                name,
                out_pos,
                out_neg,
                ctrl_pos,
                ctrl_neg,
                gain,
            } => {
                flattened.push(ComponentAst::Vcvs {
                    name: rename_comp(name, prefix),
                    out_pos: remap_node(out_pos, prefix, pin_map),
                    out_neg: remap_node(out_neg, prefix, pin_map),
                    ctrl_pos: remap_node(ctrl_pos, prefix, pin_map),
                    ctrl_neg: remap_node(ctrl_neg, prefix, pin_map),
                    gain: *gain,
                });
            }
            ComponentAst::Vccs {
                name,
                out_pos,
                out_neg,
                ctrl_pos,
                ctrl_neg,
                transconductance,
            } => {
                flattened.push(ComponentAst::Vccs {
                    name: rename_comp(name, prefix),
                    out_pos: remap_node(out_pos, prefix, pin_map),
                    out_neg: remap_node(out_neg, prefix, pin_map),
                    ctrl_pos: remap_node(ctrl_pos, prefix, pin_map),
                    ctrl_neg: remap_node(ctrl_neg, prefix, pin_map),
                    transconductance: *transconductance,
                });
            }
            ComponentAst::Diode {
                name,
                pos,
                neg,
                model_name,
            } => {
                flattened.push(ComponentAst::Diode {
                    name: rename_comp(name, prefix),
                    pos: remap_node(pos, prefix, pin_map),
                    neg: remap_node(neg, prefix, pin_map),
                    model_name: model_name.clone(),
                });
            }
            ComponentAst::Mosfet {
                name,
                drain,
                gate,
                source,
                bulk,
                model_name,
                w,
                l,
            } => {
                flattened.push(ComponentAst::Mosfet {
                    name: rename_comp(name, prefix),
                    drain: remap_node(drain, prefix, pin_map),
                    gate: remap_node(gate, prefix, pin_map),
                    source: remap_node(source, prefix, pin_map),
                    bulk: remap_node(bulk, prefix, pin_map),
                    model_name: model_name.clone(),
                    w: *w,
                    l: *l,
                });
            }
            ComponentAst::Bjt {
                name,
                collector,
                base,
                emitter,
                model_name,
            } => {
                flattened.push(ComponentAst::Bjt {
                    name: rename_comp(name, prefix),
                    collector: remap_node(collector, prefix, pin_map),
                    base: remap_node(base, prefix, pin_map),
                    emitter: remap_node(emitter, prefix, pin_map),
                    model_name: model_name.clone(),
                });
            }
            ComponentAst::TransmissionLine {
                name,
                in_pos,
                in_neg,
                out_pos,
                out_neg,
                z0,
                td,
            } => {
                flattened.push(ComponentAst::TransmissionLine {
                    name: rename_comp(name, prefix),
                    in_pos: remap_node(in_pos, prefix, pin_map),
                    in_neg: remap_node(in_neg, prefix, pin_map),
                    out_pos: remap_node(out_pos, prefix, pin_map),
                    out_neg: remap_node(out_neg, prefix, pin_map),
                    z0: *z0,
                    td: *td,
                });
            }
            ComponentAst::A2dBridge {
                name,
                in_pos,
                in_neg,
                out_dig,
                vth_low,
                vth_high,
                r_in,
            } => {
                flattened.push(ComponentAst::A2dBridge {
                    name: rename_comp(name, prefix),
                    in_pos: remap_node(in_pos, prefix, pin_map),
                    in_neg: remap_node(in_neg, prefix, pin_map),
                    out_dig: remap_node(out_dig, prefix, pin_map),
                    vth_low: *vth_low,
                    vth_high: *vth_high,
                    r_in: *r_in,
                });
            }
            ComponentAst::D2aBridge {
                name,
                in_dig,
                out_pos,
                out_neg,
                v_low,
                v_high,
                rise_time,
                fall_time,
                r_out,
            } => {
                flattened.push(ComponentAst::D2aBridge {
                    name: rename_comp(name, prefix),
                    in_dig: remap_node(in_dig, prefix, pin_map),
                    out_pos: remap_node(out_pos, prefix, pin_map),
                    out_neg: remap_node(out_neg, prefix, pin_map),
                    v_low: *v_low,
                    v_high: *v_high,
                    rise_time: *rise_time,
                    fall_time: *fall_time,
                    r_out: *r_out,
                });
            }
        }
    }

    Ok(flattened)
}

fn rename_comp(name: &str, prefix: Option<&str>) -> String {
    match prefix {
        Some(p) => format!("{p}.{name}"),
        None => name.to_string(),
    }
}

fn remap_node(node: &str, prefix: Option<&str>, pin_map: &HashMap<String, String>) -> String {
    // SPICE ground is always global
    if node == "0" || node.eq_ignore_ascii_case("gnd") {
        return "0".to_string();
    }

    // Check formal pin mapping
    if let Some(mapped) = pin_map.get(&node.to_ascii_uppercase()) {
        return mapped.clone();
    }

    // If inside a subcircuit prefix, scope the internal node
    match prefix {
        Some(p) => format!("{p}.{node}"),
        None => node.to_string(),
    }
}

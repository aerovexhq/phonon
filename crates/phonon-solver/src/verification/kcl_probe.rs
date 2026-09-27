use crate::mna::{AtomisticCompanionModel, ModelContext};
use crate::transient::TransientStep;
use phonon_core::{AtomisticChannelType, CircuitGraph, ComponentRecord, NodeId};
use std::collections::HashMap;

/// Detailed results of a KCL nodal current conservation evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct KclReport {
    /// True if all nodal residuals satisfy conservation tolerance.
    pub is_valid: bool,
    /// Maximum absolute current residual across all circuit nodes ($L_\infty$ norm in Amperes).
    pub max_residual: f64,
    /// Root-mean-square current residual across all circuit nodes ($L_2$ norm in Amperes).
    pub rms_residual: f64,
    /// The node exhibiting the largest current conservation error.
    pub worst_node: NodeId,
    /// Residual value per active node.
    pub node_residuals: Vec<(NodeId, f64)>,
}

/// Evaluates dynamic algebraic Kirchhoff's Current Law residuals at all non-reference circuit nodes,
/// accounting for capacitive displacement currents, dependent sources, and semiconductor devices.
pub fn verify_kcl_dynamic(
    graph: &CircuitGraph,
    voltages: &[f64],
    branch_currents: &[f64],
    capacitor_currents: &HashMap<String, f64>,
    context: Option<&ModelContext>,
    reltol: f64,
    abstol: f64,
) -> KclReport {
    let active_nodes = graph.active_nodes();
    let mut residuals = vec![0.0; active_nodes + 1];
    let mut max_current = 0.0f64;

    let get_v = |node: NodeId| -> f64 {
        let idx = node.index();
        if idx == 0 || idx >= voltages.len() {
            0.0
        } else {
            voltages[idx]
        }
    };

    let add_current = |res: &mut [f64], node: NodeId, current_leaving: f64| {
        let idx = node.index();
        if idx > 0 && idx < res.len() {
            res[idx] += current_leaving;
        }
    };

    for comp in graph.components() {
        match comp {
            ComponentRecord::Resistor {
                pos,
                neg,
                resistance,
                ..
            } => {
                let v = get_v(*pos) - get_v(*neg);
                let i = v / resistance.max(1e-18);
                max_current = max_current.max(i.abs());
                add_current(&mut residuals, *pos, i);
                add_current(&mut residuals, *neg, -i);
            }
            ComponentRecord::VoltageSource {
                pos, neg, branch, ..
            } => {
                let i = branch_currents[branch.index()];
                max_current = max_current.max(i.abs());
                // Current leaves pos through the source and enters neg
                add_current(&mut residuals, *pos, i);
                add_current(&mut residuals, *neg, -i);
            }
            ComponentRecord::CurrentSource {
                pos, neg, dc_value, ..
            } => {
                let i = *dc_value;
                max_current = max_current.max(i.abs());
                add_current(&mut residuals, *pos, i);
                add_current(&mut residuals, *neg, -i);
            }
            ComponentRecord::Inductor {
                pos, neg, branch, ..
            } => {
                let i = branch_currents[branch.index()];
                max_current = max_current.max(i.abs());
                add_current(&mut residuals, *pos, i);
                add_current(&mut residuals, *neg, -i);
            }
            ComponentRecord::Capacitor { name, pos, neg, .. } => {
                if let Some(&i_c) = capacitor_currents.get(name) {
                    max_current = max_current.max(i_c.abs());
                    add_current(&mut residuals, *pos, i_c);
                    add_current(&mut residuals, *neg, -i_c);
                }
            }
            ComponentRecord::Vcvs {
                out_pos,
                out_neg,
                branch,
                ..
            } => {
                let i = branch_currents[branch.index()];
                max_current = max_current.max(i.abs());
                add_current(&mut residuals, *out_pos, i);
                add_current(&mut residuals, *out_neg, -i);
            }
            ComponentRecord::Vccs {
                out_pos,
                out_neg,
                ctrl_pos,
                ctrl_neg,
                transconductance,
                ..
            } => {
                let v_ctrl = get_v(*ctrl_pos) - get_v(*ctrl_neg);
                let i = transconductance * v_ctrl;
                max_current = max_current.max(i.abs());
                add_current(&mut residuals, *out_pos, i);
                add_current(&mut residuals, *out_neg, -i);
            }
            ComponentRecord::Diode { name, pos, neg } => {
                if let Some(ctx) = context {
                    let model = ctx.get_diode_model(name);
                    let v_d = get_v(*pos) - get_v(*neg);
                    let eval = model.evaluate(v_d, ctx.temperature_kelvin);
                    max_current = max_current.max(eval.i_d.abs());
                    add_current(&mut residuals, *pos, eval.i_d);
                    add_current(&mut residuals, *neg, -eval.i_d);
                }
            }
            ComponentRecord::Mosfet {
                name,
                drain,
                gate,
                source,
                bulk,
            } => {
                if let Some(ctx) = context {
                    let model = ctx.get_mosfet_model(name);
                    let v_d = get_v(*drain);
                    let v_g = get_v(*gate);
                    let v_s = get_v(*source);
                    let v_b = get_v(*bulk);
                    let eval = model.evaluate(v_d, v_g, v_s, v_b, ctx.temperature_kelvin);
                    max_current = max_current.max(eval.i_ds.abs());
                    add_current(&mut residuals, *drain, eval.i_ds);
                    add_current(&mut residuals, *source, -eval.i_ds);
                }
            }
            ComponentRecord::Bjt {
                name,
                collector,
                base,
                emitter,
            } => {
                if let Some(ctx) = context {
                    let model = ctx.get_bjt_model(name);
                    let v_c = get_v(*collector);
                    let v_b = get_v(*base);
                    let v_e = get_v(*emitter);
                    let eval = model.evaluate(v_c, v_b, v_e, ctx.temperature_kelvin);
                    max_current = max_current.max(eval.i_c.abs().max(eval.i_b.abs()));
                    add_current(&mut residuals, *collector, eval.i_c);
                    add_current(&mut residuals, *base, eval.i_b);
                    add_current(&mut residuals, *emitter, -(eval.i_c + eval.i_b));
                }
            }
            ComponentRecord::TransmissionLine {
                in_pos,
                in_neg,
                out_pos,
                out_neg,
                ..
            } => {
                let v_pos = get_v(*in_pos) - get_v(*out_pos);
                let i_pos = v_pos * 1e6;
                add_current(&mut residuals, *in_pos, i_pos);
                add_current(&mut residuals, *out_pos, -i_pos);

                let v_neg = get_v(*in_neg) - get_v(*out_neg);
                let i_neg = v_neg * 1e6;
                add_current(&mut residuals, *in_neg, i_neg);
                add_current(&mut residuals, *out_neg, -i_neg);
            }
            ComponentRecord::TcadDiode { name, pos, neg } => {
                if let Some(ctx) = context {
                    let fallback = phonon_models::TcadDeviceBuilder::new_pn_junction(name).build();
                    let tcad = ctx.get_tcad_device(name).unwrap_or(&fallback);
                    let vd = get_v(*pos) - get_v(*neg);
                    let (i_d, _) = tcad.evaluate_diode(vd, ctx.temperature_kelvin);
                    max_current = max_current.max(i_d.abs());
                    add_current(&mut residuals, *pos, i_d);
                    add_current(&mut residuals, *neg, -i_d);
                }
            }
            ComponentRecord::TcadMosfet {
                name,
                drain,
                gate,
                source,
                bulk,
            } => {
                if let Some(ctx) = context {
                    let fallback = phonon_models::TcadDeviceBuilder::new_mosfet(name).build();
                    let tcad = ctx.get_tcad_device(name).unwrap_or(&fallback);
                    let v_d = get_v(*drain);
                    let v_g = get_v(*gate);
                    let v_s = get_v(*source);
                    let v_b = get_v(*bulk);
                    let (i_ds, _, _, _) = tcad.evaluate_mosfet(
                        v_d - v_s,
                        v_g - v_s,
                        v_b - v_s,
                        ctx.temperature_kelvin,
                    );
                    max_current = max_current.max(i_ds.abs());
                    add_current(&mut residuals, *drain, i_ds);
                    add_current(&mut residuals, *source, -i_ds);
                }
            }
            ComponentRecord::NeuralSurrogate {
                name,
                nodes,
                surrogate_id,
            } => {
                if let Some(ctx) = context {
                    if let Some(surrogate) = ctx
                        .get_neural_surrogate(surrogate_id)
                        .or_else(|| ctx.get_neural_surrogate(name))
                    {
                        if nodes.len() == 2 {
                            let vd = get_v(nodes[0]) - get_v(nodes[1]);
                            let (i_d, _) = surrogate.evaluate_diode(vd);
                            max_current = max_current.max(i_d.abs());
                            add_current(&mut residuals, nodes[0], i_d);
                            add_current(&mut residuals, nodes[1], -i_d);
                        } else if nodes.len() >= 3 {
                            let v_d = get_v(nodes[0]);
                            let v_g = get_v(nodes[1]);
                            let v_s = get_v(nodes[2]);
                            let v_b = if nodes.len() > 3 {
                                get_v(nodes[3])
                            } else {
                                v_s
                            };
                            let (i_ds, _, _, _) =
                                surrogate.evaluate_transistor(v_d - v_s, v_g - v_s, v_b - v_s);
                            max_current = max_current.max(i_ds.abs());
                            add_current(&mut residuals, nodes[0], i_ds);
                            add_current(&mut residuals, nodes[2], -i_ds);
                        }
                    }
                }
            }
            ComponentRecord::JosephsonJunction {
                name,
                pos,
                neg,
                ic,
                rn,
                phase_init,
                ..
            } => {
                let v = get_v(*pos) - get_v(*neg);
                let (g_eq, i_eq) = if let Some(ctx) = context {
                    if let Some(jj) = ctx.get_josephson_junction(name) {
                        jj.dc_stamp(v)
                    } else {
                        let default_jj =
                            phonon_models::JosephsonRcsjModel::new(*ic, *rn, 1e-13, *phase_init);
                        default_jj.dc_stamp(v)
                    }
                } else {
                    let default_jj =
                        phonon_models::JosephsonRcsjModel::new(*ic, *rn, 1e-13, *phase_init);
                    default_jj.dc_stamp(v)
                };
                let i_jj = g_eq * v - i_eq;
                max_current = max_current.max(i_jj.abs());
                add_current(&mut residuals, *pos, i_jj);
                add_current(&mut residuals, *neg, -i_jj);
            }
            ComponentRecord::AtomisticChannel {
                name,
                drain,
                gate,
                source,
                channel_type,
            } => {
                let v_d = get_v(*drain);
                let v_g = get_v(*gate);
                let v_s = get_v(*source);
                let v_ds = v_d - v_s;
                let v_gs = v_g - v_s;
                let temp_k = context
                    .map(|c| c.temperature_kelvin)
                    .unwrap_or(phonon_core::T_REF);

                let ids = if let Some(ctx) = context {
                    if let Some(comp) = ctx.get_atomistic_channel(name) {
                        match comp {
                            AtomisticCompanionModel::Tmd(tmd) => tmd.evaluate_fet_current(
                                v_gs, v_ds, 0.35, 1e-6, 30e-9, 1.2e-9, temp_k,
                            ),
                            AtomisticCompanionModel::Cnt(cnt) => {
                                cnt.evaluate_fet_current(v_gs, v_ds, 0.4, 50e-9, temp_k)
                            }
                            AtomisticCompanionModel::Electromigration(em) => {
                                v_ds / em.initial_resistance_ohms.max(1e-12)
                            }
                        }
                    } else {
                        match channel_type {
                            AtomisticChannelType::TmdMonolayer {
                                species,
                                length_m,
                                width_m,
                            } => {
                                let tmd = if species.eq_ignore_ascii_case("ws2") {
                                    phonon_models::TmdMonolayer::ws2()
                                } else {
                                    phonon_models::TmdMonolayer::mos2()
                                };
                                tmd.evaluate_fet_current(
                                    v_gs, v_ds, 0.35, *width_m, *length_m, 1.2e-9, temp_k,
                                )
                            }
                            AtomisticChannelType::CarbonNanotube { n, m, length_m } => {
                                let cnt = phonon_models::CarbonNanotube::new(*n as u32, *m as u32);
                                cnt.evaluate_fet_current(v_gs, v_ds, 0.4, *length_m, temp_k)
                            }
                            AtomisticChannelType::InterconnectNanowire {
                                length_m,
                                width_m,
                                height_m,
                                ..
                            } => {
                                let r0: f64 =
                                    (1.68e-8 * length_m) / (width_m * height_m).max(1e-20);
                                v_ds / r0.max(1e-6)
                            }
                        }
                    }
                } else {
                    match channel_type {
                        AtomisticChannelType::TmdMonolayer {
                            species,
                            length_m,
                            width_m,
                        } => {
                            let tmd = if species.eq_ignore_ascii_case("ws2") {
                                phonon_models::TmdMonolayer::ws2()
                            } else {
                                phonon_models::TmdMonolayer::mos2()
                            };
                            tmd.evaluate_fet_current(
                                v_gs, v_ds, 0.35, *width_m, *length_m, 1.2e-9, temp_k,
                            )
                        }
                        AtomisticChannelType::CarbonNanotube { n, m, length_m } => {
                            let cnt = phonon_models::CarbonNanotube::new(*n as u32, *m as u32);
                            cnt.evaluate_fet_current(v_gs, v_ds, 0.4, *length_m, temp_k)
                        }
                        AtomisticChannelType::InterconnectNanowire {
                            length_m,
                            width_m,
                            height_m,
                            ..
                        } => {
                            let r0: f64 = (1.68e-8 * length_m) / (width_m * height_m).max(1e-20);
                            v_ds / r0.max(1e-6)
                        }
                    }
                };

                max_current = max_current.max(ids.abs());
                add_current(&mut residuals, *drain, ids);
                add_current(&mut residuals, *source, -ids);
            }
            ComponentRecord::OpticalWaveguide { .. }
            | ComponentRecord::MicroRingResonator { .. } => {
                // Optical passive components have zero electrical branch currents
            }
            ComponentRecord::ElectroOpticModulator {
                elec_pos, elec_neg, ..
            } => {
                let v_mod = get_v(*elec_pos) - get_v(*elec_neg);
                let i_leak = 1e-12 * v_mod;
                max_current = max_current.max(i_leak.abs());
                add_current(&mut residuals, *elec_pos, i_leak);
                add_current(&mut residuals, *elec_neg, -i_leak);
            }
            ComponentRecord::LaserDiode {
                name,
                anode,
                cathode,
                ..
            } => {
                let v_d = get_v(*anode) - get_v(*cathode);
                let temp_k = context
                    .map(|c| c.temperature_kelvin)
                    .unwrap_or(phonon_core::T_REF);
                let (i_d, _) = if let Some(ctx) = context {
                    if let Some(laser) = ctx.get_laser_diode(name) {
                        laser.electrical_companion(v_d, temp_k)
                    } else {
                        phonon_models::LaserDiodeModel::dfb_1550nm()
                            .electrical_companion(v_d, temp_k)
                    }
                } else {
                    phonon_models::LaserDiodeModel::dfb_1550nm().electrical_companion(v_d, temp_k)
                };
                max_current = max_current.max(i_d.abs());
                add_current(&mut residuals, *anode, i_d);
                add_current(&mut residuals, *cathode, -i_d);
            }
            ComponentRecord::Photodetector {
                name,
                opt_in,
                anode,
                cathode,
                ..
            } => {
                let v_anode = get_v(*anode);
                let v_cathode = get_v(*cathode);
                let v_rev = (v_cathode - v_anode).max(0.0);
                let temp_k = context
                    .map(|c| c.temperature_kelvin)
                    .unwrap_or(phonon_core::T_REF);
                let (detector, opt_sig) = if let Some(ctx) = context {
                    let pd = ctx
                        .get_photodetector(name)
                        .cloned()
                        .unwrap_or_else(phonon_models::PhotodetectorModel::ge_on_si_pin);
                    let sig = ctx
                        .get_optical_signal(*opt_in)
                        .copied()
                        .unwrap_or(phonon_core::OpticalSignal::new(1.55e-6, 0.0, 0.0));
                    (pd, sig)
                } else {
                    (
                        phonon_models::PhotodetectorModel::ge_on_si_pin(),
                        phonon_core::OpticalSignal::new(1.55e-6, 0.0, 0.0),
                    )
                };
                let i_tot = detector.total_current(&opt_sig, v_rev, temp_k);
                max_current = max_current.max(i_tot.abs());
                add_current(&mut residuals, *cathode, i_tot);
                add_current(&mut residuals, *anode, -i_tot);
            }
            ComponentRecord::Memristor {
                name,
                pos,
                neg,
                initial_conductance,
                ..
            } => {
                let v = get_v(*pos) - get_v(*neg);
                let i_mem = if let Some(ctx) = context {
                    if let Some(rram) = ctx.get_rram_model(name) {
                        let w = ctx
                            .get_memristor_state(name)
                            .map(|s| s.internal_state_w)
                            .unwrap_or(0.5);
                        rram.evaluate_current_and_conductance(w, v).0
                    } else if let Some(pcm) = ctx.get_pcm_model(name) {
                        let uc = ctx
                            .get_memristor_state(name)
                            .map(|s| s.internal_state_w)
                            .unwrap_or(0.5);
                        pcm.evaluate_current_and_conductance(uc, v).0
                    } else if let Some(st) = ctx.get_memristor_state(name) {
                        st.conductance_s * v
                    } else {
                        let g: f64 = if *initial_conductance > 0.0 {
                            *initial_conductance
                        } else {
                            1e-3
                        };
                        g * v
                    }
                } else {
                    let g: f64 = if *initial_conductance > 0.0 {
                        *initial_conductance
                    } else {
                        1e-3
                    };
                    g * v
                };
                max_current = max_current.max(i_mem.abs());
                add_current(&mut residuals, *pos, i_mem);
                add_current(&mut residuals, *neg, -i_mem);
            }
            ComponentRecord::SpikingNeuron {
                name,
                input_node,
                output_node,
                v_thresh,
                v_reset: _,
            } => {
                let v_in = get_v(*input_node);
                let (r_m, v_th, amp) = if let Some(ctx) = context {
                    if let Some(neuron) = ctx.get_neuron_model(name) {
                        (
                            neuron.r_mem_ohms,
                            neuron.v_thresh_volts,
                            neuron.spike_amplitude_volts,
                        )
                    } else {
                        (1e6, *v_thresh, 1.0)
                    }
                } else {
                    (1e6, *v_thresh, 1.0)
                };
                let i_in = v_in / r_m.max(1.0);
                max_current = max_current.max(i_in.abs());
                add_current(&mut residuals, *input_node, i_in);

                let v_out = get_v(*output_node);
                let target_v = if v_in >= v_th { amp } else { 0.0 };
                let i_out = (v_out - target_v) * 1.0;
                max_current = max_current.max(i_out.abs());
                add_current(&mut residuals, *output_node, i_out);
            }
            ComponentRecord::RadiationStrike {
                name,
                target_node,
                strike_time_s,
                let_mev,
            } => {
                let current_time_s = context.map(|c| c.current_time_s).unwrap_or(0.0);
                let i_strike = if let Some(ctx) = context {
                    if let Some(strike) = ctx.get_heavy_ion_strike(name) {
                        strike.current_at_time(current_time_s)
                    } else {
                        phonon_models::HeavyIonStrikeModel::typical_30nm_heavy_ion(
                            *strike_time_s,
                            *let_mev,
                        )
                        .current_at_time(current_time_s)
                    }
                } else {
                    phonon_models::HeavyIonStrikeModel::typical_30nm_heavy_ion(
                        *strike_time_s,
                        *let_mev,
                    )
                    .current_at_time(current_time_s)
                };

                max_current = max_current.max(i_strike.abs());
                // Current is injected into target_node from ground, so net leaving current is -i_strike
                add_current(&mut residuals, *target_node, -i_strike);
            }
        }
    }

    let mut max_res = 0.0;
    let mut sum_sq = 0.0;
    let mut worst_node = NodeId::GROUND;
    let mut node_residuals = Vec::new();

    let tol = reltol * max_current + abstol;
    let mut is_valid = true;

    for (n, &res_val) in residuals.iter().enumerate().take(active_nodes + 1).skip(1) {
        let res = res_val.abs();
        sum_sq += res * res;
        if res > max_res {
            max_res = res;
            worst_node = NodeId::new(n as u32);
        }
        if res > tol {
            is_valid = false;
        }
        node_residuals.push((NodeId::new(n as u32), res_val));
    }

    let rms_res = if active_nodes > 0 {
        (sum_sq / active_nodes as f64).sqrt()
    } else {
        0.0
    };

    KclReport {
        is_valid,
        max_residual: max_res,
        rms_residual: rms_res,
        worst_node,
        node_residuals,
    }
}

/// Evaluates algebraic Kirchhoff's Current Law residuals at all non-reference circuit nodes for static DC states:
/// $$\sum_{k \in \text{connected}(j)} I_{kj} - I_{\text{external}, j} = 0$$
pub fn verify_kcl(
    graph: &CircuitGraph,
    voltages: &[f64],
    branch_currents: &[f64],
    reltol: f64,
    abstol: f64,
) -> KclReport {
    let empty = HashMap::new();
    verify_kcl_dynamic(
        graph,
        voltages,
        branch_currents,
        &empty,
        None,
        reltol,
        abstol,
    )
}

/// Evaluates dynamic Kirchhoff's Current Law residuals for a saved transient step,
/// automatically accounting for reactive capacitor displacement currents:
pub fn verify_transient_kcl(
    graph: &CircuitGraph,
    step: &TransientStep,
    reltol: f64,
    abstol: f64,
) -> KclReport {
    verify_kcl_dynamic(
        graph,
        &step.voltages,
        &step.branch_currents,
        &step.capacitor_currents,
        None,
        reltol,
        abstol,
    )
}

/// Evaluates dynamic Kirchhoff's Current Law residuals for a saved transient step,
/// including non-linear semiconductor models from `ModelContext`:
pub fn verify_transient_kcl_with_context(
    graph: &CircuitGraph,
    context: &ModelContext,
    step: &TransientStep,
    reltol: f64,
    abstol: f64,
) -> KclReport {
    verify_kcl_dynamic(
        graph,
        &step.voltages,
        &step.branch_currents,
        &step.capacitor_currents,
        Some(context),
        reltol,
        abstol,
    )
}

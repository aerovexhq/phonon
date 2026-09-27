//! Integration tests for Single Flux Quantum (SFQ) pulse quantization,
//! JTL soliton propagation, DC-SQUID periodic flux modulation, and KCL conservation.

use phonon_core::{CircuitGraph, FLUX_QUANTUM};
use phonon_models::superconducting::{
    verify_flux_quantization, DcSquidModel, JosephsonRcsjModel, JtlStage, SfqPulse,
};
use phonon_solver::mna::non_linear_solver::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::verification::kcl_probe::verify_kcl_dynamic;
use std::collections::HashMap;

#[test]
fn test_sfq_soliton_pulse_area_quantization() {
    let jj = JosephsonRcsjModel::overdamped_rsfq(150e-6, 4.0); // Ic = 150 uA, Rn = 4 Ohm -> Vc = 0.6 mV
    let pulse = SfqPulse::from_junction(&jj, 10e-12); // Arrives at 10 ps

    let n_steps = 4000;
    let dt = 5e-15; // 5 fs across 20 ps window
    let mut times = Vec::with_capacity(n_steps);
    let mut voltages = Vec::with_capacity(n_steps);

    for i in 0..n_steps {
        let t = (i as f64) * dt;
        times.push(t);
        voltages.push(pulse.voltage(t));
    }

    // Verify pulse area quantization to Phi_0 with error < 0.05%
    let result = verify_flux_quantization(&times, &voltages, 5e-4);
    assert!(result.is_ok(), "Flux quantization failed: {:?}", result);
    let flux = result.unwrap();
    assert!((flux - FLUX_QUANTUM).abs() / FLUX_QUANTUM < 5e-4);
}

#[test]
fn test_dc_squid_periodic_flux_modulation() {
    let ic0 = 50e-6; // 50 uA per junction -> max Ic = 100 uA
    let rn = 8.0;
    let cap = 0.1e-12;
    let l_loop = 2e-12;
    let squid = DcSquidModel::symmetric(ic0, rn, cap, l_loop);

    let n_points = 100;
    for i in 0..=n_points {
        let frac = (i as f64) / (n_points as f64);
        let phi_ext = frac * 2.0 * FLUX_QUANTUM; // 0 to 2 Phi_0
        let ic_mod = squid.critical_current(phi_ext);

        // Theoretical expectation: 2 * Ic0 * |cos(pi * phi_ext / Phi_0)|
        let expected = 2.0 * ic0 * (std::f64::consts::PI * frac * 2.0).cos().abs();
        assert!(
            (ic_mod - expected).abs() < 1e-9,
            "SQUID modulation error at phi_ext = {} Phi_0: got {}, expected {}",
            frac * 2.0,
            ic_mod,
            expected
        );
    }
}

#[test]
fn test_jtl_stage_soliton_pulse_propagation() {
    let ic = 100e-6; // 100 uA
    let rn = 5.0; // 5 Ohms
    let l_coupling = 6e-12; // 6 pH
    let mut jtl = JtlStage::new(ic, rn, l_coupling, 0.85); // 85% bias

    let dt = 5e-15; // 5 fs
    let total_steps = 4000;
    let mut v1_max = 0.0;
    let mut v2_max = 0.0;
    let mut t_v1_peak = 0.0;
    let mut t_v2_peak = 0.0;

    for step in 0..total_steps {
        let t = (step as f64) * dt;
        // Inject current pulse at input between 1 ps and 4 ps
        let i_in = if (1.0e-12..=4.0e-12).contains(&t) {
            120e-6 // 120 uA pulse trigger
        } else {
            0.0
        };

        let (v1, v2) = jtl.step_rk4(dt, i_in);

        if v1 > v1_max {
            v1_max = v1;
            t_v1_peak = t;
        }
        if v2 > v2_max {
            v2_max = v2;
            t_v2_peak = t;
        }
    }

    // Both junctions must generate voltage pulses
    assert!(v1_max > 1e-4, "J1 failed to fire: peak = {} V", v1_max);
    assert!(v2_max > 1e-4, "J2 failed to fire: peak = {} V", v2_max);

    // Pulse must propagate sequentially from stage 1 to stage 2
    assert!(
        t_v2_peak >= t_v1_peak,
        "Pulse must propagate chronologically: J1 peak at {} s, J2 peak at {} s",
        t_v1_peak,
        t_v2_peak
    );

    // Phase difference after 2*pi leap
    assert!(
        jtl.jj1.phase > std::f64::consts::PI,
        "J1 phase must advance by >= pi after firing: {}",
        jtl.jj1.phase
    );
}

#[test]
fn test_josephson_circuit_kcl_conservation() {
    let mut graph = CircuitGraph::new();

    graph.add_voltage_source("V_BIAS", "1", "0", 0.1).unwrap();
    graph.add_resistor("R_LIMIT", "1", "2", 1000.0).unwrap(); // Limits current to ~100 uA
    graph
        .add_josephson_junction("JJ1", "2", "0", 200e-6, 5.0, 0.1e-12, Some(0.0))
        .unwrap();

    let mut context = ModelContext::new();
    context.set_josephson_junction("JJ1", JosephsonRcsjModel::new(200e-6, 5.0, 0.1e-12, 0.0));

    let newton_opts = NewtonOptions::default();

    let solution =
        solve_dc_non_linear(&graph, &context, &newton_opts).expect("DC solution must converge");

    // Verify Kirchhoff's Current Law
    let empty_cap = HashMap::new();
    let kcl_report = verify_kcl_dynamic(
        &graph,
        &solution.node_voltages,
        &solution.branch_currents,
        &empty_cap,
        Some(&context),
        1e-3,
        1e-7,
    );

    assert!(
        kcl_report.is_valid,
        "KCL violated in Josephson circuit: max residual = {} A at node {:?}",
        kcl_report.max_residual, kcl_report.worst_node
    );
}

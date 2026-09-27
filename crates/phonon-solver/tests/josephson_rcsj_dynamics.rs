//! Integration tests for Josephson junction RCSJ macro-models,
//! BCS superconducting gap dynamics, and companion matrix stamping.

use phonon_core::CircuitGraph;
use phonon_models::superconducting::{JosephsonRcsjModel, SuperconductorMaterial};
use phonon_solver::mna::non_linear_solver::{solve_dc_non_linear, ModelContext, NewtonOptions};

#[test]
fn test_bcs_gap_across_superconducting_presets() {
    let materials = [
        SuperconductorMaterial::niobium(),
        SuperconductorMaterial::niobium_nitride(),
        SuperconductorMaterial::aluminum(),
        SuperconductorMaterial::ybco(),
    ];

    for mat in &materials {
        let tc = mat.critical_temp_k;
        let delta_low = mat.bcs_gap_ev(tc * 0.1);
        let delta_mid = mat.bcs_gap_ev(tc * 0.5);
        let delta_above = mat.bcs_gap_ev(tc * 1.05);

        assert!(
            delta_low > delta_mid,
            "BCS gap for {} must decrease with temperature: {} vs {}",
            mat.name,
            delta_low,
            delta_mid
        );
        assert_eq!(
            delta_above, 0.0,
            "BCS gap for {} above Tc ({}) must be zero, got {}",
            mat.name, tc, delta_above
        );

        let ic_rn = mat.ambegaokar_baratoff_ic_rn(tc * 0.5);
        assert!(
            ic_rn > 0.0,
            "Ic*Rn for {} at 0.5 Tc must be positive: {}",
            mat.name,
            ic_rn
        );
    }
}

#[test]
fn test_rcsj_plasma_frequency_and_characteristic_metrics() {
    let ic = 200e-6; // 200 uA
    let rn = 8.0; // 8 Ohms
    let cap = 0.2e-12; // 200 fF
    let jj = JosephsonRcsjModel::new(ic, rn, cap, 0.0);

    let beta_c = jj.stewart_mccumber_beta_c();
    assert!(beta_c > 0.0);

    let fp = jj.plasma_frequency_hz();
    assert!(
        fp > 1e10 && fp < 1e12,
        "Plasma frequency {} Hz out of expected microwave range",
        fp
    );

    let lj0 = jj.zero_bias_inductance_h();
    assert!(
        lj0 > 1e-13 && lj0 < 1e-10,
        "Zero-bias inductance {} H out of physical range",
        lj0
    );

    let vc = jj.characteristic_voltage();
    assert!((vc - 1.6e-3).abs() < 1e-6);
}

#[test]
fn test_rcsj_companion_stamping_and_phase_advancement() {
    let mut jj = JosephsonRcsjModel::overdamped_rsfq(100e-6, 5.0);
    let dt = 1e-13; // 0.1 ps
    let n_steps = 100;

    for step in 0..n_steps {
        let v_applied = 0.5e-3 * ((step as f64) * 0.05).sin(); // 0.5 mV sinusoidal
        let stamp = jj.companion_stamp_trapezoidal(dt, v_applied);

        assert!(stamp.g_eq > 0.0);
        assert!(stamp.g_eq.is_finite());
        assert!(stamp.i_eq.is_finite());

        jj.advance_time_step(dt, v_applied);
    }

    assert!(jj.phase.is_finite());
}

#[test]
fn test_josephson_junction_in_dc_mna_circuit() {
    let mut graph = CircuitGraph::new();

    // Current source feeding a Josephson junction in parallel with a load resistor
    // I_src = 50 uA (below Ic = 100 uA) -> superconducting state, V(n1) near 0 V
    graph.add_current_source("I1", "0", "1", 50e-6).unwrap();
    graph
        .add_josephson_junction("JJ1", "1", "0", 100e-6, 10.0, 0.1e-12, Some(0.0))
        .unwrap();
    graph.add_resistor("R_LOAD", "1", "0", 1000.0).unwrap();

    let mut context = ModelContext::new();
    context.set_josephson_junction("JJ1", JosephsonRcsjModel::new(100e-6, 10.0, 0.1e-12, 0.0));

    let newton_opts = NewtonOptions::default();

    let solution = solve_dc_non_linear(&graph, &context, &newton_opts)
        .expect("DC non-linear solver with Josephson Junction must converge");

    let n1 = graph.get_node("1").unwrap();
    let v_n1 = solution.node_voltage(n1);
    // At DC below Ic, junction holds supercurrent with V ~ 0
    assert!(
        v_n1.abs() < 1e-3,
        "Superconducting junction at DC below Ic must have near-zero voltage: got {} V",
        v_n1
    );
}

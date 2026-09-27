//! Integration test: Total Ionizing Dose (TID) degradation in submicron CMOS.
//!
//! Verifies:
//! - Multi-krad dose sweeps (0 to 500 krad(SiO2)).
//! - Oxide trapped holes ($N_{ot}$) causing negative threshold voltage shifts.
//! - Interface trap generation ($N_{it}$) causing NMOS threshold rebound and PMOS monotonic decay.
//! - Subthreshold swing $S(D)$ stretch-out.
//! - Mobility degradation $\mu(D) / \mu_0$.
//! - Shallow Trench Isolation (STI) sidewall leakage current increase by orders of magnitude.
//! - MNA circuit simulation with degraded MOSFET characteristics and KCL validation.

use phonon_core::CircuitGraph;
use phonon_models::{MosfetModel, MosfetType, TotalIonizingDoseModel};
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::verification::verify_kcl_dynamic;
use std::collections::HashMap;

#[test]
fn test_tid_multi_krad_dose_sweep_and_rebound() {
    let tid_nmos = TotalIonizingDoseModel::nmos_28nm();
    let tid_pmos = TotalIonizingDoseModel::pmos_28nm();

    let doses = [0.0, 20.0, 50.0, 100.0, 200.0, 500.0];
    let mut nmos_shifts = Vec::new();
    let mut pmos_shifts = Vec::new();
    let mut swings = Vec::new();
    let mut mobilities = Vec::new();
    let mut sti_leakages = Vec::new();

    for &dose in &doses {
        nmos_shifts.push(tid_nmos.threshold_voltage_shift(dose));
        pmos_shifts.push(tid_pmos.threshold_voltage_shift(dose));
        swings.push(tid_nmos.subthreshold_swing(dose, 300.15));
        mobilities.push(tid_nmos.mobility_degradation_factor(dose));
        sti_leakages.push(tid_nmos.sti_sidewall_leakage(dose));
    }

    // 1. PMOS shift must be strictly monotonically negative (no turnaround)
    for i in 1..pmos_shifts.len() {
        assert!(
            pmos_shifts[i] < pmos_shifts[i - 1],
            "PMOS Vth shift must be monotonically decreasing with dose"
        );
    }

    // 2. NMOS shift: negative initially at low dose (20-50 krad), rebounds at high dose (200-500 krad)
    assert!(nmos_shifts[1] < 0.0); // 20 krad
    assert!(nmos_shifts[2] < 0.0); // 50 krad
    let min_nmos_shift = nmos_shifts.iter().cloned().fold(f64::INFINITY, f64::min);
    let final_nmos_shift = *nmos_shifts.last().unwrap();
    assert!(
        final_nmos_shift > min_nmos_shift,
        "NMOS must exhibit threshold voltage rebound at high dose: min = {} V, final = {} V",
        min_nmos_shift,
        final_nmos_shift
    );

    // 3. Subthreshold swing must monotonically increase
    for i in 1..swings.len() {
        assert!(
            swings[i] > swings[i - 1],
            "Subthreshold swing must stretch monotonically with dose: S[{}] = {} vs S[{}] = {}",
            i,
            swings[i],
            i - 1,
            swings[i - 1]
        );
    }
    assert!(swings.last().unwrap() > &0.080); // Stretched from 75 mV/dec

    // 4. Mobility factor must monotonically decrease
    for i in 1..mobilities.len() {
        assert!(
            mobilities[i] < mobilities[i - 1],
            "Mobility factor must decrease monotonically with dose"
        );
    }
    assert!(*mobilities.last().unwrap() < 0.95);

    // 5. STI leakage must increase by > 100x at 500 krad
    let initial_leak = sti_leakages[0];
    let final_leak = *sti_leakages.last().unwrap();
    assert!(
        final_leak > 100.0 * initial_leak,
        "STI sidewall leakage must surge under TID: initial = {} A, final = {} A",
        initial_leak,
        final_leak
    );
}

#[test]
fn test_tid_degraded_inverter_circuit_simulation() {
    let mut graph = CircuitGraph::new();

    // Inverter circuit: VDD = 1.2V, VIN = 0V
    graph
        .add_voltage_source("V_SUPPLY", "VDD", "0", 1.2)
        .unwrap();
    graph.add_voltage_source("V_IN", "VIN", "0", 0.0).unwrap(); // Low input -> High output

    graph
        .add_mosfet("M_P1", "VOUT", "VIN", "VDD", "VDD")
        .unwrap();
    graph.add_mosfet("M_N1", "VOUT", "VIN", "0", "0").unwrap();

    let out_node = graph.get_node("VOUT").unwrap();

    let tid = TotalIonizingDoseModel::nmos_28nm();
    let delta_vth_n = tid.threshold_voltage_shift(100.0);
    let mu_factor = tid.mobility_degradation_factor(100.0);

    let mut context = ModelContext::new();

    // Baseline PMOS
    let pmos = MosfetModel {
        mos_type: MosfetType::Pmos,
        vth0: -0.7,
        w: 2.0e-6,
        ..MosfetModel::default()
    };
    context.set_mosfet_model("M_P1", pmos);

    // Degraded NMOS with TID shift and mobility degradation
    let mut nmos = MosfetModel::default();
    nmos.vth0 += delta_vth_n;
    nmos.mu0 *= mu_factor;
    context.set_mosfet_model("M_N1", nmos);

    let sol = solve_dc_non_linear(&graph, &context, &NewtonOptions::default())
        .expect("Non-linear solve of TID-degraded inverter must converge");

    let v_out = sol.node_voltages[out_node.index()];
    assert!(
        v_out > 1.15,
        "Inverter output must be near VDD (1.2V) when Vin=0V: got {} V",
        v_out
    );

    // Verify KCL dynamic conservation
    let cap_map = HashMap::new();
    let kcl_report = verify_kcl_dynamic(
        &graph,
        &sol.node_voltages,
        &sol.branch_currents,
        &cap_map,
        Some(&context),
        1e-3,
        1e-6,
    );
    assert!(
        kcl_report.is_valid,
        "KCL must hold on TID-degraded inverter: max residual = {}",
        kcl_report.max_residual
    );
}

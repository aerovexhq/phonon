//! Tests for multi-fluid extended MHD, ICRF heating, thermonuclear D-T fusion reactivity,
//! and the comprehensive tokamak co-simulation benchmark engine.

use phonon_models::plasma::{
    IcrfHeatingSource, MhdFluidState, SafetyFactorProfile, ThermonuclearFusion, TokamakGeometry,
};
use phonon_solver::plasma::{
    AlfvenMhdConfig, AlfvenMhdStepper, TokamakBenchmarkRunner, TokamakScenario,
};

#[test]
fn test_mhd_fluid_spitzer_resistivity_and_lorentz_force() {
    let state = MhdFluidState {
        electron_density: 1.0e20,
        ion_density: 1.0e20,
        electron_temp_kev: 10.0,
        ion_temp_kev: 10.0,
        velocity: [0.0, 0.0, 0.0],
        magnetic_field: [0.0, 5.0, 0.0],
        current_density: [0.0, 0.0, 1.0e6], // 1 MA/m^2 in Z
    };

    // Pressure p = (ne * Te + ni * Ti) approx 2 * 1e20 * 10 * 1.602e-16 = 3.204e5 Pa (0.32 MPa)
    let p = state.total_pressure();
    assert!((p - 3.204353268e5).abs() < 1e2);

    // Spitzer resistivity at 10 keV: very low (superconductor-like)
    let eta = state.spitzer_resistivity(1.5);
    assert!(eta < 1e-7 && eta > 1e-10);

    // Lorentz force F = J x B = [0, 0, 1e6] x [0, 5, 0] = [-5e6, 0, 0] N/m^3
    let f_l = state.lorentz_force();
    assert!((f_l[0] - (-5.0e6)).abs() < 1e-6);
    assert!(f_l[1].abs() < 1e-6);
    assert!(f_l[2].abs() < 1e-6);

    // Hall electric field E_Hall = (J x B) / (n_e * e)
    let e_hall = state.hall_electric_field();
    assert!((e_hall[0] - (-5.0e6 / (1.0e20 * 1.602176634e-19))).abs() < 1e-3);
}

#[test]
fn test_thermonuclear_fusion_reactivity_and_lawson() {
    // At Ti = 15 keV, D-T fusion reactivity is near optimal
    let sigv_15 = ThermonuclearFusion::dt_reactivity(15.0);
    assert!(sigv_15 > 1.0e-22 && sigv_15 < 5.0e-22); // ~ 1.86e-22 m^3/s

    let n_d = 0.5e20;
    let n_t = 0.5e20;
    let p_alpha = ThermonuclearFusion::alpha_power_density(n_d, n_t, 15.0);
    assert!(p_alpha > 1.0e5); // > 100 kW/m^3

    let p_brem = ThermonuclearFusion::bremsstrahlung_power_loss(1.0e20, 15.0, 1.5);
    assert!(p_brem > 0.0);
    assert!(
        p_alpha > p_brem,
        "Alpha heating must exceed Bremsstrahlung at 15 keV"
    );

    // Lawson ignition criterion: n * T * tau_E >= 3e21
    let n = 1.0e20;
    let t = 20.0;
    let tau_ignited = 2.0;
    assert!(ThermonuclearFusion::is_ignited(n, t, tau_ignited)); // 1e20 * 20 * 2 = 4e21 >= 3e21

    let tau_sub_ignited = 0.5;
    assert!(!ThermonuclearFusion::is_ignited(n, t, tau_sub_ignited)); // 1e20 * 20 * 0.5 = 1e21 < 3e21

    // ICRF heating profile
    let icrf = IcrfHeatingSource::new(50.0e6, 20.0e6, 6.2, 0.2);
    let p_dens_res = icrf.power_density_at(6.2, 10.0);
    let p_dens_off = icrf.power_density_at(5.5, 10.0);
    assert!(p_dens_res > p_dens_off);
}

#[test]
fn test_alfven_mhd_stepper_wave_propagation() {
    let geom = TokamakGeometry::diiid_baseline();
    let q_prof = SafetyFactorProfile::new(1.1, 3.5);

    let config = AlfvenMhdConfig {
        num_surfaces: 15,
        dt: 1e-8,
        core_density: 1.0e-7,
        core_temp_kev: 3.0,
        m: 1,
        n: 1,
    };
    let mut stepper = AlfvenMhdStepper::new(geom, &q_prof, config);

    stepper.initialize_wavepacket(7, 2.0, 0.02);
    let energy_init = stepper.total_wave_energy();
    assert!(energy_init > 0.0);

    for _ in 0..100 {
        stepper.step();
    }

    let energy_final = stepper.total_wave_energy();
    assert!(energy_final > 0.0);
    assert!(
        energy_final <= energy_init * 1.01,
        "Wave energy must not unphysically explode"
    );
}

#[test]
fn test_tokamak_benchmark_iter_and_sparc() {
    let runner = TokamakBenchmarkRunner::new();

    // 1. Run SPARC compact high-field scenario
    let report_sparc = runner.run_benchmark(TokamakScenario::SparcCompact);
    assert_eq!(report_sparc.scenario, TokamakScenario::SparcCompact);
    assert!(report_sparc.grad_shafranov_residual < 1e-5);
    assert!(report_sparc.q_axis >= 0.8 && report_sparc.q_axis <= 1.5);
    assert!(report_sparc.edge_shear > 0.0);
    assert!(report_sparc.fast_ion_confinement_fraction >= 0.95);
    assert!(report_sparc.fast_ion_energy_conservation_error < 1e-7);
    assert!(report_sparc.magnetic_flux_conservation_error <= 1e-6);
    assert!(report_sparc.total_fusion_power_mw > 10.0); // SPARC designed for > 50-100 MW fusion
    assert!(report_sparc.steps_per_second > 1000.0);

    // 2. Run ITER ignition baseline
    let report_iter = runner.run_benchmark(TokamakScenario::IterIgnition);
    assert_eq!(report_iter.scenario, TokamakScenario::IterIgnition);
    assert!(
        report_iter.is_ignited,
        "ITER scenario must achieve ignition condition"
    );
    assert!(report_iter.fusion_gain_q >= 5.0); // ITER Q >= 10 target
    assert!(report_iter.alpha_heating_power_mw > 50.0);
}

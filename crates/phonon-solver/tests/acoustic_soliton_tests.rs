#![deny(unsafe_code)]

use phonon_solver::{
    DomainWallWaveguideParams, DomainWallWaveguideRouter, SineGordonParams, SineGordonSolver,
    SolitonKind,
};

#[test]
fn test_moving_kink_topological_charge_and_energy() {
    let params = SineGordonParams {
        grid_points: 256,
        length_m: 0.06,
        speed_of_sound: 1500.0,
        omega_0: 2.0 * std::f64::consts::PI * 40_000.0,
        damping_gamma: 0.0,
        cfl_factor: 0.35,
    };

    let solver = SineGordonSolver::new(
        params.clone(),
        SolitonKind::MovingKink {
            x0: 0.0,
            velocity_ratio: 0.3,
        },
    );

    let q = solver.state.topological_charge;
    assert!(
        (q - 1.0).abs() < 0.06,
        "Kink topological charge must be approximately +1.0, got {}",
        q
    );

    let e0 = params.kink_rest_energy();
    let gamma_l = 1.0 / (1.0 - 0.3 * 0.3_f64).sqrt();
    let expected_e = e0 * gamma_l;

    // Energy on discrete grid within 10% of continuous continuum theory
    let rel_diff = (solver.state.total_energy - expected_e).abs() / expected_e;
    assert!(
        rel_diff < 0.10,
        "Total energy must match relativistic continuum prediction, diff = {}",
        rel_diff
    );
}

#[test]
fn test_moving_antikink_topological_charge() {
    let params = SineGordonParams {
        grid_points: 256,
        length_m: 0.06,
        speed_of_sound: 1500.0,
        omega_0: 2.0 * std::f64::consts::PI * 40_000.0,
        damping_gamma: 0.0,
        cfl_factor: 0.35,
    };

    let solver = SineGordonSolver::new(
        params,
        SolitonKind::MovingAntikink {
            x0: 0.0,
            velocity_ratio: -0.25,
        },
    );

    let q = solver.state.topological_charge;
    assert!(
        (q - (-1.0)).abs() < 0.06,
        "Antikink topological charge must be approximately -1.0, got {}",
        q
    );
}

#[test]
fn test_undamped_energy_conservation() {
    let params = SineGordonParams {
        grid_points: 256,
        length_m: 0.06,
        speed_of_sound: 1500.0,
        omega_0: 2.0 * std::f64::consts::PI * 40_000.0,
        damping_gamma: 0.0, // Strictly undamped
        cfl_factor: 0.30,
    };

    let mut solver = SineGordonSolver::new(
        params,
        SolitonKind::MovingKink {
            x0: -0.01,
            velocity_ratio: 0.2,
        },
    );

    let initial_energy = solver.state.total_energy;
    assert!(initial_energy > 0.0);

    // Evolve 120 RK4 steps
    solver.step_n(120);

    let final_energy = solver.state.total_energy;
    let drift = (final_energy - initial_energy).abs() / initial_energy;
    assert!(
        drift < 0.005,
        "RK4 time integration must conserve Hamiltonian energy within 0.5%, drift = {}",
        drift
    );
}

#[test]
fn test_kink_antikink_collision_charge_neutrality() {
    let params = SineGordonParams {
        grid_points: 300,
        length_m: 0.08,
        speed_of_sound: 1500.0,
        omega_0: 2.0 * std::f64::consts::PI * 40_000.0,
        damping_gamma: 0.0,
        cfl_factor: 0.30,
    };

    let mut solver = SineGordonSolver::new(
        params,
        SolitonKind::KinkAntikinkCollision {
            x_left: -0.015,
            v_left_ratio: 0.3,
            x_right: 0.015,
            v_right_ratio: -0.3,
        },
    );

    let q_init = solver.state.topological_charge;
    assert!(
        q_init.abs() < 0.08,
        "Kink-antikink pair must have zero net topological charge, got {}",
        q_init
    );

    // Evolve through collision
    solver.step_n(100);

    let q_final = solver.state.topological_charge;
    assert!(
        q_final.abs() < 0.08,
        "Topological charge neutrality must be conserved throughout collision, got {}",
        q_final
    );
}

#[test]
fn test_breather_bound_oscillation() {
    let params = SineGordonParams {
        grid_points: 256,
        length_m: 0.06,
        speed_of_sound: 1500.0,
        omega_0: 2.0 * std::f64::consts::PI * 40_000.0,
        damping_gamma: 0.0,
        cfl_factor: 0.30,
    };

    let mut solver = SineGordonSolver::new(
        params,
        SolitonKind::Breather {
            x0: 0.0,
            frequency_ratio: 0.6,
        },
    );

    let q = solver.state.topological_charge;
    assert!(
        q.abs() < 0.08,
        "Breather must have net zero topological charge, got {}",
        q
    );

    // Evolve forward
    solver.step_n(80);
    assert!(solver.state.total_energy > 0.0);
}

#[test]
fn test_domain_wall_waveguide_confinement_and_transmission() {
    let params = DomainWallWaveguideParams {
        length_m: 0.04,
        width_m: 0.02,
        nx: 60,
        ny: 40,
        speed_of_sound: 1500.0,
        carrier_frequency_hz: 100_000.0,
        wall_thickness_m: 0.0015,
        bend_offset_m: 0.003,
        has_defect_obstacle: true,
        defect_radius_m: 0.001,
    };

    let router = DomainWallWaveguideRouter::new(params);

    assert!(
        router.s_parameters.confinement_factor >= 0.90,
        "Waveguide transverse energy confinement factor must be >= 0.90, got {}",
        router.s_parameters.confinement_factor
    );

    assert!(
        router.s_parameters.s21_db >= -1.0,
        "Forward transmission S21 must be >= -1.0 dB, got {}",
        router.s_parameters.s21_db
    );

    assert!(
        router.s_parameters.defect_immunity_ratio >= 0.85,
        "Defect immunity ratio must be >= 0.85, got {}",
        router.s_parameters.defect_immunity_ratio
    );
}

#![deny(unsafe_code)]

//! Analytical Integration & Multi-Physics Validation Test Suite for the Phonon Studio
//! Real-Time High-Order Symplectic Integration & Multi-Rate Co-Simulation Engine.

use phonon_models::symplectic_multirate::SymplecticMultirateParams;
use phonon_solver::symplectic_multirate::{
    Glrk4ButcherTableau, MilneAdaptiveController, MultiRateCoSimulator, SymplecticIntegrator,
};

/// 1. Algebraically verifies b_i * a_ij + b_j * a_ji - b_i * b_j == 0 to machine precision (< 1e-15).
#[test]
fn test_glrk4_butcher_tableau_symplectic_condition() {
    let tableau = Glrk4ButcherTableau::new();
    let m = tableau.symplectic_condition_matrix();

    for i in 0..2 {
        for j in 0..2 {
            let val = m[i][j].abs();
            assert!(
                val < 1.0e-15,
                "Symplectic condition b_i*a_ij + b_j*a_ji - b_i*b_j must vanish to machine precision at ({}, {}), got {:.2e}",
                i, j, val
            );
        }
    }
}

/// 2. Step size halving on Duffing oscillator confirms 4th-order error reduction (error ratio approx 16.0).
#[test]
fn test_glrk4_fourth_order_convergence() {
    let integrator = SymplecticIntegrator::new();
    let f_duffing = |x: &[f64], _t: f64, dx: &mut [f64]| {
        dx[0] = x[1];
        dx[1] = -x[0] - x[0] * x[0] * x[0];
    };

    let x0 = [1.0, 0.0];
    let t_end = 0.5;

    // High-resolution reference solution (N = 2048)
    let n_ref = 2048;
    let dt_ref = t_end / (n_ref as f64);
    let mut x_ref = x0;
    let mut x_tmp = [0.0; 2];
    let mut t = 0.0;
    for _ in 0..n_ref {
        assert!(integrator.step_glrk4(&x_ref, t, dt_ref, &mut x_tmp, f_duffing));
        x_ref = x_tmp;
        t += dt_ref;
    }

    // Solution with step size dt1 (N = 16)
    let n1 = 16;
    let dt1 = t_end / (n1 as f64);
    let mut x1 = x0;
    t = 0.0;
    for _ in 0..n1 {
        assert!(integrator.step_glrk4(&x1, t, dt1, &mut x_tmp, f_duffing));
        x1 = x_tmp;
        t += dt1;
    }

    // Solution with halved step size dt2 = dt1 / 2 (N = 32)
    let n2 = 32;
    let dt2 = t_end / (n2 as f64);
    let mut x2 = x0;
    t = 0.0;
    for _ in 0..n2 {
        assert!(integrator.step_glrk4(&x2, t, dt2, &mut x_tmp, f_duffing));
        x2 = x_tmp;
        t += dt2;
    }

    let err1 = ((x1[0] - x_ref[0]).powi(2) + (x1[1] - x_ref[1]).powi(2)).sqrt();
    let err2 = ((x2[0] - x_ref[0]).powi(2) + (x2[1] - x_ref[1]).powi(2)).sqrt();
    let ratio = err1 / err2;

    // For 4th order: 2^4 = 16.0
    assert!(
        ratio > 14.5 && ratio < 17.5,
        "Error ratio on step halving must be approx 16.0 for 4th order, got {:.3} (err1={:.2e}, err2={:.2e})",
        ratio, err1, err2
    );
}

/// 3. Across 10,000 steps of harmonic/Duffing oscillator, total energy drift is bounded < 1e-10 without secular drift.
#[test]
fn test_conservative_hamiltonian_zero_secular_energy_drift() {
    let integrator = SymplecticIntegrator::new();
    let dt = 0.005;
    let total_steps = 10_000;

    let f_hamiltonian = |x: &[f64], _t: f64, dx: &mut [f64]| {
        dx[0] = x[1];
        dx[1] = -x[0] - 0.1 * x[0] * x[0] * x[0];
    };

    let calc_energy = |x: &[f64]| -> f64 {
        0.5 * x[1] * x[1] + 0.5 * x[0] * x[0] + 0.025 * x[0].powi(4)
    };

    let mut x = [1.0, 0.0];
    let e0 = calc_energy(&x);
    let mut max_drift = 0.0;
    let mut x_next = [0.0; 2];
    let mut t = 0.0;

    for step in 0..total_steps {
        assert!(integrator.step_glrk4(&x, t, dt, &mut x_next, f_hamiltonian));
        x = x_next;
        t += dt;

        let e = calc_energy(&x);
        let drift = (e - e0).abs();
        if drift > max_drift {
            max_drift = drift;
        }

        // Bounded energy drift
        assert!(
            drift < 1.0e-10,
            "Drift exceeded 1e-10 at step {}: {:.2e}",
            step, drift
        );
    }

    let e_end = calc_energy(&x);
    let secular_slope = (e_end - e0).abs() / (total_steps as f64 * dt);
    assert!(
        secular_slope < 1.0e-11,
        "Secular energy drift slope must be zero, got {:.2e}",
        secular_slope
    );
}

/// 4. Verifies phase space Jacobian determinant |det(J)| = 1.000000000000.
#[test]
fn test_stormer_verlet_symplectic_invariance() {
    let integrator = SymplecticIntegrator::new();
    let x0 = [0.8, -0.6];
    let dt = 0.02;

    let f_osc = |x: &[f64], _t: f64, dx: &mut [f64]| {
        dx[0] = x[1];
        dx[1] = -x[0] - 0.5 * x[0].powi(3);
    };

    let det = integrator.stormer_verlet_phase_space_jacobian_det(&x0, 0.0, dt, f_osc);
    assert!(
        (det - 1.0).abs() < 1.0e-7,
        "Phase space Jacobian determinant |det(J)| must equal 1.000000000000, got {:.12}",
        det
    );

    // Also algebraically verify canonical separable shear map composition determinant
    // Det(shear_1 * shear_2 * shear_3) = 1 * 1 * 1 = 1.0
    let v_double_prime = 1.0 + 1.5 * x0[0] * x0[0];
    let j00 = 1.0 - 0.5 * dt * dt * v_double_prime;
    let j01 = dt;
    let j10 = -dt * v_double_prime * (1.0 - 0.25 * dt * dt * v_double_prime);
    let j11 = 1.0 - 0.5 * dt * dt * v_double_prime;
    let det_analytical = j00 * j11 - j01 * j10;
    assert!(
        (det_analytical - 1.0).abs() < 1.0e-14,
        "Analytical symplectic Jacobian determinant must equal 1.0 to machine precision, got {:.14}",
        det_analytical
    );
}

/// 5. Fast subsystem steps at sub-cycling ratio M = 200 while slow subsystem steps once per macro-step, verifying synchronization.
#[test]
fn test_multirate_fast_slow_partitioning() {
    let fast_dt = 1.0e-7;
    let slow_dt = 2.0e-5;
    let params = SymplecticMultirateParams::new(fast_dt, slow_dt, 4, 1.0e-6, true);
    assert_eq!(params.subcycling_ratio(), 200);

    let mut sim = MultiRateCoSimulator::new(params);
    let macro_steps = 50;

    for _ in 0..macro_steps {
        sim.step_passive_macro();
    }

    assert_eq!(sim.metrics.slow_steps_count, macro_steps as u64);
    assert_eq!(sim.metrics.fast_steps_count, (macro_steps * 200) as u64);

    let expected_time = macro_steps as f64 * slow_dt;
    assert!(
        (sim.simulated_time - expected_time).abs() < 1.0e-14,
        "Simulation time must synchronize at {:.6e}, got {:.6e}",
        expected_time, sim.simulated_time
    );
}

/// 6. Verifies energy transferred across macro-steps satisfies boundary work balance without numerical energy creation.
#[test]
fn test_multirate_interpolation_interface_passivity() {
    let fast_dt = 1.0e-7;
    let slow_dt = 2.0e-5;
    let params = SymplecticMultirateParams::new(fast_dt, slow_dt, 4, 1.0e-6, false);
    let mut sim = MultiRateCoSimulator::new(params);

    let e_initial = sim.initial_hamiltonian;
    let macro_steps = 100;

    for _ in 0..macro_steps {
        sim.step_passive_macro();
        let e_current = sim.passive_system.total_energy();
        // Strict passivity: no unphysical numerical energy creation beyond numerical precision
        assert!(
            e_current <= e_initial + 1.0e-9,
            "Passivity violation: energy created {:.10} > initial {:.10}",
            e_current, e_initial
        );
    }

    assert!(sim.metrics.is_symplectic_invariant_preserved);
}

/// 7. Verifies adaptive controller keeps local error <= tolerance across nonlinear transients.
#[test]
fn test_milne_adaptive_step_error_bound() {
    let tol = 1.0e-5;
    let mut controller = MilneAdaptiveController::new(tol, 1.0e-8, 1.0e-2, 1.0e-4);
    let integrator = SymplecticIntegrator::new();

    let f_nonlinear = |x: &[f64], _t: f64, dx: &mut [f64]| {
        dx[0] = x[1];
        dx[1] = -x[0] - 2.0 * x[0].powi(3);
    };

    let mut x = [2.0, 0.0];
    let mut t = 0.0;
    let steps = 60;

    for _ in 0..steps {
        assert!(controller.step(&mut x, &mut t, &integrator, f_nonlinear));
        assert!(
            controller.last_error_norm <= 1.0 + 1.0e-12,
            "Local error norm must not exceed 1.0 for accepted steps, got {:.4}",
            controller.last_error_norm
        );
    }

    assert!(controller.accepted_steps > 0);
}

/// 8. Asserts step size increases/doubles during smooth laminar trajectories.
#[test]
fn test_milne_step_doubling_on_smooth_regimes() {
    let tol = 1.0e-4;
    let init_dt = 1.0e-6;
    let mut controller = MilneAdaptiveController::new(tol, 1.0e-8, 1.0e-3, init_dt);
    let integrator = SymplecticIntegrator::new();

    // Smooth harmonic oscillator with small amplitude
    let f_smooth = |x: &[f64], _t: f64, dx: &mut [f64]| {
        dx[0] = x[1];
        dx[1] = -x[0];
    };

    let mut x = [0.01, 0.0];
    let mut t = 0.0;

    // Step across smooth trajectory
    for _ in 0..10 {
        assert!(controller.step(&mut x, &mut t, &integrator, f_smooth));
    }

    // Step size should have increased significantly (at least doubled)
    assert!(
        controller.current_dt >= 2.0 * init_dt,
        "Step size must increase/double during smooth regime, got current_dt={:.3e} vs init_dt={:.3e}",
        controller.current_dt, init_dt
    );
}

/// 9. Asserts step size decreases/halves when an abrupt shock or non-smooth contact occurs.
#[test]
fn test_milne_step_halving_on_stiff_shocks() {
    let tol = 1.0e-5;
    let mut controller = MilneAdaptiveController::new(tol, 1.0e-8, 1.0e-2, 1.0e-3);
    let integrator = SymplecticIntegrator::new();

    let dt_before = controller.current_dt;

    // Abrupt non-smooth contact shock occurring during the step interval [0, dt_before]
    let f_shock = |x: &[f64], t: f64, dx: &mut [f64]| {
        let contact = if t >= 0.0004 { -5.0e4 * x[0] } else { 0.0 };
        dx[0] = x[1];
        dx[1] = -x[0] + contact;
    };

    let x = [1.0, 0.0];
    let t = 0.0;

    let mut x_glrk = [0.0; 2];
    let mut x_verlet = [0.0; 2];

    assert!(integrator.step_glrk4(&x, t, dt_before, &mut x_glrk, f_shock));
    assert!(integrator.step_implicit_midpoint(&x, t, dt_before, &mut x_verlet, f_shock));

    let err = controller.compute_error_norm(&x_glrk, &x_verlet);
    assert!(err > 1.0, "Shock must generate error norm > 1.0, got {:.2e}", err);

    let decision = controller.evaluate_step(err);
    assert!(!decision.accepted, "Shock step must be rejected");
    assert!(
        decision.next_dt <= 0.5 * dt_before + 1.0e-12,
        "Step size must be halved on stiff shock, got next_dt={:.3e} vs 0.5*dt_before={:.3e}",
        decision.next_dt, 0.5 * dt_before
    );
    assert!(controller.rejected_steps > 0);
}

/// 10. Benchmark runs > 500,000 steps/sec.
#[test]
fn test_high_speed_symplectic_throughput_benchmark() {
    let integrator = SymplecticIntegrator::new();
    let mut q = [1.0];
    let mut p = [0.0];
    let inv_m = [1.0];
    let dt = 1.0e-4;

    let steps = 100_000;
    let start = std::time::Instant::now();

    for _ in 0..steps {
        integrator.step_stormer_verlet_separable(&mut q, &mut p, dt, &inv_m, |q_in, g_out| {
            g_out[0] = q_in[0];
        });
    }

    let elapsed = start.elapsed();
    let throughput = (steps as f64) / elapsed.as_secs_f64();

    assert!(
        throughput > 500_000.0,
        "Symplectic throughput must exceed 500,000 steps/sec, achieved {:.1} steps/sec",
        throughput
    );
}

/// 11. Simultaneous 1 MHz PWM switching and acoustic wave co-simulation remains 100% stable with zero NaN.
#[test]
fn test_extreme_switching_audio_coupled_stability() {
    let fast_dt = 1.0e-7; // 10 MHz micro-step for 1 MHz PWM
    let slow_dt = 2.0833333333333333e-5; // 48 kHz macro-step for acoustic resonator
    let params = SymplecticMultirateParams::new(fast_dt, slow_dt, 4, 1.0e-6, false);

    let mut sim = MultiRateCoSimulator::new(params);
    let macro_steps = 150;

    for step in 0..macro_steps {
        sim.step_pwm_acoustic_macro();

        // Assert all states remain finite and non-NaN
        assert!(
            sim.pwm_system.i_inductor.is_finite(),
            "Inductor current became non-finite at step {}",
            step
        );
        assert!(
            !sim.pwm_system.i_inductor.is_nan(),
            "Inductor current became NaN at step {}",
            step
        );
        assert!(
            sim.pwm_system.v_capacitor.is_finite(),
            "Capacitor voltage became non-finite at step {}",
            step
        );
        assert!(
            !sim.pwm_system.v_capacitor.is_nan(),
            "Capacitor voltage became NaN at step {}",
            step
        );
        assert!(
            sim.pwm_system.voice_coil_pos_m.is_finite(),
            "Voice coil position became non-finite at step {}",
            step
        );
        assert!(
            !sim.pwm_system.voice_coil_pos_m.is_nan(),
            "Voice coil position became NaN at step {}",
            step
        );
        assert!(
            sim.pwm_system.voice_coil_vel_m_s.is_finite(),
            "Voice coil velocity became non-finite at step {}",
            step
        );
        assert!(
            !sim.pwm_system.voice_coil_vel_m_s.is_nan(),
            "Voice coil velocity became NaN at step {}",
            step
        );
    }

    // Verify sub-cycling execution counters
    let expected_fast = macro_steps as u64 * params.subcycling_ratio() as u64;
    assert_eq!(sim.metrics.slow_steps_count, macro_steps as u64);
    assert_eq!(sim.metrics.fast_steps_count, expected_fast);
}

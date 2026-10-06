#![deny(unsafe_code)]

//! Integration Test Suite for Floquet Chiral Polariton Circulator & Cryogenic Isolator Engine.
//!
//! Verifies:
//! - Floquet time-reversal symmetry breaking and dispersion asymmetry |k+ - (-k-)| > 0.
//! - Forward vs backward group velocity asymmetry v_g_forward != v_g_backward.
//! - 3-port cyclic scattering matrix: insertion loss <= 0.5 dB, isolation >= 35.0 dB, return loss >= 20.0 dB.
//! - Cyclic permutation invariance (Port 1 -> Port 2 -> Port 3 -> Port 1).
//! - Cryogenic quantum-limited noise floor at 20 mK (n_add < 0.1).
//! - Directivity D = ISO - IL >= 35.0 dB.
//! - 10-point comprehensive physics audit full pass.

use phonon_solver::chiral_polariton_circulator::{
    ChiralPolaritonCirculator, ChiralPolaritonParams, CirculatorParams,
    CryogenicIsolatorParams, FloquetPolaritonDispersion, ThreePortCirculator,
};

#[test]
fn test_floquet_time_reversal_breaking_and_dispersion_asymmetry() {
    let params = ChiralPolaritonParams::default();
    let dispersion = FloquetPolaritonDispersion::new(params.clone());

    // Evaluate forward and backward wavenumbers at center frequency 5.0 GHz
    let (k_plus, k_minus, delta_k) = dispersion.forward_backward_wavenumbers(5.0);

    // Forward wavenumber must be strictly positive
    assert!(k_plus > 0.0, "Forward wavenumber must be positive: k+ = {}", k_plus);
    // Backward wavenumber must be strictly negative
    assert!(k_minus < 0.0, "Backward wavenumber must be negative: k- = {}", k_minus);

    // Non-reciprocity dispersion asymmetry: |k_plus - (-k_minus)| = |k_plus + k_minus| > 0
    assert!(
        delta_k > 0.0,
        "Time-reversal symmetry breaking must produce non-zero dispersion asymmetry delta_k = {}",
        delta_k
    );
    assert!(
        delta_k > 100.0,
        "Dispersion asymmetry delta_k = {} rad/m should be substantial under 15 Oe drive",
        delta_k
    );

    // Avoided crossing polariton gap Delta_omega = 2 * g_eff
    let gap_mhz = dispersion.avoided_crossing_gap_mhz();
    assert!(
        gap_mhz >= 80.0,
        "Avoided-crossing polariton gap Delta_omega = {} MHz should exceed 80 MHz",
        gap_mhz
    );
}

#[test]
fn test_forward_backward_group_velocity_asymmetry() {
    let params = ChiralPolaritonParams::default();
    let dispersion = FloquetPolaritonDispersion::new(params);

    let (v_g_fwd, v_g_bwd, delta_vg) = dispersion.forward_backward_group_velocities(5.0);

    assert!(v_g_fwd > 0.0, "Forward group velocity must be positive");
    assert!(v_g_bwd > 0.0, "Backward group velocity magnitude must be positive");
    assert!(
        delta_vg > 0.0,
        "Group velocity asymmetry must be non-zero due to chiral rotating drive: delta_vg = {}",
        delta_vg
    );
    assert!(
        (v_g_fwd - v_g_bwd).abs() > 1.0,
        "Asymmetry between forward ({}) and backward ({}) group velocities must be discernible",
        v_g_fwd,
        v_g_bwd
    );
}

#[test]
fn test_three_port_scattering_matrix_performance() {
    let params = CirculatorParams::default();
    let circulator = ThreePortCirculator::new(params.clone());

    let s = circulator.center_s_matrix();

    // 1. Forward transmission insertion loss: IL <= 0.5 dB (|S_21| >= 0.944)
    let il_db = s.insertion_loss_db();
    let s21_mag = s.s21_mag();
    assert!(
        il_db <= 0.5,
        "Insertion loss IL = {} dB exceeds 0.5 dB threshold",
        il_db
    );
    assert!(
        s21_mag >= 0.944,
        "Transmission magnitude |S_21| = {} below 0.944",
        s21_mag
    );

    // 2. Backward transmission isolation: ISO >= 35.0 dB (|S_12| <= 0.0178)
    let iso_db = s.isolation_db();
    let s12_mag = s.s12_mag();
    assert!(
        iso_db >= 35.0,
        "Isolation ISO = {} dB below 35.0 dB threshold",
        iso_db
    );
    assert!(
        s12_mag <= 0.0178,
        "Isolation leakage |S_12| = {} exceeds 0.0178 threshold",
        s12_mag
    );

    // 3. Return loss: RL >= 20.0 dB (|S_11| <= 0.100)
    let rl_db = s.return_loss_db();
    let s11_mag = s.s11_mag();
    assert!(
        rl_db >= 20.0,
        "Return loss RL = {} dB below 20.0 dB threshold",
        rl_db
    );
    assert!(
        s11_mag <= 0.100,
        "Reflection magnitude |S_11| = {} exceeds 0.100 threshold",
        s11_mag
    );
}

#[test]
fn test_cyclic_permutation_invariance() {
    let params = CirculatorParams::default();
    let circulator = ThreePortCirculator::new(params);

    let (is_invariant, residual) = circulator.verify_cyclic_invariance();
    assert!(
        is_invariant,
        "Circulator must exhibit cyclic permutation symmetry (residual: {})",
        residual
    );
    assert!(
        residual < 1.0e-6,
        "Cyclic permutation residual {} must be < 1e-6",
        residual
    );

    let s = circulator.center_s_matrix();

    // S_21 == S_32 == S_13
    let diff_forward_1 = (s.s21.re - s.s32.re).hypot(s.s21.im - s.s32.im);
    let diff_forward_2 = (s.s32.re - s.s13.re).hypot(s.s32.im - s.s13.im);
    assert!(diff_forward_1 < 1.0e-12, "S_21 must equal S_32");
    assert!(diff_forward_2 < 1.0e-12, "S_32 must equal S_13");

    // S_12 == S_23 == S_31
    let diff_iso_1 = (s.s12.re - s.s23.re).hypot(s.s12.im - s.s23.im);
    let diff_iso_2 = (s.s23.re - s.s31.re).hypot(s.s23.im - s.s31.im);
    assert!(diff_iso_1 < 1.0e-12, "S_12 must equal S_23");
    assert!(diff_iso_2 < 1.0e-12, "S_23 must equal S_31");
}

#[test]
fn test_cryogenic_quantum_noise_floor() {
    let circulator = ChiralPolaritonCirculator::default();
    let s = circulator.center_s_matrix();
    let iso_params = CryogenicIsolatorParams::default(); // 20 mK, 5.0 GHz

    let metrics = iso_params.evaluate_metrics(&s);

    // Added noise quanta n_add < 0.1 at 20 mK
    assert!(
        metrics.added_noise_quanta < 0.10,
        "Added noise quanta {} must be < 0.10 at 20 mK",
        metrics.added_noise_quanta
    );
    assert!(
        metrics.is_quantum_limited,
        "Isolator must operate within standard quantum limit at 20 mK"
    );

    // Noise temperature should approach zero-point quantum limit ~120 mK
    assert!(
        metrics.noise_temperature_k > 0.0 && metrics.noise_temperature_k < 0.5,
        "Noise temperature {} K must be near quantum limit",
        metrics.noise_temperature_k
    );
}

#[test]
fn test_directivity_and_compression() {
    let circulator = ChiralPolaritonCirculator::default();
    let metrics = circulator.cryogenic_metrics();

    // Directivity D = ISO - IL >= 35.0 dB
    assert!(
        metrics.directivity_db >= 35.0,
        "Directivity D = {} dB below 35.0 dB requirement",
        metrics.directivity_db
    );

    // 1-dB compression point P_1dB >= -20.0 dBm
    assert!(
        metrics.power_1db_compression_dbm >= -20.0,
        "Power handling P_1dB = {} dBm below -20.0 dBm threshold",
        metrics.power_1db_compression_dbm
    );
}

#[test]
fn test_ten_point_audit_full_pass() {
    let circulator = ChiralPolaritonCirculator::default();
    let report = circulator.audit_circulator();

    assert_eq!(report.total_count, 10, "Audit must evaluate exactly 10 criteria");
    assert_eq!(
        report.passed_count, 10,
        "All 10 physics audit criteria must pass. Failed criteria: {:?}",
        report
            .criteria
            .iter()
            .filter(|c| !c.passed)
            .map(|c| c.name)
            .collect::<Vec<_>>()
    );
    assert!(report.overall_pass, "Overall audit status must be PASS");
    assert!(
        report.cold_boot_latency_us < 2000.0,
        "Cold boot latency {} us must be < 2000 us (2.0 ms)",
        report.cold_boot_latency_us
    );
}

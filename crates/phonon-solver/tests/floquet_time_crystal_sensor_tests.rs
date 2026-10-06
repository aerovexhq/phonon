#![deny(unsafe_code)]

//! Test suite for Floquet Discrete Time Crystal Magnetometer & Sensor Network.

use phonon_solver::floquet_time_crystal_sensor::{
    DistributedSensorNetwork, FloquetTimeCrystalSensorProcessor, MagneticDipoleSource,
    MagnetometerParams, SensorNetworkParams, SubharmonicMagnetometer, TimeCrystalDynamicsSolver,
    TimeCrystalParams,
};

#[test]
fn test_discrete_time_translation_symmetry_breaking_and_period_doubling() {
    let params = TimeCrystalParams {
        chain_length: 8,
        drive_period_us: 1.0,
        pulse_error_epsilon: 0.05,
        ising_coupling_j_khz: 250.0,
        disorder_w_khz: 500.0,
        cycles: 60,
    };
    let solver = TimeCrystalDynamicsSolver::new(params);
    let result = solver.evolve_stroboscopic();

    assert_eq!(result.cycles, 60);
    assert_eq!(result.magnetization.len(), 60);

    // Initial state is all spins up, M_z(0) should be near +1.0
    assert!((result.magnetization[0] - 1.0).abs() < 1.0e-6);

    // Check period-2T oscillation: sign of M_z(nT) should alternate
    let mut sign_alternations = 0;
    for n in 1..result.cycles {
        if result.magnetization[n] * result.magnetization[n - 1] < 0.0 {
            sign_alternations += 1;
        }
    }
    let alternation_fraction = (sign_alternations as f64) / ((result.cycles - 1) as f64);
    assert!(
        alternation_fraction >= 0.90,
        "Period-doubling requires regular sign alternations, found fraction: {}",
        alternation_fraction
    );

    assert!(
        (result.period_doubling_ratio - 2.0).abs() < 0.1,
        "Period doubling ratio must be ~2.0, found: {}",
        result.period_doubling_ratio
    );

    // Edwards-Anderson temporal order parameter q_EA must be >= 0.70
    assert!(
        result.edwards_anderson_q_ea >= 0.70,
        "Edwards-Anderson q_EA must be >= 0.70, found: {}",
        result.edwards_anderson_q_ea
    );
    assert!(result.is_time_crystal);
}

#[test]
fn test_mbl_stabilization_prevents_thermalization() {
    let params = TimeCrystalParams::default();
    assert!(params.disorder_w_khz >= 2.0 * params.ising_coupling_j_khz);
    assert!(params.mbl_ratio() >= 2.0);

    let solver = TimeCrystalDynamicsSolver::new(params);
    let res = solver.evolve_stroboscopic();

    // In an MBL time crystal, the magnetization envelope does not decay to zero
    let late_cycles_mag_abs: Vec<f64> = res.magnetization[40..60]
        .iter()
        .map(|m| m.abs())
        .collect();
    let mean_late_mag: f64 = late_cycles_mag_abs.iter().sum::<f64>() / (late_cycles_mag_abs.len() as f64);

    assert!(
        mean_late_mag >= 0.70,
        "MBL stabilization must prevent decay to zero, late cycle average: {}",
        mean_late_mag
    );
}

#[test]
fn test_subharmonic_fourier_peak_at_half_omega() {
    let params = TimeCrystalParams::default();
    let solver = TimeCrystalDynamicsSolver::new(params);
    let res = solver.evolve_stroboscopic();
    let spec = solver.compute_fourier_spectrum(&res.magnetization);

    // Dominant peak must be at omega / Omega = 0.5
    assert!(
        (spec.peak_frequency_norm - 0.5).abs() <= (1.0 / 60.0 + 1.0e-5),
        "Peak must be at 0.5 * Omega, found: {}",
        spec.peak_frequency_norm
    );

    // Peak power ratio must be >= 70%
    assert!(
        spec.peak_power_ratio >= 0.70,
        "Subharmonic peak power ratio must be >= 0.70, found: {}",
        spec.peak_power_ratio
    );
    assert!(spec.is_subharmonic_locked);
}

#[test]
fn test_perturbation_rigidity_plateau() {
    let solver = TimeCrystalDynamicsSolver::new(TimeCrystalParams::default());
    let plateau = solver.sweep_rigidity_plateau(-0.15, 0.15, 7);

    assert!(
        plateau.is_rigid,
        "DTC must be rigid against non-zero pulse errors in [-0.15, 0.15]"
    );
    assert!(plateau.plateau_width >= 0.30);

    // Verify subharmonic ratio remains >= 70% across the sweep
    for (i, &ratio) in plateau.subharmonic_power_ratios.iter().enumerate() {
        assert!(
            ratio >= 0.70,
            "Ratio at epsilon={} must be >= 0.70, found: {}",
            plateau.epsilons[i],
            ratio
        );
    }
}

#[test]
fn test_sub_femtotesla_magnetic_sensitivity() {
    let mag_params = MagnetometerParams::default();
    let mag = SubharmonicMagnetometer::new(mag_params);

    let b_min = mag.minimum_detectable_field();
    assert!(
        b_min <= 1.0,
        "Sensitivity B_min must be <= 1.0 fT / sqrt(Hz), found: {}",
        b_min
    );
    // Typical design target is ~0.35 fT / sqrt(Hz)
    assert!(
        b_min < 0.50,
        "B_min should comfortably reach sub-femtotesla regime: {}",
        b_min
    );

    // Check dynamic range DR >= 70.0 dB (measured >= 75 dB)
    let dr = mag.dynamic_range_db();
    assert!(
        dr >= 70.0,
        "Dynamic range must be >= 70.0 dB, found: {}",
        dr
    );
    assert!(
        dr >= 75.0,
        "Measured dynamic range target >= 75.0 dB, found: {}",
        dr
    );

    // Check phase shift calculation and inversion consistency
    let test_b = 150.0; // 150 fT
    let phase = mag.compute_subharmonic_phase_shift(test_b);
    let reconstructed_b = mag.compute_field_from_phase(phase);
    assert!(
        (reconstructed_b - test_b).abs() < 1.0e-9,
        "Phase inversion must match input field"
    );
}

#[test]
fn test_differential_gradiometer_common_mode_noise_rejection() {
    let net_params = SensorNetworkParams::default();
    let mag = SubharmonicMagnetometer::new(MagnetometerParams::default());
    let mut network = DistributedSensorNetwork::new(net_params, mag);

    let cmrr = network.common_mode_rejection_ratio_db();
    assert!(
        cmrr >= 40.0,
        "CMRR must be >= 40.0 dB, found: {}",
        cmrr
    );
    assert!(
        cmrr >= 45.0,
        "Measured CMRR target >= 45.0 dB, found: {}",
        cmrr
    );

    // Sample dipole field and test dipole reconstruction fidelity
    let dipole = MagneticDipoleSource::default();
    network.sample_dipole_field(&dipole, true);
    let recon = network.reconstruct_dipole(&dipole);

    assert!(
        recon.fidelity >= 0.95,
        "Spatial gradient reconstruction fidelity must be >= 95%, found: {}",
        recon.fidelity * 100.0
    );
    assert!(
        recon.localization_error_mm < 0.5,
        "Localization error should be sub-millimeter: {}",
        recon.localization_error_mm
    );
}

#[test]
fn test_ten_point_physics_audit_full_pass() {
    let processor = FloquetTimeCrystalSensorProcessor::default();
    let report = processor.audit_sensor();

    assert_eq!(report.total_count, 10);
    assert_eq!(
        report.passed_count, 10,
        "All 10 physics audit criteria must pass! Passed: {} / {}",
        report.passed_count, report.total_count
    );
    assert!(report.overall_pass);

    for (idx, crit) in report.criteria.iter().enumerate() {
        assert!(
            crit.passed,
            "Criterion {} ('{}') failed: measured {} {} (target: {})",
            idx + 1,
            crit.name,
            crit.measured_value,
            crit.units,
            crit.target_threshold
        );
    }
}

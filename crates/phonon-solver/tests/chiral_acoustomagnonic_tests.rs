#![deny(unsafe_code)]

use phonon_solver::chiral_acoustomagnonic_isolator::{
    AcoustomagnonicDispersionSolver, AcoustomagnonicParams, ChiralAcoustomagnonicProcessor,
    CryogenicCirculatorParams, CryogenicQubitCirculator, NonReciprocalSawIsolator,
};

#[test]
fn test_bare_saw_and_magnon_dispersion() {
    let params = AcoustomagnonicParams::default();
    let solver = AcoustomagnonicDispersionSolver::new(params);

    // Test acoustic SAW frequency scales linearly with wavenumber
    let k1 = 2.0; // rad/um
    let k2 = 4.0;
    let f1 = solver.bare_saw_freq(k1);
    let f2 = solver.bare_saw_freq(k2);

    assert!(f1 > 0.5 && f1 < 3.0, "f1 was {}", f1);
    assert!(f2 > f1, "f2 should exceed f1: {} vs {}", f2, f1);
    assert!((f2 / f1 - 2.0).abs() < 1e-6, "Linear SAW scaling expected");

    // Test magnon frequency is positive and Damon-Eshbach surface mode exists
    let f_mag = solver.bare_magnon_freq(k1);
    assert!(f_mag > 2.0 && f_mag < 8.0, "f_mag was {}", f_mag);
}

#[test]
fn test_magnetoelastic_avoided_crossing_and_polariton_gap() {
    let params = AcoustomagnonicParams::default();
    let solver = AcoustomagnonicDispersionSolver::new(params);

    let gap_mhz = solver.polariton_gap_mhz();
    assert!(
        gap_mhz >= 40.0,
        "Polariton avoided crossing gap must be >= 40 MHz, got {}",
        gap_mhz
    );

    // Test forward and backward branches at resonance
    let (fwd_l, fwd_u, bwd_l, bwd_u) = solver.solve_polariton_branches(5.0);
    assert!(fwd_u > fwd_l, "Upper branch must exceed lower branch");
    assert!(bwd_u > bwd_l, "Backward upper must exceed lower");
    assert!((fwd_u - fwd_l) * 1.0e3 >= gap_mhz * 0.5);
}

#[test]
fn test_nonreciprocal_wavevector_splitting() {
    let params = AcoustomagnonicParams::default();
    let solver = AcoustomagnonicDispersionSolver::new(params);

    let delta_k = solver.nonreciprocal_wavevector_splitting();
    assert!(
        delta_k >= 0.035,
        "Non-reciprocal wavevector splitting Delta_k must be >= 0.035 rad/um, got {}",
        delta_k
    );
}

#[test]
fn test_nonreciprocal_saw_s_parameters() {
    let isolator = NonReciprocalSawIsolator::default_config();
    let metrics = isolator.compute_metrics();

    assert!(
        metrics.insertion_loss_db <= 0.60,
        "Insertion loss must be <= 0.60 dB, got {}",
        metrics.insertion_loss_db
    );
    assert!(
        metrics.isolation_depth_db >= 35.0,
        "Isolation depth must be >= 35.0 dB, got {}",
        metrics.isolation_depth_db
    );
    assert!(
        metrics.isolation_contrast_db >= 34.0,
        "Isolation contrast must be >= 34.0 dB, got {}",
        metrics.isolation_contrast_db
    );
    assert!(
        metrics.bandwidth_3db_mhz >= 30.0,
        "3-dB bandwidth must be >= 30.0 MHz, got {}",
        metrics.bandwidth_3db_mhz
    );
    assert!(
        metrics.return_loss_s11_db <= -22.0,
        "Return loss must be <= -22.0 dB, got {}",
        metrics.return_loss_s11_db
    );
    assert!(
        metrics.forward_transmission >= 0.85,
        "Forward transmission must be >= 0.85, got {}",
        metrics.forward_transmission
    );
    assert!(
        metrics.backward_transmission <= 0.001,
        "Backward transmission must be <= 0.001, got {}",
        metrics.backward_transmission
    );
}

#[test]
fn test_cryogenic_circulator_s_matrix_and_unitarity() {
    let params = CryogenicCirculatorParams::default();
    let circulator = CryogenicQubitCirculator::new(params);

    let s_mat = circulator.evaluate_s_matrix();
    assert!(
        s_mat.s_forward_mag > 0.90,
        "Forward transmission mag must be > 0.90, got {}",
        s_mat.s_forward_mag
    );
    assert!(
        s_mat.s_reverse_mag < 0.02,
        "Reverse isolation mag must be < 0.02, got {}",
        s_mat.s_reverse_mag
    );
    assert!(
        s_mat.unitarity_deficit < 0.05,
        "Unitarity deficit must be < 0.05, got {}",
        s_mat.unitarity_deficit
    );
    assert!(
        -s_mat.s_reverse_db >= 35.0,
        "Isolation must be >= 35 dB, got {}",
        -s_mat.s_reverse_db
    );
}

#[test]
fn test_cryogenic_quantum_added_noise_and_thermal_leakage() {
    let params = CryogenicCirculatorParams::default();
    let circulator = CryogenicQubitCirculator::new(params);

    let n_add = circulator.added_noise_quanta();
    assert!(
        n_add <= 0.55,
        "Added noise quanta at 20 mK must be <= 0.55, got {}",
        n_add
    );

    let (n_leak, iso_db) = circulator.thermal_leakage_photons();
    assert!(
        n_leak < 1.0e-3,
        "Thermal leakage photons from 4K must be < 1e-3, got {}",
        n_leak
    );
    assert!(
        iso_db >= 35.0,
        "Thermal isolation must be >= 35 dB, got {}",
        iso_db
    );
}

#[test]
fn test_qubit_readout_snr_and_fidelity() {
    let params = CryogenicCirculatorParams::default();
    let circulator = CryogenicQubitCirculator::new(params);

    let (snr_db, fidelity, dephasing_mhz) = circulator.evaluate_readout_performance();
    assert!(
        snr_db >= 18.5,
        "Readout SNR must be >= 18.5 dB, got {}",
        snr_db
    );
    assert!(
        fidelity >= 0.998,
        "QND Readout fidelity must be >= 0.998, got {}",
        fidelity
    );
    assert!(
        dephasing_mhz > 0.0,
        "Measurement dephasing must be positive, got {}",
        dephasing_mhz
    );
}

#[test]
fn test_master_processor_10_point_physics_audit() {
    let processor = ChiralAcoustomagnonicProcessor::default();
    let audit = processor.audit_acoustomagnonic_processor();

    assert!(
        audit.all_passed,
        "All 10 physics audit criteria must pass, pass_count was {}",
        audit.pass_count
    );
    assert_eq!(audit.pass_count, 10);
    assert!(audit.pass_polariton_gap);
    assert!(audit.pass_wavevector_splitting);
    assert!(audit.pass_insertion_loss);
    assert!(audit.pass_isolation_depth);
    assert!(audit.pass_isolation_contrast);
    assert!(audit.pass_bandwidth);
    assert!(audit.pass_circulator_unitarity);
    assert!(audit.pass_quantum_added_noise);
    assert!(audit.pass_thermal_leakage_suppression);
    assert!(audit.pass_readout_performance);
}

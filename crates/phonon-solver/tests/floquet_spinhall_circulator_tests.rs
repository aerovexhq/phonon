#![deny(unsafe_code)]

//! Automated Physical and Numerical Verification Test Suite for Phase 418:
//! Quantum Metamaterial Topological Acoustic Floquet Spin-Hall Insulator
//! & Non-Reciprocal Cryogenic Circulator.

use phonon_solver::floquet_spinhall_circulator::{
    CryogenicReadoutEngine, CryogenicReadoutParams, FloquetCirculator, FloquetCirculatorParams,
    FloquetSpinHallCirculator, FloquetSpinHallParams, SpinHallLattice, SpinHallParams,
    SpinHallPseudoSpin,
};

#[test]
fn test_spinhall_pseudo_spin_and_bulk_gap() {
    let top_params = SpinHallParams {
        bare_frequency_ghz: 1.0,
        lattice_constant_a_mm: 5.0,
        inter_intra_ratio: 1.15,
        acoustic_velocity_m_s: 3400.0,
        corner_angle_deg: 60.0,
        boundary_cells: 32,
    };
    let top_lattice = SpinHallLattice::new(top_params);

    assert!(top_lattice.is_topological());
    assert_eq!(top_lattice.spin_chern_number(), 1.0);
    let gap = top_lattice.bulk_bandgap_ghz();
    assert!(gap > 0.030, "Bulk bandgap should be > 30 MHz, got {} GHz", gap);

    // Trivial lattice
    let triv_params = SpinHallParams {
        inter_intra_ratio: 0.85,
        ..Default::default()
    };
    let triv_lattice = SpinHallLattice::new(triv_params);
    assert!(!triv_lattice.is_topological());
    assert_eq!(triv_lattice.spin_chern_number(), 0.0);
}

#[test]
fn test_helical_edge_dispersion_and_spin_momentum_locking() {
    let lattice = SpinHallLattice::new(SpinHallParams::default());
    let modes = lattice.compute_helical_edge_dispersion(31);

    assert_eq!(modes.len(), 62); // 31 spin-up + 31 spin-down

    let spin_up_modes: Vec<_> = modes
        .iter()
        .filter(|m| m.pseudo_spin == SpinHallPseudoSpin::SpinUp)
        .collect();
    let spin_down_modes: Vec<_> = modes
        .iter()
        .filter(|m| m.pseudo_spin == SpinHallPseudoSpin::SpinDown)
        .collect();

    assert_eq!(spin_up_modes.len(), 31);
    assert_eq!(spin_down_modes.len(), 31);

    // Verify spin-momentum locking: spin-up has positive group velocity, spin-down negative
    for m in &spin_up_modes {
        assert!(m.group_velocity_m_s > 0.0);
    }
    for m in &spin_down_modes {
        assert!(m.group_velocity_m_s < 0.0);
    }

    // Modal confinement within domain wall interface >= 85%
    let max_conf_up = spin_up_modes.iter().map(|m| m.confinement_ratio).fold(0.0, f64::max);
    assert!(max_conf_up >= 0.85, "Max edge confinement ratio should be >= 85%, got {}", max_conf_up);
}

#[test]
fn test_corner_zero_backscattering() {
    let lattice = SpinHallLattice::new(SpinHallParams::default());

    // 60-degree corner
    let (t_60, s11_60) = lattice.evaluate_corner_transmission(60.0);
    assert!(t_60 >= 0.990, "60-deg corner transmission should be >= 99.0%, got {}", t_60);
    assert!(s11_60 <= -22.0, "60-deg corner return loss should be <= -22 dB, got {}", s11_60);

    // 120-degree corner
    let (t_120, s11_120) = lattice.evaluate_corner_transmission(120.0);
    assert!(t_120 >= 0.990, "120-deg corner transmission should be >= 99.0%, got {}", t_120);
    assert!(s11_120 <= -22.0, "120-deg corner return loss should be <= -22 dB, got {}", s11_120);
}

#[test]
fn test_floquet_circulator_nonreciprocity_and_isolation() {
    let circulator = FloquetCirculator::new(FloquetCirculatorParams::default());
    let s = circulator.evaluate_s_matrix(circulator.params.center_freq_ghz);

    // Forward insertion loss IL <= 0.80 dB
    assert!(s.s21_db >= -0.80, "Forward transmission S21 should be >= -0.80 dB, got {}", s.s21_db);

    // Reverse isolation ISO >= 30.0 dB
    assert!(s.s12_db <= -30.0, "Reverse isolation S12 should be <= -30.0 dB, got {}", s.s12_db);

    // Input return loss RL <= -22.0 dB
    assert!(s.s11_db <= -22.0, "Return loss S11 should be <= -22.0 dB, got {}", s.s11_db);

    // Isolation contrast >= 28.0 dB
    let contrast = circulator.isolation_contrast_db();
    assert!(contrast >= 28.0, "Isolation contrast should be >= 28.0 dB, got {}", contrast);

    // Circulation bandwidth >= 25.0 MHz
    let bw = circulator.circulation_bandwidth_mhz();
    assert!(bw >= 25.0, "Circulation bandwidth should be >= 25.0 MHz, got {} MHz", bw);
}

#[test]
fn test_cryogenic_readout_noise_and_snr() {
    let engine = CryogenicReadoutEngine::new(CryogenicReadoutParams::default());

    let n_th = engine.thermal_phonon_occupancy();
    assert!(n_th < 0.10, "15 mK thermal phonon occupancy should be < 0.10, got {}", n_th);

    let n_add = engine.quantum_added_noise_quanta();
    assert!(n_add <= 0.55, "Quantum added noise should be <= 0.55 quanta, got {}", n_add);

    let n_leak = engine.amplifier_backaction_leakage_quanta();
    assert!(n_leak < 0.10, "Back-action noise leakage should be < 0.10 quanta, got {}", n_leak);

    let snr = engine.peak_readout_snr_db();
    assert!(snr >= 18.0, "Peak readout SNR should be >= 18.0 dB, got {} dB", snr);

    let spec = engine.compute_readout_spectrum(51, 60.0);
    assert_eq!(spec.len(), 51);
}

#[test]
fn test_master_orchestrator_10_point_audit() {
    let system = FloquetSpinHallCirculator::new(FloquetSpinHallParams::default());
    let audit = system.audit_spinhall_circulator();

    assert!(audit.pseudo_spin_degeneracy_passed);
    assert!(audit.spin_chern_quantized_passed);
    assert!(audit.helical_edge_confinement_passed);
    assert!(audit.corner_backscattering_suppressed);
    assert!(audit.forward_insertion_loss_passed);
    assert!(audit.reverse_isolation_passed);
    assert!(audit.return_loss_matched_passed);
    assert!(audit.circulation_bandwidth_passed);
    assert!(audit.cryogenic_noise_floor_passed);
    assert!(audit.readout_snr_passed);

    assert_eq!(audit.total_pass_score, 10);
    assert!(audit.all_passed);
}

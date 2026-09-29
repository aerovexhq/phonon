//! Integration tests for Terahertz antiferromagnetic magnon polaritons,
//! ultra-strong microcavity coupling, and vacuum Rabi splitting.

use phonon_models::afm_spintronics::{AfmMaterialParams, ThzCavityPolaritonParams};
use phonon_solver::afm_spintronics::AfmPolaritonSolver;

#[test]
fn test_afmr_thz_frequency_and_exchange_velocity() {
    // Exchange field 350 T, anisotropy field 1.2 T (nominal alpha-Fe2O3 / NiO)
    let afm = AfmMaterialParams::new(350.0, 1.2, 1.0e-11, 5.0e-4);
    let f_thz = afm.afmr_frequency_thz();

    assert!(
        (0.5..=2.0).contains(&f_thz),
        "AFMR resonance frequency must fall in the THz band (0.5 - 2.0 THz), got {:.3} THz",
        f_thz
    );

    let v_ex = afm.exchange_velocity_m_s();
    assert!(
        v_ex > 10_000.0,
        "Exchange spin-wave velocity must exceed 10,000 m/s (10 km/s), got {:.1} m/s",
        v_ex
    );
    assert!(
        v_ex < 100_000.0,
        "Exchange spin-wave velocity must be within relativistic physical limits (< 100 km/s), got {:.1} m/s",
        v_ex
    );
}

#[test]
fn test_vacuum_rabi_splitting_exceeds_100_ghz() {
    let afm = AfmMaterialParams::new(350.0, 1.2, 1.0e-11, 5.0e-4);
    let cavity = ThzCavityPolaritonParams::new(1.0, 250.0, 150.0);
    let solver = AfmPolaritonSolver::new(afm, cavity);

    let rabi_ghz = solver.solve_rabi_splitting_ghz();
    assert!(
        rabi_ghz > 100.0,
        "Vacuum Rabi splitting must exceed 100 GHz, got {:.2} GHz",
        rabi_ghz
    );
    assert!(
        (rabi_ghz - 300.0).abs() < 1e-3,
        "Rabi splitting at zero detuning must be 2*g = 300 GHz, got {:.2} GHz",
        rabi_ghz
    );

    let metrics = solver.solve_polaritons_at_k(0.0);
    assert!(
        metrics.normalized_coupling_eta >= 0.10,
        "Normalized coupling eta = g / omega_c must exceed 0.10 for ultra-strong coupling, got {:.3}",
        metrics.normalized_coupling_eta
    );
    assert!(
        metrics.cooperativity > 100.0,
        "Polariton cooperativity must greatly exceed 100, got {:.1}",
        metrics.cooperativity
    );
}

#[test]
fn test_polariton_hopfield_fractions_and_conservation() {
    let afm = AfmMaterialParams::new(350.0, 1.2, 1.0e-11, 5.0e-4);
    let f_res = afm.afmr_frequency_thz();
    let cavity = ThzCavityPolaritonParams::new(f_res, 250.0, 150.0);
    let solver = AfmPolaritonSolver::new(afm, cavity);

    // At k = 0 (exact resonance, zero detuning)
    let (u2_res, v2_res) = solver.solve_hopfield_fractions(0.0);
    assert!(
        (u2_res + v2_res - 1.0).abs() < 1e-5,
        "Hopfield coefficients must conserve normalization |u|^2 + |v|^2 = 1, got {:.5}",
        u2_res + v2_res
    );
    assert!(
        (u2_res - 0.50).abs() < 1e-4,
        "At exact resonance, photonic and magnonic content must be 50%/50%, got u^2={:.3}, v^2={:.3}",
        u2_res,
        v2_res
    );

    // At large k (large detuning, magnon branch diverges)
    let (u2_det, v2_det) = solver.solve_hopfield_fractions(1.0e8);
    assert!(
        (u2_det + v2_det - 1.0).abs() < 1e-5,
        "Hopfield normalization must hold at large detuning"
    );
}

#[test]
fn test_polariton_transmission_spectrum_doublet() {
    let afm = AfmMaterialParams::new(350.0, 1.2, 1.0e-11, 5.0e-4);
    let cavity = ThzCavityPolaritonParams::new(1.0, 250.0, 150.0);
    let solver = AfmPolaritonSolver::new(afm, cavity);

    let metrics = solver.solve_polaritons_at_k(0.0);
    let f_low = metrics.lower_polariton_thz;
    let f_up = metrics.upper_polariton_thz;

    let probe_freqs = vec![f_low, 1.0, f_up];
    let s21_spec = solver.solve_transmission_spectrum(&probe_freqs);

    let s21_low = s21_spec[0].1;
    let s21_mid = s21_spec[1].1;
    let s21_up = s21_spec[2].1;

    // Both polariton resonance peaks should have high transmission compared to the dip in between
    assert!(
        s21_low > s21_mid && s21_up > s21_mid,
        "Transmission must exhibit anti-crossing doublet: S21(lower)={:.3}, S21(mid)={:.3}, S21(upper)={:.3}",
        s21_low,
        s21_mid,
        s21_up
    );
}

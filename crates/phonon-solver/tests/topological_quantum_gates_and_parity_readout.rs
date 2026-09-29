//! Integration tests for topological quantum gates, fermion parity readout,
//! and cryogenic dephasing in acoustic Majorana networks.

use phonon_models::majorana_chiral_phonon::MajoranaBraidingParams;
use phonon_solver::majorana_chiral_phonon::MajoranaBraidingSolver;

#[test]
fn test_parity_readout_and_dephasing_bounds() {
    let params = MajoranaBraidingParams::default();
    let solver = MajoranaBraidingSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.parity_readout_snr_db >= 20.0,
        "Parity readout SNR {} dB must be >= 20.0 dB",
        metrics.parity_readout_snr_db
    );
    assert!(
        metrics.topological_dephasing_time_us >= 10.0,
        "Dephasing time {} us must be >= 10.0 us",
        metrics.topological_dephasing_time_us
    );
}

#[test]
fn test_readout_power_snr_scaling() {
    let p_low_power = MajoranaBraidingParams {
        saw_acoustic_power_uw: 40.0,
        ..Default::default()
    };
    let p_high_power = MajoranaBraidingParams {
        saw_acoustic_power_uw: 300.0,
        ..Default::default()
    };

    let solver_low = MajoranaBraidingSolver::new(p_low_power);
    let solver_high = MajoranaBraidingSolver::new(p_high_power);

    let snr_low = solver_low.compute_parity_readout_snr_db();
    let snr_high = solver_high.compute_parity_readout_snr_db();

    assert!(
        snr_high > snr_low,
        "Higher SAW readout power must improve SNR (high={} vs low={})",
        snr_high,
        snr_low
    );
    assert!(snr_low >= 20.0);
    assert!(snr_high >= 20.0);
}

#[test]
fn test_sub_kelvin_temperature_dephasing() {
    let p_cold = MajoranaBraidingParams {
        ambient_temperature_mk: 15.0,
        ..Default::default()
    };
    let p_warm = MajoranaBraidingParams {
        ambient_temperature_mk: 60.0,
        ..Default::default()
    };

    let solver_cold = MajoranaBraidingSolver::new(p_cold);
    let solver_warm = MajoranaBraidingSolver::new(p_warm);

    let t2_cold = solver_cold.compute_topological_dephasing_time_us();
    let t2_warm = solver_warm.compute_topological_dephasing_time_us();

    assert!(
        t2_cold > t2_warm,
        "Lower temperature must protect against thermal quasiparticle dephasing"
    );
    assert!(t2_cold >= 10.0);
    assert!(t2_warm >= 10.0);
}

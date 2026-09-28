use phonon_models::diamond_nv::{NvCenterConfig, OdmrSpectrumConfig};
use phonon_solver::diamond_nv::NvPulseDynamicsSolver;

#[test]
fn test_odmr_photoluminescence_spectrum() {
    let nv = NvCenterConfig::default();
    let odmr = OdmrSpectrumConfig {
        f_min_hz: 2.80e9,
        f_max_hz: 2.94e9,
        linewidth_hz: 5.0e6,
        contrast: 0.20,
        baseline_intensity: 1.0,
    };

    // Magnetic field along Z: 1.0 mT
    let b_vector = [0.0, 0.0, 1.0e-3];

    // Far off resonance, PL should be near baseline 1.0
    let pl_off = odmr.evaluate_point(&nv, b_vector, 2.70e9);
    assert!((pl_off - 1.0).abs() < 0.01);

    // Spectrum generation
    let spectrum = odmr.generate_spectrum(&nv, b_vector, 101);
    assert_eq!(spectrum.len(), 101);

    // Minimum PL should drop by contrast
    let min_pl = spectrum
        .iter()
        .map(|(_, pl)| *pl)
        .fold(1.0_f64, |a, b| a.min(b));
    assert!(
        min_pl < 0.98,
        "Minimum PL dip was {min_pl}, expected < 0.98"
    );
}

#[test]
fn test_ramsey_and_hahn_echo_decoherence() {
    let nv = NvCenterConfig::default();
    let solver = NvPulseDynamicsSolver::new(nv.clone());

    // At t = 0, full coherence
    let r0 = solver.ramsey_coherence(0.0, 1.0e6);
    let e0 = solver.hahn_echo_coherence(0.0);
    assert!((r0 - 1.0).abs() < 1e-6);
    assert!((e0 - 1.0).abs() < 1e-6);

    // Ramsey dephases rapidly at t = 3 * T2*
    let r_decay = solver.ramsey_coherence(3.0 * nv.dephasing_time_t2_star_s, 0.0);
    assert!(
        (r_decay - 0.5).abs() < 0.01,
        "Ramsey dephased value was {r_decay}"
    );

    // Hahn echo retains coherence far beyond T2* (e.g. at 10 * T2* = 25 us << T2 = 250 us)
    let e_extended = solver.hahn_echo_coherence(10.0 * nv.dephasing_time_t2_star_s);
    assert!(
        e_extended > 0.95,
        "Hahn echo coherence was {e_extended}, expected > 0.95"
    );

    // Hahn echo dephases at t = 2 * T2
    let e_decay = solver.hahn_echo_coherence(2.0 * nv.coherence_time_t2_echo_s);
    assert!((e_decay - 0.5).abs() < 0.01);
}

#[test]
fn test_xy8_nanoscale_nmr_proton_resonance() {
    let nv = NvCenterConfig::default();
    let solver = NvPulseDynamicsSolver::new(nv);

    let proton_larmor_hz = 128.0e3; // 128 kHz Larmor frequency
    let b_rms = 1.5e-7; // 150 nT fluctuating field from nanoscale proton volume
    let num_pulses = 32;

    // Resonant pulse delay: tau0 = 1 / (4 * f_L)
    let tau0_res = 1.0 / (4.0 * proton_larmor_hz);
    let w_res = solver.nanoscale_nmr_coherence(tau0_res, num_pulses, proton_larmor_hz, b_rms);

    // Off-resonant pulse delay: 20% detuned
    let tau0_off = 1.0 / (4.0 * (proton_larmor_hz * 1.20));
    let w_off = solver.nanoscale_nmr_coherence(tau0_off, num_pulses, proton_larmor_hz, b_rms);

    // Distinct NMR dip at resonance
    assert!(
        w_res < w_off,
        "Resonant coherence {w_res} should be lower than off-resonance {w_off}"
    );
    let dip_depth = w_off - w_res;
    assert!(
        dip_depth > 0.15,
        "NMR resonance dip depth was {dip_depth}, expected > 0.15"
    );

    // Solve full NMR spectrum
    let spectrum =
        solver.solve_nmr_spectrum(100.0e3, 150.0e3, 21, num_pulses, proton_larmor_hz, b_rms);
    assert_eq!(spectrum.len(), 21);

    // Check that minimum coherence is close to target Larmor frequency
    let min_point = spectrum
        .into_iter()
        .min_by(|a, b| a.coherence.partial_cmp(&b.coherence).unwrap())
        .unwrap();
    assert!((min_point.filter_frequency_hz - proton_larmor_hz).abs() < 5.0e3);
}

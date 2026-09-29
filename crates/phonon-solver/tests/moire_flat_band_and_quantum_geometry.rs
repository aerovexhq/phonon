use phonon_models::fractional_chern::{MoireFlatBand, MoireLatticeParams};

#[test]
fn test_moire_lattice_and_flat_band_quenching() {
    let params = MoireLatticeParams::twisted_mote2_125();
    let lm = params.moire_period_m();

    // Moiré period should be around 16.1 nm for 1.25° MoTe2
    assert!(
        (lm - 1.61e-8).abs() < 1e-9,
        "Moiré period must be approximately 16.1 nm: got {} nm",
        lm * 1e9
    );

    let ec = params.characteristic_coulomb_ev();
    assert!(
        ec > 0.010,
        "Characteristic Coulomb interaction must exceed 10 meV: got {} eV",
        ec
    );

    let flat_band = MoireFlatBand::new(params, 1);
    assert_eq!(flat_band.chern_number, 1);

    // Bandwidth must be quenched (< 5 meV)
    assert!(
        flat_band.bandwidth_ev < 0.005,
        "Flat-band kinetic bandwidth must be quenched: got {} eV",
        flat_band.bandwidth_ev
    );

    // Correlation ratio U/W must exceed 3.0 (strongly correlated regime)
    let u_w = flat_band.correlation_ratio();
    assert!(
        u_w > 3.0,
        "Correlation ratio U/W must exceed 3.0: got {}",
        u_w
    );
}

#[test]
fn test_quantum_geometry_and_fubini_study_trace_condition() {
    let params = MoireLatticeParams::twisted_mote2_125();
    let flat_band = MoireFlatBand::new(params, 1);

    let qg = flat_band.evaluate_quantum_geometry(0.0, 0.0);

    // Berry curvature must be non-zero
    assert!(
        qg.berry_curvature_m2 > 0.0,
        "Peak Berry curvature must be positive for C = 1"
    );

    // Fubini-Study trace condition: tr(g) >= |Omega_z|
    assert!(
        qg.trace_ratio >= 1.0,
        "Trace condition ratio must satisfy eta_FS >= 1.0: got {}",
        qg.trace_ratio
    );

    // In topological flat band with LLL-like geometry, eta_FS should be close to 1.0 (<= 1.15)
    assert!(
        qg.trace_ratio <= 1.15,
        "Trace condition ratio must be near ideal LLL limit (<= 1.15): got {}",
        qg.trace_ratio
    );

    // Determinant condition: det(g) - 1/4 Omega_z^2 >= 0
    assert!(
        qg.det_margin >= -1e-50,
        "Determinant margin must be non-negative: got {}",
        qg.det_margin
    );
}

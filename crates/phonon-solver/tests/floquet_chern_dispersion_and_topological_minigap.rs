#![deny(unsafe_code)]

use phonon_models::floquet_acoustic_chern::FloquetAcousticChernParams;
use phonon_solver::floquet_acoustic_chern::FloquetAcousticChernSolver;

#[test]
fn test_default_floquet_chern_insulator() {
    let params = FloquetAcousticChernParams::default();
    let solver = FloquetAcousticChernSolver::new(params);
    let metrics = solver.solve();

    // Chern number |C| = 1.0
    assert_eq!(
        metrics.chern_number.abs(),
        1.0,
        "Chern number {} != 1.0",
        metrics.chern_number
    );

    // Topological minigap >= 2.5 MHz
    assert!(
        metrics.topological_minigap_mhz >= 2.5,
        "Minigap {} MHz < 2.5 MHz",
        metrics.topological_minigap_mhz
    );

    // Forward bend efficiency >= 92.0%
    assert!(
        metrics.forward_bend_efficiency_pct >= 92.0,
        "Bend efficiency {}% < 92.0%",
        metrics.forward_bend_efficiency_pct
    );

    // Reverse isolation >= 30.0 dB
    assert!(
        metrics.reverse_isolation_db >= 30.0,
        "Reverse isolation {} dB < 30.0 dB",
        metrics.reverse_isolation_db
    );

    // Chiral edge velocity > 1000 m/s
    assert!(
        metrics.chiral_edge_velocity_m_s > 1000.0,
        "Edge velocity {} m/s <= 1000 m/s",
        metrics.chiral_edge_velocity_m_s
    );

    assert!(
        metrics.is_physically_compliant,
        "Default params must be compliant"
    );
}

#[test]
fn test_minigap_scaling_with_strain_and_frequency() {
    let p_low_strain = FloquetAcousticChernParams {
        dynamic_strain_amplitude: 1.5e-4,
        ..Default::default()
    };
    let s_low = FloquetAcousticChernSolver::new(p_low_strain);
    let m_low = s_low.solve();

    let p_high_strain = FloquetAcousticChernParams {
        dynamic_strain_amplitude: 3.5e-4,
        ..Default::default()
    };
    let s_high = FloquetAcousticChernSolver::new(p_high_strain);
    let m_high = s_high.solve();

    assert!(
        m_high.topological_minigap_mhz > m_low.topological_minigap_mhz,
        "Higher strain must open larger minigap: {} vs {}",
        m_high.topological_minigap_mhz,
        m_low.topological_minigap_mhz
    );

    let p_fast_drive = FloquetAcousticChernParams {
        floquet_drive_freq_mhz: 120.0,
        ..Default::default()
    };
    let s_fast = FloquetAcousticChernSolver::new(p_fast_drive);
    let m_fast = s_fast.solve();

    let p_slow_drive = FloquetAcousticChernParams {
        floquet_drive_freq_mhz: 60.0,
        ..Default::default()
    };
    let s_slow = FloquetAcousticChernSolver::new(p_slow_drive);
    let m_slow = s_slow.solve();

    assert!(
        m_slow.topological_minigap_mhz > m_fast.topological_minigap_mhz,
        "Lower drive freq must produce larger 1/Omega minigap: {} vs {}",
        m_slow.topological_minigap_mhz,
        m_fast.topological_minigap_mhz
    );
}

#![deny(unsafe_code)]

use phonon_solver::valley_majorana_router::{
    ChiralBeamSplitterParams, ChiralBeamSplitterSolver, FaultTolerantInterconnectSolver,
    InterconnectBusParams, MajoranaTransmonCouplingSolver, MajoranaTransmonParams,
    ValleyMajoranaLatticeParams, ValleyMajoranaLatticeSolver, ValleyMajoranaRouterProcessor,
};

#[test]
fn test_valley_majorana_lattice_topology_and_bends() {
    let params = ValleyMajoranaLatticeParams::default();
    let solver = ValleyMajoranaLatticeSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // Verify valley bandgap Delta >= 18.0 MHz
    assert!(
        metrics.valley_bandgap_mhz >= 18.0,
        "Valley bandgap {} MHz should be >= 18.0 MHz",
        metrics.valley_bandgap_mhz
    );

    // Verify valley Chern number difference |Delta C_V| == 2
    assert_eq!(
        metrics.delta_valley_chern_number.abs(), 2,
        "Delta C_V {} must be 2",
        metrics.delta_valley_chern_number
    );
    assert_eq!(metrics.valley_k_chern_number, 1);
    assert_eq!(metrics.valley_kprime_chern_number, -1);

    // Verify edge mode spatial localization depth xi <= 2.0 unit cells
    assert!(
        metrics.edge_mode_decay_depth_cells <= 2.0,
        "Decay depth {} cells should be <= 2.0",
        metrics.edge_mode_decay_depth_cells
    );

    // Verify sharp bend transmission ratio >= 0.940
    assert!(
        metrics.bend_transmission_ratio >= 0.940,
        "Bend transmission ratio {} should be >= 0.940",
        metrics.bend_transmission_ratio
    );

    // Verify dispersion curve
    let dispersion = solver.compute_dispersion(80);
    assert_eq!(dispersion.len(), 80);

    // Verify spatial mode intensity peaks at the center domain wall
    let spatial = solver.compute_spatial_mode();
    assert!(!spatial.is_empty());
    let center_pt = spatial.iter().find(|p| p.cell_index_y == 0).unwrap();
    let edge_pt = spatial.first().unwrap();
    assert!(center_pt.energy_density > edge_pt.energy_density);
}

#[test]
fn test_majorana_transmon_coherent_coupling() {
    let params = MajoranaTransmonParams::default();
    let solver = MajoranaTransmonCouplingSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify electromechanical coupling rate g >= 25.0 MHz
    assert!(
        metrics.coupling_rate_g_mhz >= 25.0,
        "Coupling g {} MHz should be >= 25.0 MHz",
        metrics.coupling_rate_g_mhz
    );

    // Verify strong coupling cooperativity C >= 150.0
    assert!(
        metrics.cooperativity >= 150.0,
        "Cooperativity {} should be >= 150.0",
        metrics.cooperativity
    );

    // Verify parity-dependent dispersive shift chi_MZM >= 3.5 MHz
    assert!(
        metrics.dispersive_shift_chi_mhz >= 3.5,
        "Dispersive shift {} MHz should be >= 3.5 MHz",
        metrics.dispersive_shift_chi_mhz
    );

    // Verify parity readout contrast >= 85.0%
    assert!(
        metrics.parity_readout_contrast_pct >= 85.0,
        "Readout contrast {} % should be >= 85.0%",
        metrics.parity_readout_contrast_pct
    );

    // Verify transmission spectrum doublet
    let spectrum = solver.compute_spectrum(60);
    assert_eq!(spectrum.len(), 60);
}

#[test]
fn test_chiral_beam_splitter_s_matrix() {
    let params = ChiralBeamSplitterParams::default();
    let solver = ChiralBeamSplitterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify forward insertion loss <= 0.45 dB
    assert!(
        metrics.insertion_loss_db <= 0.45,
        "Insertion loss {} dB should be <= 0.45 dB",
        metrics.insertion_loss_db
    );

    // Verify backward cross-port isolation >= 35.0 dB
    assert!(
        metrics.backward_isolation_db >= 35.0,
        "Isolation {} dB should be >= 35.0 dB",
        metrics.backward_isolation_db
    );

    // Verify return loss >= 22.0 dB
    assert!(
        metrics.return_loss_db >= 22.0,
        "Return loss {} dB should be >= 22.0 dB",
        metrics.return_loss_db
    );

    // Verify wavepacket transfer fidelity >= 99.0%
    assert!(
        metrics.transfer_fidelity_pct >= 99.0,
        "Transfer fidelity {} % should be >= 99.0%",
        metrics.transfer_fidelity_pct
    );

    // Verify frequency S-parameter curve
    let s_params = solver.compute_s_parameters(60);
    assert_eq!(s_params.len(), 60);
}

#[test]
fn test_fault_tolerant_interconnect_cryogenic_coherence() {
    let params = InterconnectBusParams::default();
    let solver = FaultTolerantInterconnectSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify thermal noise occupancy at 20 mK <= 0.05 quanta
    assert!(
        metrics.thermal_noise_occupancy <= 0.05,
        "Thermal occupancy {} should be <= 0.05 quanta",
        metrics.thermal_noise_occupancy
    );

    // Verify quasiparticle poisoning rate <= 25.0 Hz
    assert!(
        metrics.quasiparticle_poisoning_rate_hz <= 25.0,
        "Quasiparticle rate {} Hz should be <= 25.0 Hz",
        metrics.quasiparticle_poisoning_rate_hz
    );

    // Verify dephasing coherence time T2* >= 45.0 us
    assert!(
        metrics.dephasing_time_t2_us >= 45.0,
        "Dephasing time T2* {} us should be >= 45.0 us",
        metrics.dephasing_time_t2_us
    );

    // Verify end-to-end fidelity >= 90.0%
    assert!(
        metrics.end_to_end_fidelity_pct >= 90.0,
        "End-to-end fidelity {} % should be >= 90.0%",
        metrics.end_to_end_fidelity_pct
    );

    // Verify temperature curve
    let thermal_curve = solver.compute_thermal_curve(40);
    assert_eq!(thermal_curve.len(), 40);
}

#[test]
fn test_valley_majorana_router_10_point_audit() {
    let processor = ValleyMajoranaRouterProcessor::default();
    let report = processor.evaluate_audit();

    println!("{}", report.summary());

    assert!(report.valley_bandgap_pass, "Criterion 1 failed");
    assert!(report.valley_chern_pass, "Criterion 2 failed");
    assert!(report.edge_decay_depth_pass, "Criterion 3 failed");
    assert!(report.bend_transmission_pass, "Criterion 4 failed");
    assert!(report.transmon_coupling_pass, "Criterion 5 failed");
    assert!(report.cooperativity_pass, "Criterion 6 failed");
    assert!(report.dispersive_shift_pass, "Criterion 7 failed");
    assert!(report.chiral_isolation_pass, "Criterion 8 failed");
    assert!(report.transfer_fidelity_pass, "Criterion 9 failed");
    assert!(report.thermal_noise_pass, "Criterion 10 failed");

    assert!(report.all_passed(), "Expected all 10 physics audit criteria to pass");
    assert_eq!(report.score(), (10, 10));
}

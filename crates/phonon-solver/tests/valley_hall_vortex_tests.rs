#![deny(unsafe_code)]

//! Test suite for Phase 372: Acoustic Valley-Hall Vortex Pumping & Synthetic Chiral Gauge Field Engine.

use phonon_solver::valley_hall_vortex::{
    AcousticValley, DomainWallKind, ValleyHallParams, ValleyHallPhase, ValleyHamiltonian,
    ValleyRibbonParams, VortexPumpingEngine,
};

#[test]
fn test_valley_hall_bandgap_and_phase() {
    // 1. Dirac semimetal (Delta = 0)
    let p_dirac = ValleyHallParams::dirac_semimetal();
    let h_dirac = ValleyHamiltonian::new(p_dirac);
    assert_eq!(h_dirac.phase(), ValleyHallPhase::DiracSemimetal);
    assert_eq!(p_dirac.valley_bandgap_khz(), 0.0);

    let (lower, upper) = h_dirac.valley_dispersion_at(0.0, 0.0, AcousticValley::ValleyK);
    assert!((upper - lower).abs() < 1e-12, "Dirac point must be strictly gapless at Delta = 0");

    // 2. Valley-Hall Insulator (Delta = 0.8 kHz)
    let p_vhi = ValleyHallParams::valley_hall_insulator(0.8);
    let h_vhi = ValleyHamiltonian::new(p_vhi);
    assert_eq!(h_vhi.phase(), ValleyHallPhase::ValleyHallInsulator);
    assert!((p_vhi.valley_bandgap_khz() - 1.6).abs() < 1e-12);

    let (low_v, up_v) = h_vhi.valley_dispersion_at(0.0, 0.0, AcousticValley::ValleyK);
    assert!(((up_v - low_v) - 1.6).abs() < 1e-6, "Valley bandgap must equal 2*Delta");

    // 3. Strained Pseudomagnetic Metamaterial
    let p_strain = ValleyHallParams::pseudo_landau_quantized(0.20);
    let h_strain = ValleyHamiltonian::new(p_strain);
    assert_eq!(h_strain.phase(), ValleyHallPhase::PseudoLandauQuantized);
    assert!(p_strain.synthetic_pseudomagnetic_field() > 0.01);
}

#[test]
fn test_valley_chern_numbers_and_difference() {
    // Positive mass detuning (Delta > 0)
    let p_pos = ValleyHallParams::valley_hall_insulator(0.8);
    let h_pos = ValleyHamiltonian::new(p_pos);

    let c_k = h_pos.valley_chern_number(AcousticValley::ValleyK);
    let c_kp = h_pos.valley_chern_number(AcousticValley::ValleyKPrime);
    assert_eq!(c_k, -0.5);
    assert_eq!(c_kp, 0.5);

    let delta_c = h_pos.valley_chern_difference();
    assert_eq!(delta_c, -1.0);

    // Negative mass detuning (Delta < 0)
    let mut p_neg = p_pos;
    p_neg.mass_detuning_delta_khz = -0.8;
    let h_neg = ValleyHamiltonian::new(p_neg);

    assert_eq!(h_neg.valley_chern_number(AcousticValley::ValleyK), 0.5);
    assert_eq!(h_neg.valley_chern_number(AcousticValley::ValleyKPrime), -0.5);
    assert_eq!(h_neg.valley_chern_difference(), 1.0);

    // Berry curvature evaluation
    let bc_k = h_neg.berry_curvature_at(0.0, 0.0, AcousticValley::ValleyK);
    let bc_kp = h_neg.berry_curvature_at(0.0, 0.0, AcousticValley::ValleyKPrime);
    assert!(bc_k > 0.0, "Berry curvature at K must be positive for Delta < 0");
    assert!(bc_kp < 0.0, "Berry curvature at K' must be negative for Delta < 0");
    assert!((bc_k + bc_kp).abs() < 1e-10, "Opposite valleys must have opposite Berry curvature");
}

#[test]
fn test_pseudo_landau_quantization() {
    let p = ValleyHallParams::pseudo_landau_quantized(0.25);
    let h = ValleyHamiltonian::new(p);

    let levels = h.pseudo_landau_levels(5);
    assert!(!levels.is_empty());

    // Find n = 0 level
    let level_0 = levels.iter().find(|l| l.index_n == 0).expect("n=0 level must exist");
    assert!((level_0.energy_offset_khz - p.mass_detuning_delta_khz).abs() < 1e-6);

    // Verify sqrt(n) scaling for n = 1, 4
    let level_1_pos = levels.iter().find(|l| l.index_n == 1 && l.energy_offset_khz > 0.0).expect("n=1 pos");
    let level_4_pos = levels.iter().find(|l| l.index_n == 4 && l.energy_offset_khz > 0.0).expect("n=4 pos");

    // E_4 / E_1 must equal sqrt(4) / sqrt(1) = 2.0
    let ratio = level_4_pos.energy_offset_khz / level_1_pos.energy_offset_khz;
    assert!((ratio - 2.0).abs() < 1e-4, "Pseudo-Landau levels must exhibit relativistic sqrt(n) scaling");
}

#[test]
fn test_ribbon_domain_wall_edge_modes() {
    let params = ValleyRibbonParams {
        bulk_params: ValleyHallParams::valley_hall_insulator(1.0),
        num_cells_y: 16,
        interface_kind: DomainWallKind::ZigzagInterface,
        vortex_charge_l: 1,
        pumping_freq_hz: 100.0,
        defect_angle_deg: 0.0,
    };

    let engine = VortexPumpingEngine::new(params);
    assert!(!engine.ribbon_modes.is_empty());

    // Locate topological edge states
    let edge_modes: Vec<_> = engine.ribbon_modes.iter().filter(|m| m.is_edge_mode).collect();
    assert!(!edge_modes.is_empty(), "Domain wall interface must host in-gap edge states");

    // Check localization at domain wall interface
    for mode in &edge_modes {
        assert!(
            mode.localization_ratio >= 0.70,
            "Edge mode must be strongly confined to interface (ratio={:.2})",
            mode.localization_ratio
        );
    }
}

#[test]
fn test_topological_vortex_charge_pumping() {
    // 1. Charge l = +1 pumping
    let params_1 = ValleyRibbonParams {
        vortex_charge_l: 1,
        ..Default::default()
    };
    let engine_1 = VortexPumpingEngine::new(params_1);
    assert_eq!(engine_1.metrics.quantized_pumped_charge, 1.0);

    let last_pt_1 = engine_1.pumping_cycle.last().expect("Cycle point exists");
    assert!((last_pt_1.cumulative_charge_pumped - 1.0).abs() < 0.15);

    // 2. Charge l = +2 pumping
    let params_2 = ValleyRibbonParams {
        vortex_charge_l: 2,
        ..Default::default()
    };
    let engine_2 = VortexPumpingEngine::new(params_2);
    assert_eq!(engine_2.metrics.quantized_pumped_charge, 2.0);

    let last_pt_2 = engine_2.pumping_cycle.last().expect("Cycle point exists");
    assert!((last_pt_2.cumulative_charge_pumped - 2.0).abs() < 0.20);
}

#[test]
fn test_valley_router_s_parameters_and_defect_immunity() {
    let mut params = ValleyRibbonParams::default();
    params.defect_angle_deg = 0.0;

    let engine_clean = VortexPumpingEngine::new(params);
    let m_clean = engine_clean.metrics;

    assert!(m_clean.s21_transmission_db >= -0.8, "Transmission must be >= -0.8 dB");
    assert!(m_clean.s31_isolation_db <= -28.0, "Crosstalk isolation must be <= -28 dB");
    assert!(m_clean.valley_directivity_db >= 28.0, "Directivity must be >= 28 dB");
    assert!(m_clean.edge_confinement_pct >= 82.0, "Confinement must be >= 82%");

    // Defect immunity with 60-degree bend
    params.defect_angle_deg = 60.0;
    let engine_defect = VortexPumpingEngine::new(params);
    let m_defect = engine_defect.metrics;

    assert!(
        m_defect.defect_transmission_pct >= 90.0,
        "Transmission around sharp 60-degree bend must be >= 90% (got {:.1}%)",
        m_defect.defect_transmission_pct
    );
}

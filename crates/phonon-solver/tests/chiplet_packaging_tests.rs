#![deny(unsafe_code)]

//! Unit and integration test suite for 2.5D/3D Multi-Die & Chiplet Packaging Co-Simulator.

use phonon_solver::chiplet_packaging::{
    compute_s_parameters, evaluate_thermo_mechanics, evaluate_ucie_phy, extract_rdl_rlgc,
    extract_tsv_rlgc, ChipletPackagingCoSimulator, PackageStackGeometry, PackagingArchitecture,
    RdlGeometry, ThermalCycleParams, TsvGeometry, UcieDataRateGbps, UciePackageType,
    UciePhyParams,
};
use std::time::Instant;

#[test]
fn test_chiplet_packaging_new_fast_cold_boot() {
    let start = Instant::now();
    let sim = ChipletPackagingCoSimulator::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "ChipletPackagingCoSimulator::new_fast() must boot in sub-5ms, took {:?}",
        elapsed
    );

    let rep = sim.report();
    assert_eq!(rep.architecture, PackagingArchitecture::CoWoS_S);
    assert!(rep.eye_height_mv > 0.0);
    assert!(rep.eye_width_ui > 0.0 && rep.eye_width_ui <= 1.0);
    assert!(rep.tsv_inductance_ph > 0.0);
    assert!(rep.tsv_capacitance_ff > 0.0);
    assert!(rep.rdl_z0_ohm > 0.0);
    assert!(rep.package_bow_warpage_um > 0.0);
    assert!(rep.corner_bump_shear_stress_mpa > 0.0);
    assert!(rep.thermal_fatigue_cycles > 0);
    assert!(rep.is_fully_qualified);
}

#[test]
fn test_ucie_phy_channel_and_eye_diagram() {
    let params = UciePhyParams {
        package_type: UciePackageType::AdvancedSiliconBridge,
        data_rate: UcieDataRateGbps::Rate16Gbps,
        tx_swing_v: 0.50,
        trace_length_mm: 2.0,
        random_jitter_ps_rms: 0.45,
        deterministic_jitter_ps: 3.2,
        rx_equalization_boost_db: 4.0,
    };

    let (metrics, sample) = evaluate_ucie_phy(&params);

    assert_eq!(metrics.ui_ps, 62.5); // 1000 / 16 Gbps = 62.5 ps
    assert!(metrics.insertion_loss_db < 0.0);
    assert!(metrics.total_jitter_ps > params.deterministic_jitter_ps);
    assert!(metrics.eye_width_ps > 0.0);
    assert!(metrics.eye_width_ui > 0.40);
    assert!(metrics.eye_height_mv >= 40.0);
    assert!(metrics.energy_efficiency_pj_bit < 0.50);
    assert!(metrics.is_compliant);

    assert!(!sample.upper_inner_contour.is_empty());
    assert!(!sample.lower_inner_contour.is_empty());
    assert_eq!(sample.upper_inner_contour.len(), sample.lower_inner_contour.len());
}

#[test]
fn test_tsv_and_rdl_parasitics_extraction() {
    let tsv_geom = TsvGeometry {
        diameter_um: 5.0,
        height_um: 50.0,
        liner_thickness_um: 0.25,
        pitch_um: 20.0,
    };

    let tsv_rlgc = extract_tsv_rlgc(&tsv_geom);
    assert!(tsv_rlgc.r_dc_mohm > 10.0 && tsv_rlgc.r_dc_mohm < 100.0);
    assert!(tsv_rlgc.l_ph > 5.0 && tsv_rlgc.l_ph < 80.0);
    assert!(tsv_rlgc.c_ox_ff > 10.0 && tsv_rlgc.c_ox_ff < 150.0);
    assert!(tsv_rlgc.c_total_ff > 5.0);

    let rdl_geom = RdlGeometry {
        width_um: 2.0,
        space_um: 2.0,
        thickness_um: 2.0,
        length_mm: 1.5,
        dielectric_eps_r: 3.2,
    };

    let rdl_rlgc = extract_rdl_rlgc(&rdl_geom);
    assert!(rdl_rlgc.r_dc_ohm > 1.0 && rdl_rlgc.r_dc_ohm < 25.0);
    assert!(rdl_rlgc.l_nh > 0.1);
    assert!(rdl_rlgc.c_pf > 0.05);
    assert!(rdl_rlgc.z0_ohm > 20.0 && rdl_rlgc.z0_ohm < 120.0);
    assert!(rdl_rlgc.t_pd_ps > 5.0);

    let s_params = compute_s_parameters(&tsv_rlgc, &rdl_rlgc);
    assert_eq!(s_params.len(), 50);

    // Insertion loss should degrade (more negative) at higher frequency
    let s21_1ghz = s_params[0].s21_db;
    let s21_50ghz = s_params[49].s21_db;
    assert!(s21_50ghz <= s21_1ghz);
    assert!(s21_1ghz <= 0.0);
    assert!(s_params[0].s11_db < 0.0);
}

#[test]
fn test_thermo_mechanical_warpage_and_fatigue() {
    let geom = PackageStackGeometry::default();
    let thermal = ThermalCycleParams::default();

    let report = evaluate_thermo_mechanics(&geom, &thermal);

    assert!(report.delta_cte_ppm_k > 0.0);
    assert!(report.max_cycle_warpage_um > 0.0);
    assert!(report.is_coplanar);
    assert!(report.dnp_mm > 0.0);
    assert!(report.corner_bump_shear_strain_pct > 0.0);
    assert!(report.corner_bump_shear_stress_mpa > 0.0);
    assert!(report.fatigue_cycles_to_failure >= 1_000);
}

#[test]
fn test_chiplet_packaging_architecture_presets() {
    let mut sim = ChipletPackagingCoSimulator::new();

    // Preset 1: Intel EMIB
    sim.apply_architecture_preset(PackagingArchitecture::Intel_EMIB);
    assert_eq!(sim.architecture, PackagingArchitecture::Intel_EMIB);
    assert_eq!(sim.stack_geom.bump_pitch_um, 45.0);
    assert_eq!(sim.tsv_geom.diameter_um, 8.0);

    // Preset 2: Standard Organic 2.5D
    sim.apply_architecture_preset(PackagingArchitecture::Organic_2_5D);
    assert_eq!(sim.architecture, PackagingArchitecture::Organic_2_5D);
    assert_eq!(sim.ucie_params.package_type, UciePackageType::StandardOrganic);
    assert_eq!(sim.stack_geom.bump_pitch_um, 110.0);

    // Preset 3: 3D SoIC
    sim.apply_architecture_preset(PackagingArchitecture::SoIC_3D);
    assert_eq!(sim.architecture, PackagingArchitecture::SoIC_3D);
    assert_eq!(sim.ucie_params.data_rate, UcieDataRateGbps::Rate32Gbps);
    assert_eq!(sim.stack_geom.bump_pitch_um, 9.0);

    // Preset 4: TSMC CoWoS-S
    sim.apply_architecture_preset(PackagingArchitecture::CoWoS_S);
    assert_eq!(sim.architecture, PackagingArchitecture::CoWoS_S);
    assert!(sim.report().is_fully_qualified);
}

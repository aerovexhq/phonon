//! Integration tests for CPU floorplan assembly, 2D thermal diffusion, and thermo-mechanical stress.

use phonon_models::hetero::{
    BlackElectromigrationModel, BlockAllocationMap, HeteroMaterialType, ProcessorBlockType,
    RiscVFloorplanBuilder, ThermalHotspotSolver, ThermoMechanicalStressModel,
};

#[test]
fn test_riscv_floorplan_geometry_and_power() {
    let alloc = BlockAllocationMap::synthesized_heterogeneous();
    let floorplan = RiscVFloorplanBuilder::build(&alloc);

    assert_eq!(floorplan.blocks.len(), 8);
    assert!((floorplan.die_width_um - 500.0).abs() < 1e-3);
    assert!((floorplan.die_height_um - 250.0).abs() < 1e-3);
    assert!((floorplan.total_area_mm2() - 0.125).abs() < 1e-3);

    let v_dd = 0.8;
    let freq_ghz = 4.0;
    let p_dyn = floorplan.total_dynamic_power_mw(v_dd, freq_ghz);
    let p_stat = floorplan.total_static_power_mw(v_dd);
    let p_tot = floorplan.total_power_mw(v_dd, freq_ghz);

    assert!(p_dyn > 0.0);
    assert!(p_stat > 0.0);
    assert!((p_tot - (p_dyn + p_stat)).abs() < 1e-6);

    // ALU block check
    let alu = floorplan
        .get_block(ProcessorBlockType::ExecutionAlu)
        .unwrap();
    assert_eq!(alu.material, HeteroMaterialType::InGaAsNmos);
    assert!(alu.power_density_w_per_cm2(v_dd, freq_ghz) > 0.5);
}

#[test]
fn test_2d_thermal_hotspot_diffusion_convergence() {
    let alloc = BlockAllocationMap::synthesized_heterogeneous();
    let floorplan = RiscVFloorplanBuilder::build(&alloc);

    let solver = ThermalHotspotSolver::default();
    let report = solver.solve(&floorplan, 0.8, 3.5);

    // Peak temperature must be above ambient (300 K = 26.85 °C) and physically bounded (< 100 °C)
    assert!(
        report.t_peak_c > 26.85,
        "Expected temperature rise above ambient from CPU active dissipation"
    );
    assert!(
        report.t_peak_c < 100.0,
        "Peak temperature should remain within safe limits with vapor chamber"
    );
    assert!(report.t_mean_c() <= report.t_peak_c);
    assert!(report.t_mean_c() >= 26.85);

    // The hottest block should typically be ExecutionAlu or ClockDistribution
    assert!(
        report.hottest_block_name.contains("ALU") || report.hottest_block_name.contains("Clock"),
        "Hottest block was: {}",
        report.hottest_block_name
    );
}

#[test]
fn test_thermo_mechanical_stress_and_reliability() {
    let t_junction_k = 360.0; // ~87 °C
    let t_ref_k = 300.0; // room temperature assembly

    // Silicon on Silicon substrate -> zero CTE mismatch stress
    let si_stress = ThermoMechanicalStressModel::evaluate_block_stress(
        HeteroMaterialType::SiliconGaa,
        t_junction_k,
        t_ref_k,
    );
    assert!(si_stress.thermal_stress_mpa.abs() < 1e-6);
    assert!(si_stress.is_mechanically_sound);

    // InGaAs on Silicon substrate
    let ingaas_stress = ThermoMechanicalStressModel::evaluate_block_stress(
        HeteroMaterialType::InGaAsNmos,
        t_junction_k,
        t_ref_k,
    );
    // CTE mismatch creates bi-axial stress
    assert!(ingaas_stress.thermal_stress_mpa.abs() > 0.0);
    // Should still be well below critical 800 MPa yield threshold
    assert!(ingaas_stress.is_mechanically_sound);
    assert!(ingaas_stress.thermal_stress_mpa.abs() < 800.0);
}

#[test]
fn test_black_electromigration_cnt_vs_copper() {
    let t_k = 370.0; // ~97 °C
    let j_density = 5.0e6; // 5 MA/cm^2 (very aggressive clock wire current density)

    let mttf_cu = BlackElectromigrationModel::compute_mttf_years(
        HeteroMaterialType::SiliconGaa, // Standard Cu wire on Si
        j_density,
        t_k,
    );
    let mttf_cnt = BlackElectromigrationModel::compute_mttf_years(
        HeteroMaterialType::CntBundleInterconnect, // Carbon Nanotube bundle
        j_density,
        t_k,
    );

    // CNT bundles have activation energy 2.8 eV vs 0.9 eV for Cu
    // Therefore, at high temperatures and current densities, CNT MTTF should be orders of magnitude longer
    assert!(
        mttf_cnt > mttf_cu * 1000.0,
        "CNT bundle MTTF ({} yrs) should be > 1000x Cu MTTF ({} yrs)",
        mttf_cnt,
        mttf_cu
    );
}

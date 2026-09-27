//! Integration tests for heterogeneous material allocation and physical properties.

use phonon_models::hetero::{
    BlockAllocationMap, HeteroMaterialProperties, HeteroMaterialType, ProcessorBlockType,
};
use phonon_solver::TimingPathAnalyzer;

#[test]
fn test_all_hetero_materials_physical_properties() {
    let materials = [
        HeteroMaterialType::SiliconGaa,
        HeteroMaterialType::StrainedGePmos,
        HeteroMaterialType::InGaAsNmos,
        HeteroMaterialType::WideBandgapGan,
        HeteroMaterialType::SiliconCarbide,
        HeteroMaterialType::IgzoBeol,
        HeteroMaterialType::CntBundleInterconnect,
        HeteroMaterialType::TopologicalInsulator,
    ];

    for &mat in &materials {
        let props: HeteroMaterialProperties = mat.properties();

        // Bandgap must be non-negative
        assert!(
            props.bandgap_ev >= 0.0,
            "Material {:?} has invalid bandgap",
            mat
        );
        // Thermal conductivity must be positive
        assert!(
            props.thermal_conductivity_w_mk > 0.0,
            "Material {:?} has invalid thermal conductivity",
            mat
        );
        // Young's modulus must be positive
        assert!(
            props.youngs_modulus_gpa > 0.0,
            "Material {:?} has invalid Young's modulus",
            mat
        );
        // Breakdown field must be positive
        assert!(
            props.critical_breakdown_field_mv_cm > 0.0,
            "Material {:?} has invalid breakdown field",
            mat
        );
        // Carrier injection velocity must be positive
        assert!(
            props.injection_velocity_cm_s > 0.0,
            "Material {:?} has invalid injection velocity",
            mat
        );
    }
}

#[test]
fn test_ingaas_and_strained_ge_injection_velocities() {
    let si_props = HeteroMaterialType::SiliconGaa.properties();
    let ingaas_props = HeteroMaterialType::InGaAsNmos.properties();
    let ge_props = HeteroMaterialType::StrainedGePmos.properties();

    // InGaAs injection velocity must be at least 2.5x of Silicon for ultra-fast nMOS switching
    assert!(
        ingaas_props.injection_velocity_cm_s >= 2.5 * si_props.injection_velocity_cm_s,
        "InGaAs injection velocity must be >= 2.5x Silicon"
    );

    // Strained Ge hole injection velocity must be at least 1.8x of Silicon for balanced CMOS
    assert!(
        ge_props.injection_velocity_cm_s >= 1.8 * si_props.injection_velocity_cm_s,
        "Strained Ge injection velocity must be >= 1.8x Silicon"
    );
}

#[test]
fn test_igzo_beol_ultra_low_leakage() {
    let si_props = HeteroMaterialType::SiliconGaa.properties();
    let igzo_props = HeteroMaterialType::IgzoBeol.properties();

    // IGZO subthreshold off-state leakage current must be < 10^-15 A/um
    assert!(
        igzo_props.subthreshold_leakage_a_per_um < 1.0e-15,
        "IGZO leakage should be near zero"
    );

    // Leakage ratio compared to 3nm Silicon GAA should be > 10^7 lower
    let leakage_ratio =
        si_props.subthreshold_leakage_a_per_um / igzo_props.subthreshold_leakage_a_per_um;
    assert!(
        leakage_ratio >= 1.0e7,
        "IGZO should achieve > 10^7x lower leakage than Silicon GAA"
    );
}

#[test]
fn test_cnt_bundle_ballistic_interconnect() {
    let analyzer = TimingPathAnalyzer::new(0.8);
    let wire_length_um = 150.0;

    let t_cu =
        analyzer.evaluate_interconnect_delay_ps(HeteroMaterialType::SiliconGaa, wire_length_um);
    let t_cnt = analyzer
        .evaluate_interconnect_delay_ps(HeteroMaterialType::CntBundleInterconnect, wire_length_um);

    // Ballistic CNT bundle delay must be strictly lower than diffusive Copper
    assert!(
        t_cnt < t_cu * 0.6,
        "CNT bundle should provide > 40% interconnect delay reduction over Copper"
    );
}

#[test]
fn test_block_allocation_map_assignment_and_defaults() {
    let mut map = BlockAllocationMap::uniform_silicon();
    assert_eq!(
        map.get(ProcessorBlockType::ExecutionAlu),
        HeteroMaterialType::SiliconGaa
    );
    assert_eq!(
        map.get(ProcessorBlockType::L1Cache),
        HeteroMaterialType::SiliconGaa
    );

    map.assign(
        ProcessorBlockType::ExecutionAlu,
        HeteroMaterialType::InGaAsNmos,
    );
    map.assign(ProcessorBlockType::L1Cache, HeteroMaterialType::IgzoBeol);
    map.assign(
        ProcessorBlockType::PowerDeliveryFivr,
        HeteroMaterialType::WideBandgapGan,
    );
    map.assign(
        ProcessorBlockType::ClockDistribution,
        HeteroMaterialType::CntBundleInterconnect,
    );

    assert_eq!(
        map.get(ProcessorBlockType::ExecutionAlu),
        HeteroMaterialType::InGaAsNmos
    );
    assert_eq!(
        map.get(ProcessorBlockType::L1Cache),
        HeteroMaterialType::IgzoBeol
    );
    assert_eq!(
        map.get(ProcessorBlockType::PowerDeliveryFivr),
        HeteroMaterialType::WideBandgapGan
    );
    assert_eq!(
        map.get(ProcessorBlockType::ClockDistribution),
        HeteroMaterialType::CntBundleInterconnect
    );

    // Verify synthesized heterogeneous default allocation
    let synth = BlockAllocationMap::synthesized_heterogeneous();
    assert_eq!(
        synth.get(ProcessorBlockType::ExecutionAlu),
        HeteroMaterialType::InGaAsNmos
    );
    assert_eq!(
        synth.get(ProcessorBlockType::L1Cache),
        HeteroMaterialType::IgzoBeol
    );
    assert_eq!(
        synth.get(ProcessorBlockType::PowerDeliveryFivr),
        HeteroMaterialType::WideBandgapGan
    );
    assert_eq!(
        synth.get(ProcessorBlockType::ClockDistribution),
        HeteroMaterialType::CntBundleInterconnect
    );
}

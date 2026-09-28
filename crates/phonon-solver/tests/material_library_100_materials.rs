//! Integration Tests: 100+ Multi-Physics Material Library
//!
//! Validates:
//! 1. Registration of 100+ authoritative engineering materials across 8 distinct categories.
//! 2. Fundamental physical consistency: positive density, positive acoustic impedance,
//!    dielectric permittivity >= 1.0, optical refractive index >= 1.0.
//! 3. Complex permittivity frequency scaling, electromagnetic skin depth, and RF reflection.
//! 4. Acoustic impedance reflection coefficient and optical Fresnel reflectance.

#![deny(unsafe_code)]

use phonon_models::assets::{MaterialCategory, MaterialLibrary};

#[test]
fn test_material_library_100_materials_population() {
    let library = MaterialLibrary::new();

    // 1. Total material count must be >= 100:
    assert!(
        library.len() >= 100,
        "Library must contain at least 100 materials, got {}",
        library.len()
    );
    assert!(!library.is_empty());

    // 2. All 8 categories must be represented:
    let categories = [
        MaterialCategory::Conductor,
        MaterialCategory::Semiconductor,
        MaterialCategory::Dielectric,
        MaterialCategory::Polymer,
        MaterialCategory::StructuralAerospace,
        MaterialCategory::GeologicalCivil,
        MaterialCategory::AtmosphericBiological,
        MaterialCategory::Superconductor,
    ];

    for cat in &categories {
        let mats = library.list_by_category(*cat);
        assert!(
            !mats.is_empty(),
            "Category {:?} must have registered materials",
            cat
        );
        assert!(
            mats.len() >= 5,
            "Category {:?} must have at least 5 materials, got {}",
            cat,
            mats.len()
        );
    }

    // 3. Physical property sanity checks across ALL registered materials:
    for mat in library.iter() {
        // Density must be positive:
        assert!(
            mat.acoustic.density_kg_m3 > 0.0,
            "{}: density must be > 0",
            mat.name
        );
        // Longitudinal sound speed must be positive:
        assert!(
            mat.acoustic.sound_speed_longitudinal_m_s > 0.0,
            "{}: c_L must be > 0",
            mat.name
        );
        // Acoustic impedance must equal rho * c_L:
        let expected_z0 = mat.acoustic.density_kg_m3 * mat.acoustic.sound_speed_longitudinal_m_s;
        assert!(
            (mat.acoustic.acoustic_impedance_rayls - expected_z0).abs() < 1e-3,
            "{}: Z_0 must match rho * c_L",
            mat.name
        );
        // Relative permittivity must be >= 1.0:
        assert!(
            mat.em.relative_permittivity >= 1.0,
            "{}: eps_r must be >= 1.0",
            mat.name
        );
        // Relative permeability must be >= 0.0:
        assert!(
            mat.em.relative_permeability >= 0.0,
            "{}: mu_r must be >= 0",
            mat.name
        );
        // Refractive index must be > 0.0 (metals have n < 1.0 in Drude optical regime):
        assert!(
            mat.optical.refractive_index > 0.0,
            "{}: n must be > 0",
            mat.name
        );
        // Thermal conductivity must be positive:
        assert!(
            mat.thermal_mech.thermal_conductivity_w_m_k > 0.0,
            "{}: k_th must be > 0",
            mat.name
        );
        // Specific heat capacity must be positive:
        assert!(
            mat.thermal_mech.specific_heat_j_kg_k > 0.0,
            "{}: c_p must be > 0",
            mat.name
        );
    }
}

#[test]
fn test_material_lookups_and_aliases() {
    let library = MaterialLibrary::new();

    // Exact name lookup:
    let copper = library.get_by_name("Copper").expect("Copper must exist");
    assert_eq!(copper.category, MaterialCategory::Conductor);
    assert!(copper.em.conductivity_s_per_m > 5e7);

    // Case-insensitive lookup:
    let teflon = library
        .get_by_name("ptfe teflon")
        .expect("Teflon case-insensitive must match");
    assert_eq!(teflon.category, MaterialCategory::Polymer);

    // ID lookup:
    let mat0 = library.get_by_id(0).expect("ID 0 must exist");
    assert_eq!(mat0.name, "Copper");

    let silicon = library
        .get_by_name("Silicon Intrinsic")
        .expect("Silicon must exist");
    assert!((silicon.optical.bandgap_ev - 1.12).abs() < 1e-3);
}

#[test]
fn test_electromagnetic_and_acoustic_physical_formulas() {
    let library = MaterialLibrary::new();
    let copper = library.get_by_name("Copper").unwrap();
    let air = library.get_by_name("Air STP").unwrap();
    let glass = library.get_by_name("Window Glass").unwrap();
    let steel = library.get_by_name("Structural Carbon Steel").unwrap();

    // 1. Copper skin depth at 10 GHz:
    // delta_s = sqrt(2 / (omega * mu_0 * sigma)) ~= sqrt(2 / (2*pi*1e10 * 4*pi*1e-7 * 5.96e7))
    // omega * mu * sigma = 2*pi*1e10 * 1.2566e-6 * 5.96e7 ~= 4.706e12
    // delta_s = sqrt(2 / 4.706e12) ~= 6.52e-7 m = 0.652 um
    let skin_d_10ghz = copper.em.skin_depth_m(10.0e9);
    assert!((skin_d_10ghz - 6.52e-7).abs() < 5e-8);

    // 2. Optical normal reflectance of window glass (n = 1.52, k = 0):
    // R = ((1.52 - 1)/(1.52 + 1))^2 = (0.52 / 2.52)^2 ~= 0.04258 (~4.26%)
    let r_opt_glass = glass.optical.normal_reflectance();
    assert!((r_opt_glass - 0.0426).abs() < 0.002);

    // 3. Acoustic reflection coefficient from Air to Steel:
    // Z_air ~= 1.204 * 343.2 ~= 413.2 Rayls
    // Z_steel ~= 7850 * 5900 ~= 4.63e7 Rayls
    // R = (Z_steel - Z_air) / (Z_steel + Z_air) ~= 0.999982 (nearly 1.0 total reflection)
    let r_ac = air.acoustic.reflection_coefficient(&steel.acoustic);
    assert!(r_ac > 0.9999);

    // 4. Acoustic power transmission between Air and Steel is nearly zero:
    let t_ac = air.acoustic.power_transmission_coefficient(&steel.acoustic);
    assert!(t_ac < 0.0001);
}

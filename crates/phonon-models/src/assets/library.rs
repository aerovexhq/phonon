//! Authoritative 100+ Multi-Physics Material Library
//!
//! Pre-populates comprehensive cross-domain physical material properties
//! from CRC Handbook of Chemistry & Physics, NIST, ITU-R recommendations,
//! and IEEE standards.

use super::material::{
    AcousticProperties, ElectromagneticProperties, MaterialCategory, MaterialRecord,
    OpticalProperties, ThermalMechanicalProperties,
};
use std::collections::HashMap;

/// Pre-populated database of authoritative multi-physics engineering materials.
#[derive(Debug, Clone)]
pub struct MaterialLibrary {
    materials: Vec<MaterialRecord>,
    name_index: HashMap<&'static str, usize>,
}

impl Default for MaterialLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialLibrary {
    /// Creates and pre-populates the library with 100+ standard materials.
    pub fn new() -> Self {
        let mut materials = Vec::with_capacity(120);

        // Material 0: Copper
        materials.push(MaterialRecord {
            id: 0,
            name: "Copper",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 59600000.0),
            optical: OpticalProperties::new(0.25, 3.4, 0.0, 0.65, 0.85, 0.2),
            acoustic: AcousticProperties::new(8960.0, 4660.0, 2260.0, 0.005),
            thermal_mech: ThermalMechanicalProperties::new(401.0, 385.0, 117.0, 0.34, 70.0),
        });

        // Material 1: Annealed Copper
        materials.push(MaterialRecord {
            id: 1,
            name: "Annealed Copper",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 58000000.0),
            optical: OpticalProperties::new(0.24, 3.3, 0.0, 0.63, 0.84, 0.2),
            acoustic: AcousticProperties::new(8940.0, 4660.0, 2260.0, 0.005),
            thermal_mech: ThermalMechanicalProperties::new(398.0, 385.0, 115.0, 0.34, 60.0),
        });

        // Material 2: Aluminum Pure
        materials.push(MaterialRecord {
            id: 2,
            name: "Aluminum Pure",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 37700000.0),
            optical: OpticalProperties::new(1.4, 7.5, 0.0, 0.85, 0.92, 0.15),
            acoustic: AcousticProperties::new(2700.0, 6320.0, 3130.0, 0.003),
            thermal_mech: ThermalMechanicalProperties::new(237.0, 900.0, 70.0, 0.33, 30.0),
        });

        // Material 3: Aluminum 6061-T6
        materials.push(MaterialRecord {
            id: 3,
            name: "Aluminum 6061-T6",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 25000000.0),
            optical: OpticalProperties::new(1.4, 7.5, 0.0, 0.8, 0.88, 0.25),
            acoustic: AcousticProperties::new(2700.0, 6300.0, 3120.0, 0.004),
            thermal_mech: ThermalMechanicalProperties::new(167.0, 896.0, 68.9, 0.33, 276.0),
        });

        // Material 4: Aluminum 7075-T6
        materials.push(MaterialRecord {
            id: 4,
            name: "Aluminum 7075-T6",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 19000000.0),
            optical: OpticalProperties::new(1.4, 7.5, 0.0, 0.78, 0.86, 0.25),
            acoustic: AcousticProperties::new(2810.0, 6200.0, 3100.0, 0.004),
            thermal_mech: ThermalMechanicalProperties::new(130.0, 960.0, 71.7, 0.33, 503.0),
        });

        // Material 5: Gold
        materials.push(MaterialRecord {
            id: 5,
            name: "Gold",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 41000000.0),
            optical: OpticalProperties::new(0.2, 2.9, 0.0, 0.75, 0.95, 0.1),
            acoustic: AcousticProperties::new(19300.0, 3240.0, 1200.0, 0.006),
            thermal_mech: ThermalMechanicalProperties::new(318.0, 129.0, 78.0, 0.44, 20.0),
        });

        // Material 6: Silver
        materials.push(MaterialRecord {
            id: 6,
            name: "Silver",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 63000000.0),
            optical: OpticalProperties::new(0.15, 3.5, 0.0, 0.9, 0.98, 0.08),
            acoustic: AcousticProperties::new(10490.0, 3650.0, 1610.0, 0.004),
            thermal_mech: ThermalMechanicalProperties::new(429.0, 235.0, 83.0, 0.37, 50.0),
        });

        // Material 7: Platinum
        materials.push(MaterialRecord {
            id: 7,
            name: "Platinum",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 9430000.0),
            optical: OpticalProperties::new(2.3, 4.3, 0.0, 0.65, 0.75, 0.15),
            acoustic: AcousticProperties::new(21450.0, 3260.0, 1730.0, 0.005),
            thermal_mech: ThermalMechanicalProperties::new(71.6, 133.0, 168.0, 0.38, 120.0),
        });

        // Material 8: Titanium Grade 5
        materials.push(MaterialRecord {
            id: 8,
            name: "Titanium Grade 5",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 580000.0),
            optical: OpticalProperties::new(2.8, 3.4, 0.0, 0.55, 0.65, 0.3),
            acoustic: AcousticProperties::new(4430.0, 6100.0, 3120.0, 0.008),
            thermal_mech: ThermalMechanicalProperties::new(6.7, 526.0, 114.0, 0.34, 880.0),
        });

        // Material 9: Tungsten
        materials.push(MaterialRecord {
            id: 9,
            name: "Tungsten",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 17900000.0),
            optical: OpticalProperties::new(3.6, 3.0, 0.0, 0.5, 0.6, 0.2),
            acoustic: AcousticProperties::new(19250.0, 5220.0, 2890.0, 0.003),
            thermal_mech: ThermalMechanicalProperties::new(173.0, 132.0, 411.0, 0.28, 750.0),
        });

        // Material 10: Structural Carbon Steel
        materials.push(MaterialRecord {
            id: 10,
            name: "Structural Carbon Steel",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 100.0, 5900000.0),
            optical: OpticalProperties::new(2.5, 3.5, 0.0, 0.5, 0.6, 0.35),
            acoustic: AcousticProperties::new(7850.0, 5900.0, 3200.0, 0.006),
            thermal_mech: ThermalMechanicalProperties::new(50.0, 486.0, 200.0, 0.3, 250.0),
        });

        // Material 11: Stainless Steel 304
        materials.push(MaterialRecord {
            id: 11,
            name: "Stainless Steel 304",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 1450000.0),
            optical: OpticalProperties::new(2.6, 3.7, 0.0, 0.55, 0.65, 0.25),
            acoustic: AcousticProperties::new(8000.0, 5740.0, 3130.0, 0.008),
            thermal_mech: ThermalMechanicalProperties::new(16.2, 500.0, 193.0, 0.29, 215.0),
        });

        // Material 12: Stainless Steel 316
        materials.push(MaterialRecord {
            id: 12,
            name: "Stainless Steel 316",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 1350000.0),
            optical: OpticalProperties::new(2.6, 3.8, 0.0, 0.55, 0.65, 0.25),
            acoustic: AcousticProperties::new(8000.0, 5700.0, 3100.0, 0.008),
            thermal_mech: ThermalMechanicalProperties::new(15.0, 500.0, 193.0, 0.3, 290.0),
        });

        // Material 13: Nickel
        materials.push(MaterialRecord {
            id: 13,
            name: "Nickel",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 100.0, 14300000.0),
            optical: OpticalProperties::new(2.0, 3.7, 0.0, 0.6, 0.7, 0.2),
            acoustic: AcousticProperties::new(8908.0, 5600.0, 3000.0, 0.005),
            thermal_mech: ThermalMechanicalProperties::new(90.9, 444.0, 200.0, 0.31, 140.0),
        });

        // Material 14: Nichrome
        materials.push(MaterialRecord {
            id: 14,
            name: "Nichrome",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 909000.0),
            optical: OpticalProperties::new(2.5, 3.0, 0.0, 0.45, 0.55, 0.35),
            acoustic: AcousticProperties::new(8400.0, 5300.0, 2900.0, 0.01),
            thermal_mech: ThermalMechanicalProperties::new(11.3, 450.0, 220.0, 0.32, 350.0),
        });

        // Material 15: Cartridge Brass
        materials.push(MaterialRecord {
            id: 15,
            name: "Cartridge Brass",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 15000000.0),
            optical: OpticalProperties::new(0.4, 3.0, 0.0, 0.7, 0.85, 0.2),
            acoustic: AcousticProperties::new(8530.0, 4700.0, 2110.0, 0.006),
            thermal_mech: ThermalMechanicalProperties::new(115.0, 380.0, 110.0, 0.35, 130.0),
        });

        // Material 16: Phosphor Bronze
        materials.push(MaterialRecord {
            id: 16,
            name: "Phosphor Bronze",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 9260000.0),
            optical: OpticalProperties::new(0.5, 3.1, 0.0, 0.65, 0.8, 0.25),
            acoustic: AcousticProperties::new(8860.0, 4500.0, 2100.0, 0.007),
            thermal_mech: ThermalMechanicalProperties::new(75.0, 380.0, 110.0, 0.36, 180.0),
        });

        // Material 17: Lead
        materials.push(MaterialRecord {
            id: 17,
            name: "Lead",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 4810000.0),
            optical: OpticalProperties::new(2.0, 3.5, 0.0, 0.4, 0.5, 0.4),
            acoustic: AcousticProperties::new(11340.0, 2160.0, 700.0, 0.03),
            thermal_mech: ThermalMechanicalProperties::new(35.3, 129.0, 16.0, 0.44, 12.0),
        });

        // Material 18: Molybdenum
        materials.push(MaterialRecord {
            id: 18,
            name: "Molybdenum",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 19200000.0),
            optical: OpticalProperties::new(3.4, 3.6, 0.0, 0.55, 0.65, 0.2),
            acoustic: AcousticProperties::new(10280.0, 6250.0, 3350.0, 0.003),
            thermal_mech: ThermalMechanicalProperties::new(138.0, 250.0, 329.0, 0.31, 500.0),
        });

        // Material 19: Tantalum
        materials.push(MaterialRecord {
            id: 19,
            name: "Tantalum",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 7630000.0),
            optical: OpticalProperties::new(3.2, 2.5, 0.0, 0.5, 0.6, 0.25),
            acoustic: AcousticProperties::new(16690.0, 4100.0, 2000.0, 0.006),
            thermal_mech: ThermalMechanicalProperties::new(57.5, 140.0, 186.0, 0.34, 200.0),
        });

        // Material 20: Zinc
        materials.push(MaterialRecord {
            id: 20,
            name: "Zinc",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 16700000.0),
            optical: OpticalProperties::new(1.9, 4.0, 0.0, 0.6, 0.75, 0.3),
            acoustic: AcousticProperties::new(7140.0, 4200.0, 2400.0, 0.008),
            thermal_mech: ThermalMechanicalProperties::new(116.0, 388.0, 108.0, 0.25, 110.0),
        });

        // Material 21: Tin
        materials.push(MaterialRecord {
            id: 21,
            name: "Tin",
            category: MaterialCategory::Conductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 9170000.0),
            optical: OpticalProperties::new(2.1, 4.5, 0.0, 0.65, 0.8, 0.3),
            acoustic: AcousticProperties::new(7310.0, 3320.0, 1670.0, 0.012),
            thermal_mech: ThermalMechanicalProperties::new(66.8, 227.0, 50.0, 0.36, 20.0),
        });

        // Material 22: Silicon Intrinsic
        materials.push(MaterialRecord {
            id: 22,
            name: "Silicon Intrinsic",
            category: MaterialCategory::Semiconductor,
            em: ElectromagneticProperties::new(11.7, 0.0001, 1.0, 0.00156),
            optical: OpticalProperties::new(3.42, 0.001, 1.12, 0.3, 0.35, 0.05),
            acoustic: AcousticProperties::new(2329.0, 8430.0, 5840.0, 0.001),
            thermal_mech: ThermalMechanicalProperties::new(149.0, 705.0, 130.0, 0.28, 120.0),
        });

        // Material 23: Silicon N-Doped
        materials.push(MaterialRecord {
            id: 23,
            name: "Silicon N-Doped",
            category: MaterialCategory::Semiconductor,
            em: ElectromagneticProperties::new(11.7, 0.005, 1.0, 1000.0),
            optical: OpticalProperties::new(3.45, 0.05, 1.12, 0.32, 0.38, 0.05),
            acoustic: AcousticProperties::new(2330.0, 8400.0, 5820.0, 0.002),
            thermal_mech: ThermalMechanicalProperties::new(140.0, 710.0, 130.0, 0.28, 120.0),
        });

        // Material 24: Silicon P-Doped
        materials.push(MaterialRecord {
            id: 24,
            name: "Silicon P-Doped",
            category: MaterialCategory::Semiconductor,
            em: ElectromagneticProperties::new(11.7, 0.008, 1.0, 500.0),
            optical: OpticalProperties::new(3.45, 0.04, 1.12, 0.32, 0.38, 0.05),
            acoustic: AcousticProperties::new(2330.0, 8400.0, 5820.0, 0.002),
            thermal_mech: ThermalMechanicalProperties::new(140.0, 710.0, 130.0, 0.28, 120.0),
        });

        // Material 25: Germanium
        materials.push(MaterialRecord {
            id: 25,
            name: "Germanium",
            category: MaterialCategory::Semiconductor,
            em: ElectromagneticProperties::new(16.0, 0.001, 1.0, 2.17),
            optical: OpticalProperties::new(4.01, 0.01, 0.66, 0.35, 0.45, 0.05),
            acoustic: AcousticProperties::new(5323.0, 5400.0, 3550.0, 0.002),
            thermal_mech: ThermalMechanicalProperties::new(60.0, 320.0, 103.0, 0.26, 90.0),
        });

        // Material 26: Gallium Arsenide
        materials.push(MaterialRecord {
            id: 26,
            name: "Gallium Arsenide",
            category: MaterialCategory::Semiconductor,
            em: ElectromagneticProperties::new(12.9, 0.0006, 1.0, 1e-06),
            optical: OpticalProperties::new(3.65, 0.001, 1.42, 0.32, 0.4, 0.05),
            acoustic: AcousticProperties::new(5317.0, 4730.0, 3350.0, 0.002),
            thermal_mech: ThermalMechanicalProperties::new(55.0, 350.0, 85.5, 0.31, 80.0),
        });

        // Material 27: Gallium Nitride
        materials.push(MaterialRecord {
            id: 27,
            name: "Gallium Nitride",
            category: MaterialCategory::Semiconductor,
            em: ElectromagneticProperties::new(8.9, 0.0005, 1.0, 0.0001),
            optical: OpticalProperties::new(2.4, 0.0001, 3.4, 0.18, 0.22, 0.05),
            acoustic: AcousticProperties::new(6150.0, 8000.0, 4200.0, 0.001),
            thermal_mech: ThermalMechanicalProperties::new(130.0, 490.0, 295.0, 0.25, 200.0),
        });

        // Material 28: Indium Phosphide
        materials.push(MaterialRecord {
            id: 28,
            name: "Indium Phosphide",
            category: MaterialCategory::Semiconductor,
            em: ElectromagneticProperties::new(12.5, 0.001, 1.0, 1e-05),
            optical: OpticalProperties::new(3.45, 0.001, 1.34, 0.3, 0.38, 0.05),
            acoustic: AcousticProperties::new(4810.0, 5130.0, 3080.0, 0.002),
            thermal_mech: ThermalMechanicalProperties::new(68.0, 310.0, 61.0, 0.36, 70.0),
        });

        // Material 29: Silicon Carbide 4H
        materials.push(MaterialRecord {
            id: 29,
            name: "Silicon Carbide 4H",
            category: MaterialCategory::Semiconductor,
            em: ElectromagneticProperties::new(9.7, 0.0002, 1.0, 1e-05),
            optical: OpticalProperties::new(2.65, 0.0001, 3.23, 0.2, 0.25, 0.05),
            acoustic: AcousticProperties::new(3210.0, 13000.0, 7800.0, 0.0005),
            thermal_mech: ThermalMechanicalProperties::new(490.0, 690.0, 450.0, 0.14, 400.0),
        });

        // Material 30: Diamond CVD
        materials.push(MaterialRecord {
            id: 30,
            name: "Diamond CVD",
            category: MaterialCategory::Semiconductor,
            em: ElectromagneticProperties::new(5.7, 0.0001, 1.0, 1e-12),
            optical: OpticalProperties::new(2.42, 0.0, 5.47, 0.17, 0.22, 0.02),
            acoustic: AcousticProperties::new(3515.0, 18000.0, 11000.0, 0.0002),
            thermal_mech: ThermalMechanicalProperties::new(2200.0, 512.0, 1050.0, 0.07, 1000.0),
        });

        // Material 31: InGaAs
        materials.push(MaterialRecord {
            id: 31,
            name: "InGaAs",
            category: MaterialCategory::Semiconductor,
            em: ElectromagneticProperties::new(13.9, 0.001, 1.0, 0.001),
            optical: OpticalProperties::new(3.52, 0.01, 0.74, 0.31, 0.39, 0.05),
            acoustic: AcousticProperties::new(5500.0, 4500.0, 2900.0, 0.003),
            thermal_mech: ThermalMechanicalProperties::new(5.0, 330.0, 75.0, 0.33, 65.0),
        });

        // Material 32: Cadmium Telluride
        materials.push(MaterialRecord {
            id: 32,
            name: "Cadmium Telluride",
            category: MaterialCategory::Semiconductor,
            em: ElectromagneticProperties::new(10.2, 0.001, 1.0, 1e-06),
            optical: OpticalProperties::new(2.7, 0.005, 1.49, 0.22, 0.28, 0.05),
            acoustic: AcousticProperties::new(5850.0, 3300.0, 1800.0, 0.005),
            thermal_mech: ThermalMechanicalProperties::new(6.2, 210.0, 52.0, 0.35, 45.0),
        });

        // Material 33: MoS2 Monolayer
        materials.push(MaterialRecord {
            id: 33,
            name: "MoS2 Monolayer",
            category: MaterialCategory::Semiconductor,
            em: ElectromagneticProperties::new(4.0, 0.002, 1.0, 0.0001),
            optical: OpticalProperties::new(4.2, 0.1, 1.8, 0.38, 0.48, 0.05),
            acoustic: AcousticProperties::new(5060.0, 7200.0, 4100.0, 0.002),
            thermal_mech: ThermalMechanicalProperties::new(34.5, 397.0, 270.0, 0.29, 300.0),
        });

        // Material 34: Graphene Monolayer
        materials.push(MaterialRecord {
            id: 34,
            name: "Graphene Monolayer",
            category: MaterialCategory::Semiconductor,
            em: ElectromagneticProperties::new(2.5, 0.01, 1.0, 100000.0),
            optical: OpticalProperties::new(2.6, 1.3, 0.0, 0.2, 0.3, 0.02),
            acoustic: AcousticProperties::new(2267.0, 21000.0, 13000.0, 0.0001),
            thermal_mech: ThermalMechanicalProperties::new(4000.0, 700.0, 1000.0, 0.16, 1000.0),
        });

        // Material 35: Zinc Oxide
        materials.push(MaterialRecord {
            id: 35,
            name: "Zinc Oxide",
            category: MaterialCategory::Semiconductor,
            em: ElectromagneticProperties::new(8.5, 0.002, 1.0, 1e-05),
            optical: OpticalProperties::new(2.0, 0.001, 3.37, 0.12, 0.15, 0.05),
            acoustic: AcousticProperties::new(5610.0, 6100.0, 2800.0, 0.003),
            thermal_mech: ThermalMechanicalProperties::new(54.0, 523.0, 140.0, 0.36, 120.0),
        });

        // Material 36: FR-4 PCB
        materials.push(MaterialRecord {
            id: 36,
            name: "FR-4 PCB",
            category: MaterialCategory::Dielectric,
            em: ElectromagneticProperties::new(4.4, 0.02, 1.0, 1e-12),
            optical: OpticalProperties::new(1.55, 0.0, 3.5, 0.25, 0.15, 0.3),
            acoustic: AcousticProperties::new(1850.0, 3200.0, 1800.0, 0.05),
            thermal_mech: ThermalMechanicalProperties::new(0.3, 1200.0, 24.0, 0.18, 70.0),
        });

        // Material 37: Rogers RT/duroid 5880
        materials.push(MaterialRecord {
            id: 37,
            name: "Rogers RT/duroid 5880",
            category: MaterialCategory::Dielectric,
            em: ElectromagneticProperties::new(2.2, 0.0009, 1.0, 1e-13),
            optical: OpticalProperties::new(1.48, 0.0, 4.0, 0.2, 0.1, 0.2),
            acoustic: AcousticProperties::new(1370.0, 1500.0, 800.0, 0.04),
            thermal_mech: ThermalMechanicalProperties::new(0.2, 1000.0, 0.8, 0.35, 15.0),
        });

        // Material 38: Rogers RO4350B
        materials.push(MaterialRecord {
            id: 38,
            name: "Rogers RO4350B",
            category: MaterialCategory::Dielectric,
            em: ElectromagneticProperties::new(3.48, 0.0037, 1.0, 1e-12),
            optical: OpticalProperties::new(1.52, 0.0, 3.8, 0.22, 0.12, 0.25),
            acoustic: AcousticProperties::new(1860.0, 3000.0, 1600.0, 0.04),
            thermal_mech: ThermalMechanicalProperties::new(0.69, 1100.0, 18.0, 0.2, 50.0),
        });

        // Material 39: Alumina 99.6%
        materials.push(MaterialRecord {
            id: 39,
            name: "Alumina 99.6%",
            category: MaterialCategory::Dielectric,
            em: ElectromagneticProperties::new(9.8, 0.0002, 1.0, 1e-13),
            optical: OpticalProperties::new(1.76, 0.0, 7.0, 0.15, 0.2, 0.05),
            acoustic: AcousticProperties::new(3900.0, 10500.0, 6200.0, 0.001),
            thermal_mech: ThermalMechanicalProperties::new(35.0, 880.0, 380.0, 0.22, 300.0),
        });

        // Material 40: Aluminum Nitride
        materials.push(MaterialRecord {
            id: 40,
            name: "Aluminum Nitride",
            category: MaterialCategory::Dielectric,
            em: ElectromagneticProperties::new(8.8, 0.0005, 1.0, 1e-12),
            optical: OpticalProperties::new(2.15, 0.0, 6.2, 0.18, 0.25, 0.05),
            acoustic: AcousticProperties::new(3260.0, 11000.0, 6000.0, 0.001),
            thermal_mech: ThermalMechanicalProperties::new(180.0, 740.0, 330.0, 0.24, 280.0),
        });

        // Material 41: Silicon Dioxide
        materials.push(MaterialRecord {
            id: 41,
            name: "Silicon Dioxide",
            category: MaterialCategory::Dielectric,
            em: ElectromagneticProperties::new(3.9, 0.0001, 1.0, 1e-14),
            optical: OpticalProperties::new(1.46, 0.0, 8.9, 0.1, 0.15, 0.02),
            acoustic: AcousticProperties::new(2200.0, 5900.0, 3750.0, 0.002),
            thermal_mech: ThermalMechanicalProperties::new(1.4, 750.0, 73.0, 0.17, 110.0),
        });

        // Material 42: Silicon Nitride
        materials.push(MaterialRecord {
            id: 42,
            name: "Silicon Nitride",
            category: MaterialCategory::Dielectric,
            em: ElectromagneticProperties::new(7.5, 0.0005, 1.0, 1e-12),
            optical: OpticalProperties::new(2.05, 0.0, 5.0, 0.15, 0.2, 0.05),
            acoustic: AcousticProperties::new(3100.0, 10300.0, 5800.0, 0.001),
            thermal_mech: ThermalMechanicalProperties::new(30.0, 710.0, 310.0, 0.27, 450.0),
        });

        // Material 43: Hafnium Oxide
        materials.push(MaterialRecord {
            id: 43,
            name: "Hafnium Oxide",
            category: MaterialCategory::Dielectric,
            em: ElectromagneticProperties::new(25.0, 0.001, 1.0, 1e-11),
            optical: OpticalProperties::new(2.1, 0.0, 5.7, 0.2, 0.25, 0.05),
            acoustic: AcousticProperties::new(9680.0, 5500.0, 3100.0, 0.003),
            thermal_mech: ThermalMechanicalProperties::new(2.3, 120.0, 240.0, 0.25, 200.0),
        });

        // Material 44: Sapphire
        materials.push(MaterialRecord {
            id: 44,
            name: "Sapphire",
            category: MaterialCategory::Dielectric,
            em: ElectromagneticProperties::new(11.5, 0.0001, 1.0, 1e-14),
            optical: OpticalProperties::new(1.77, 0.0, 9.9, 0.1, 0.18, 0.01),
            acoustic: AcousticProperties::new(3980.0, 11100.0, 6000.0, 0.001),
            thermal_mech: ThermalMechanicalProperties::new(42.0, 760.0, 400.0, 0.29, 400.0),
        });

        // Material 45: Fused Quartz
        materials.push(MaterialRecord {
            id: 45,
            name: "Fused Quartz",
            category: MaterialCategory::Dielectric,
            em: ElectromagneticProperties::new(3.78, 0.0001, 1.0, 1e-15),
            optical: OpticalProperties::new(1.46, 0.0, 9.0, 0.08, 0.12, 0.02),
            acoustic: AcousticProperties::new(2200.0, 5970.0, 3760.0, 0.001),
            thermal_mech: ThermalMechanicalProperties::new(1.4, 740.0, 72.0, 0.17, 110.0),
        });

        // Material 46: Fused Silica
        materials.push(MaterialRecord {
            id: 46,
            name: "Fused Silica",
            category: MaterialCategory::Dielectric,
            em: ElectromagneticProperties::new(3.82, 8e-05, 1.0, 1e-15),
            optical: OpticalProperties::new(1.46, 0.0, 9.0, 0.08, 0.12, 0.02),
            acoustic: AcousticProperties::new(2201.0, 5960.0, 3760.0, 0.001),
            thermal_mech: ThermalMechanicalProperties::new(1.38, 745.0, 73.0, 0.17, 110.0),
        });

        // Material 47: Low-k Organosilicate
        materials.push(MaterialRecord {
            id: 47,
            name: "Low-k Organosilicate",
            category: MaterialCategory::Dielectric,
            em: ElectromagneticProperties::new(2.5, 0.002, 1.0, 1e-13),
            optical: OpticalProperties::new(1.35, 0.0, 4.5, 0.12, 0.15, 0.05),
            acoustic: AcousticProperties::new(1200.0, 3400.0, 1800.0, 0.01),
            thermal_mech: ThermalMechanicalProperties::new(0.35, 1000.0, 10.0, 0.22, 25.0),
        });

        // Material 48: Barium Titanate
        materials.push(MaterialRecord {
            id: 48,
            name: "Barium Titanate",
            category: MaterialCategory::Dielectric,
            em: ElectromagneticProperties::new(1200.0, 0.02, 1.0, 1e-10),
            optical: OpticalProperties::new(2.4, 0.0, 3.2, 0.25, 0.35, 0.1),
            acoustic: AcousticProperties::new(6020.0, 4800.0, 2600.0, 0.005),
            thermal_mech: ThermalMechanicalProperties::new(6.0, 430.0, 120.0, 0.35, 80.0),
        });

        // Material 49: Gallium Oxide
        materials.push(MaterialRecord {
            id: 49,
            name: "Gallium Oxide",
            category: MaterialCategory::Dielectric,
            em: ElectromagneticProperties::new(10.0, 0.0004, 1.0, 1e-12),
            optical: OpticalProperties::new(1.9, 0.0, 4.8, 0.14, 0.2, 0.05),
            acoustic: AcousticProperties::new(5880.0, 6800.0, 3800.0, 0.002),
            thermal_mech: ThermalMechanicalProperties::new(27.0, 560.0, 260.0, 0.26, 200.0),
        });

        // Material 50: PTFE Teflon
        materials.push(MaterialRecord {
            id: 50,
            name: "PTFE Teflon",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(2.1, 0.0002, 1.0, 1e-15),
            optical: OpticalProperties::new(1.35, 0.0, 6.0, 0.7, 0.1, 0.25),
            acoustic: AcousticProperties::new(2200.0, 1400.0, 500.0, 0.08),
            thermal_mech: ThermalMechanicalProperties::new(0.25, 1050.0, 0.5, 0.46, 25.0),
        });

        // Material 51: Polyimide Kapton
        materials.push(MaterialRecord {
            id: 51,
            name: "Polyimide Kapton",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(3.4, 0.002, 1.0, 1e-14),
            optical: OpticalProperties::new(1.7, 0.0, 3.2, 0.4, 0.2, 0.2),
            acoustic: AcousticProperties::new(1420.0, 2200.0, 950.0, 0.05),
            thermal_mech: ThermalMechanicalProperties::new(0.12, 1090.0, 2.5, 0.34, 69.0),
        });

        // Material 52: PEEK
        materials.push(MaterialRecord {
            id: 52,
            name: "PEEK",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(3.2, 0.003, 1.0, 1e-14),
            optical: OpticalProperties::new(1.65, 0.0, 3.5, 0.35, 0.25, 0.25),
            acoustic: AcousticProperties::new(1320.0, 2500.0, 1050.0, 0.05),
            thermal_mech: ThermalMechanicalProperties::new(0.25, 1340.0, 3.8, 0.38, 100.0),
        });

        // Material 53: Polycarbonate
        materials.push(MaterialRecord {
            id: 53,
            name: "Polycarbonate",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(2.9, 0.001, 1.0, 1e-14),
            optical: OpticalProperties::new(1.58, 0.0, 3.8, 0.15, 0.2, 0.15),
            acoustic: AcousticProperties::new(1200.0, 2270.0, 960.0, 0.06),
            thermal_mech: ThermalMechanicalProperties::new(0.2, 1250.0, 2.4, 0.37, 65.0),
        });

        // Material 54: PMMA Acrylic
        materials.push(MaterialRecord {
            id: 54,
            name: "PMMA Acrylic",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(2.6, 0.005, 1.0, 1e-14),
            optical: OpticalProperties::new(1.49, 0.0, 4.0, 0.1, 0.15, 0.1),
            acoustic: AcousticProperties::new(1190.0, 2750.0, 1380.0, 0.04),
            thermal_mech: ThermalMechanicalProperties::new(0.19, 1460.0, 3.2, 0.35, 75.0),
        });

        // Material 55: Polyethylene HDPE
        materials.push(MaterialRecord {
            id: 55,
            name: "Polyethylene HDPE",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(2.3, 0.0004, 1.0, 1e-15),
            optical: OpticalProperties::new(1.53, 0.0, 6.0, 0.6, 0.15, 0.3),
            acoustic: AcousticProperties::new(950.0, 2430.0, 950.0, 0.06),
            thermal_mech: ThermalMechanicalProperties::new(0.45, 1900.0, 1.0, 0.42, 26.0),
        });

        // Material 56: Polyethylene LDPE
        materials.push(MaterialRecord {
            id: 56,
            name: "Polyethylene LDPE",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(2.25, 0.0003, 1.0, 1e-15),
            optical: OpticalProperties::new(1.51, 0.0, 6.0, 0.55, 0.15, 0.35),
            acoustic: AcousticProperties::new(920.0, 2000.0, 800.0, 0.07),
            thermal_mech: ThermalMechanicalProperties::new(0.33, 2300.0, 0.3, 0.45, 10.0),
        });

        // Material 57: Polypropylene
        materials.push(MaterialRecord {
            id: 57,
            name: "Polypropylene",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(2.25, 0.0005, 1.0, 1e-15),
            optical: OpticalProperties::new(1.49, 0.0, 5.0, 0.5, 0.15, 0.3),
            acoustic: AcousticProperties::new(905.0, 2650.0, 1300.0, 0.05),
            thermal_mech: ThermalMechanicalProperties::new(0.22, 1900.0, 1.5, 0.4, 30.0),
        });

        // Material 58: Nylon 6-6
        materials.push(MaterialRecord {
            id: 58,
            name: "Nylon 6-6",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(3.5, 0.02, 1.0, 1e-13),
            optical: OpticalProperties::new(1.53, 0.0, 4.2, 0.4, 0.2, 0.3),
            acoustic: AcousticProperties::new(1140.0, 2600.0, 1100.0, 0.06),
            thermal_mech: ThermalMechanicalProperties::new(0.25, 1700.0, 2.8, 0.39, 80.0),
        });

        // Material 59: Epoxy Resin
        materials.push(MaterialRecord {
            id: 59,
            name: "Epoxy Resin",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(3.6, 0.015, 1.0, 1e-13),
            optical: OpticalProperties::new(1.58, 0.0, 3.6, 0.3, 0.2, 0.25),
            acoustic: AcousticProperties::new(1250.0, 2500.0, 1100.0, 0.05),
            thermal_mech: ThermalMechanicalProperties::new(0.2, 1100.0, 3.5, 0.35, 60.0),
        });

        // Material 60: Silicone Elastomer
        materials.push(MaterialRecord {
            id: 60,
            name: "Silicone Elastomer",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(3.2, 0.005, 1.0, 1e-14),
            optical: OpticalProperties::new(1.43, 0.0, 5.0, 0.5, 0.1, 0.35),
            acoustic: AcousticProperties::new(1100.0, 1000.0, 200.0, 0.15),
            thermal_mech: ThermalMechanicalProperties::new(0.2, 1200.0, 0.01, 0.48, 5.0),
        });

        // Material 61: Natural Rubber
        materials.push(MaterialRecord {
            id: 61,
            name: "Natural Rubber",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(2.7, 0.003, 1.0, 1e-14),
            optical: OpticalProperties::new(1.52, 0.0, 4.5, 0.3, 0.1, 0.4),
            acoustic: AcousticProperties::new(920.0, 1550.0, 300.0, 0.12),
            thermal_mech: ThermalMechanicalProperties::new(0.13, 1800.0, 0.005, 0.49, 15.0),
        });

        // Material 62: Polyurethane
        materials.push(MaterialRecord {
            id: 62,
            name: "Polyurethane",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(3.5, 0.025, 1.0, 1e-12),
            optical: OpticalProperties::new(1.55, 0.0, 3.8, 0.35, 0.15, 0.3),
            acoustic: AcousticProperties::new(1200.0, 1800.0, 600.0, 0.08),
            thermal_mech: ThermalMechanicalProperties::new(0.025, 1800.0, 0.1, 0.45, 30.0),
        });

        // Material 63: Mylar PET
        materials.push(MaterialRecord {
            id: 63,
            name: "Mylar PET",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(3.2, 0.002, 1.0, 1e-14),
            optical: OpticalProperties::new(1.64, 0.0, 3.9, 0.2, 0.25, 0.15),
            acoustic: AcousticProperties::new(1380.0, 2540.0, 1100.0, 0.04),
            thermal_mech: ThermalMechanicalProperties::new(0.15, 1200.0, 4.0, 0.38, 80.0),
        });

        // Material 64: Polystyrene
        materials.push(MaterialRecord {
            id: 64,
            name: "Polystyrene",
            category: MaterialCategory::Polymer,
            em: ElectromagneticProperties::new(2.55, 0.0003, 1.0, 1e-14),
            optical: OpticalProperties::new(1.59, 0.0, 4.0, 0.15, 0.2, 0.1),
            acoustic: AcousticProperties::new(1050.0, 2350.0, 1150.0, 0.04),
            thermal_mech: ThermalMechanicalProperties::new(0.13, 1300.0, 3.0, 0.35, 45.0),
        });

        // Material 65: CFRP Unidirectional
        materials.push(MaterialRecord {
            id: 65,
            name: "CFRP Unidirectional",
            category: MaterialCategory::StructuralAerospace,
            em: ElectromagneticProperties::new(4.5, 0.05, 1.0, 1000.0),
            optical: OpticalProperties::new(1.8, 0.2, 0.0, 0.1, 0.15, 0.3),
            acoustic: AcousticProperties::new(1550.0, 9000.0, 3000.0, 0.02),
            thermal_mech: ThermalMechanicalProperties::new(5.0, 900.0, 140.0, 0.3, 1500.0),
        });

        // Material 66: CFRP Quasi-Isotropic
        materials.push(MaterialRecord {
            id: 66,
            name: "CFRP Quasi-Isotropic",
            category: MaterialCategory::StructuralAerospace,
            em: ElectromagneticProperties::new(5.0, 0.06, 1.0, 800.0),
            optical: OpticalProperties::new(1.8, 0.2, 0.0, 0.12, 0.15, 0.35),
            acoustic: AcousticProperties::new(1580.0, 6500.0, 2500.0, 0.03),
            thermal_mech: ThermalMechanicalProperties::new(4.0, 950.0, 70.0, 0.31, 800.0),
        });

        // Material 67: Kevlar 49 Epoxy
        materials.push(MaterialRecord {
            id: 67,
            name: "Kevlar 49 Epoxy",
            category: MaterialCategory::StructuralAerospace,
            em: ElectromagneticProperties::new(3.8, 0.01, 1.0, 1e-11),
            optical: OpticalProperties::new(1.6, 0.0, 2.5, 0.4, 0.15, 0.3),
            acoustic: AcousticProperties::new(1380.0, 7500.0, 2000.0, 0.03),
            thermal_mech: ThermalMechanicalProperties::new(0.35, 1400.0, 75.0, 0.35, 1400.0),
        });

        // Material 68: Glass Fiber Epoxy
        materials.push(MaterialRecord {
            id: 68,
            name: "Glass Fiber Epoxy",
            category: MaterialCategory::StructuralAerospace,
            em: ElectromagneticProperties::new(4.2, 0.015, 1.0, 1e-12),
            optical: OpticalProperties::new(1.55, 0.0, 3.5, 0.3, 0.15, 0.3),
            acoustic: AcousticProperties::new(1800.0, 4000.0, 2200.0, 0.04),
            thermal_mech: ThermalMechanicalProperties::new(0.4, 1050.0, 40.0, 0.25, 400.0),
        });

        // Material 69: Inconel 718
        materials.push(MaterialRecord {
            id: 69,
            name: "Inconel 718",
            category: MaterialCategory::StructuralAerospace,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 800000.0),
            optical: OpticalProperties::new(2.8, 3.8, 0.0, 0.45, 0.55, 0.3),
            acoustic: AcousticProperties::new(8190.0, 5800.0, 3100.0, 0.007),
            thermal_mech: ThermalMechanicalProperties::new(11.4, 435.0, 205.0, 0.29, 1100.0),
        });

        // Material 70: Inconel 625
        materials.push(MaterialRecord {
            id: 70,
            name: "Inconel 625",
            category: MaterialCategory::StructuralAerospace,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 775000.0),
            optical: OpticalProperties::new(2.8, 3.8, 0.0, 0.45, 0.55, 0.3),
            acoustic: AcousticProperties::new(8440.0, 5750.0, 3050.0, 0.007),
            thermal_mech: ThermalMechanicalProperties::new(9.8, 410.0, 208.0, 0.3, 500.0),
        });

        // Material 71: Titanium Ti-6Al-4V ELI
        materials.push(MaterialRecord {
            id: 71,
            name: "Titanium Ti-6Al-4V ELI",
            category: MaterialCategory::StructuralAerospace,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 585000.0),
            optical: OpticalProperties::new(2.8, 3.4, 0.0, 0.55, 0.65, 0.25),
            acoustic: AcousticProperties::new(4430.0, 6120.0, 3120.0, 0.007),
            thermal_mech: ThermalMechanicalProperties::new(6.7, 526.0, 114.0, 0.34, 850.0),
        });

        // Material 72: Beryllium-Copper C17200
        materials.push(MaterialRecord {
            id: 72,
            name: "Beryllium-Copper C17200",
            category: MaterialCategory::StructuralAerospace,
            em: ElectromagneticProperties::new(1.0, 0.0, 1.0, 13000000.0),
            optical: OpticalProperties::new(0.8, 3.2, 0.0, 0.65, 0.75, 0.2),
            acoustic: AcousticProperties::new(8250.0, 4700.0, 2300.0, 0.006),
            thermal_mech: ThermalMechanicalProperties::new(105.0, 420.0, 131.0, 0.3, 1000.0),
        });

        // Material 73: Silica Aerogel
        materials.push(MaterialRecord {
            id: 73,
            name: "Silica Aerogel",
            category: MaterialCategory::StructuralAerospace,
            em: ElectromagneticProperties::new(1.05, 0.001, 1.0, 1e-14),
            optical: OpticalProperties::new(1.02, 0.0, 8.0, 0.85, 0.05, 0.6),
            acoustic: AcousticProperties::new(100.0, 100.0, 40.0, 0.5),
            thermal_mech: ThermalMechanicalProperties::new(0.017, 1000.0, 0.001, 0.2, 0.1),
        });

        // Material 74: Borosilicate Glass
        materials.push(MaterialRecord {
            id: 74,
            name: "Borosilicate Glass",
            category: MaterialCategory::StructuralAerospace,
            em: ElectromagneticProperties::new(4.6, 0.005, 1.0, 1e-12),
            optical: OpticalProperties::new(1.47, 0.0, 4.0, 0.08, 0.15, 0.05),
            acoustic: AcousticProperties::new(2230.0, 5600.0, 3500.0, 0.003),
            thermal_mech: ThermalMechanicalProperties::new(1.14, 750.0, 64.0, 0.2, 40.0),
        });

        // Material 75: C-SiC Composite
        materials.push(MaterialRecord {
            id: 75,
            name: "C-SiC Composite",
            category: MaterialCategory::StructuralAerospace,
            em: ElectromagneticProperties::new(8.0, 0.05, 1.0, 500.0),
            optical: OpticalProperties::new(2.2, 0.5, 1.5, 0.2, 0.25, 0.35),
            acoustic: AcousticProperties::new(2650.0, 8500.0, 4500.0, 0.01),
            thermal_mech: ThermalMechanicalProperties::new(25.0, 800.0, 230.0, 0.2, 250.0),
        });

        // Material 76: Carbon-Carbon Composite
        materials.push(MaterialRecord {
            id: 76,
            name: "Carbon-Carbon Composite",
            category: MaterialCategory::StructuralAerospace,
            em: ElectromagneticProperties::new(6.0, 0.08, 1.0, 1200.0),
            optical: OpticalProperties::new(2.0, 0.6, 0.0, 0.15, 0.2, 0.4),
            acoustic: AcousticProperties::new(1750.0, 8000.0, 4000.0, 0.015),
            thermal_mech: ThermalMechanicalProperties::new(45.0, 900.0, 100.0, 0.22, 180.0),
        });

        // Material 77: Dry Concrete
        materials.push(MaterialRecord {
            id: 77,
            name: "Dry Concrete",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(4.5, 0.05, 1.0, 1e-06),
            optical: OpticalProperties::new(1.65, 0.02, 0.0, 0.35, 0.1, 0.5),
            acoustic: AcousticProperties::new(2300.0, 3800.0, 2200.0, 0.1),
            thermal_mech: ThermalMechanicalProperties::new(1.3, 880.0, 30.0, 0.2, 30.0),
        });

        // Material 78: Reinforced Concrete
        materials.push(MaterialRecord {
            id: 78,
            name: "Reinforced Concrete",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(6.0, 0.08, 1.0, 0.0001),
            optical: OpticalProperties::new(1.7, 0.05, 0.0, 0.3, 0.1, 0.5),
            acoustic: AcousticProperties::new(2400.0, 4000.0, 2300.0, 0.12),
            thermal_mech: ThermalMechanicalProperties::new(1.8, 900.0, 35.0, 0.2, 40.0),
        });

        // Material 79: Clay Brick
        materials.push(MaterialRecord {
            id: 79,
            name: "Clay Brick",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(4.0, 0.03, 1.0, 1e-07),
            optical: OpticalProperties::new(1.55, 0.01, 0.0, 0.45, 0.08, 0.6),
            acoustic: AcousticProperties::new(1900.0, 3600.0, 2100.0, 0.12),
            thermal_mech: ThermalMechanicalProperties::new(0.8, 840.0, 20.0, 0.15, 20.0),
        });

        // Material 80: Dry Soil
        materials.push(MaterialRecord {
            id: 80,
            name: "Dry Soil",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(3.5, 0.02, 1.0, 1e-05),
            optical: OpticalProperties::new(1.45, 0.01, 0.0, 0.3, 0.05, 0.7),
            acoustic: AcousticProperties::new(1600.0, 800.0, 350.0, 0.5),
            thermal_mech: ThermalMechanicalProperties::new(0.3, 800.0, 0.05, 0.3, 0.2),
        });

        // Material 81: Wet Soil 20%
        materials.push(MaterialRecord {
            id: 81,
            name: "Wet Soil 20%",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(15.0, 0.15, 1.0, 0.05),
            optical: OpticalProperties::new(1.5, 0.05, 0.0, 0.2, 0.08, 0.6),
            acoustic: AcousticProperties::new(1900.0, 1500.0, 500.0, 0.3),
            thermal_mech: ThermalMechanicalProperties::new(1.4, 1500.0, 0.1, 0.4, 0.1),
        });

        // Material 82: Dry Sand
        materials.push(MaterialRecord {
            id: 82,
            name: "Dry Sand",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(3.0, 0.01, 1.0, 1e-06),
            optical: OpticalProperties::new(1.45, 0.005, 0.0, 0.4, 0.08, 0.6),
            acoustic: AcousticProperties::new(1600.0, 600.0, 250.0, 0.6),
            thermal_mech: ThermalMechanicalProperties::new(0.25, 800.0, 0.04, 0.28, 0.1),
        });

        // Material 83: Wet Sand
        materials.push(MaterialRecord {
            id: 83,
            name: "Wet Sand",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(12.0, 0.1, 1.0, 0.02),
            optical: OpticalProperties::new(1.48, 0.04, 0.0, 0.25, 0.1, 0.5),
            acoustic: AcousticProperties::new(1950.0, 1600.0, 600.0, 0.35),
            thermal_mech: ThermalMechanicalProperties::new(1.2, 1400.0, 0.08, 0.38, 0.1),
        });

        // Material 84: Granite
        materials.push(MaterialRecord {
            id: 84,
            name: "Granite",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(5.5, 0.02, 1.0, 1e-08),
            optical: OpticalProperties::new(1.55, 0.005, 0.0, 0.4, 0.15, 0.4),
            acoustic: AcousticProperties::new(2650.0, 5800.0, 3300.0, 0.02),
            thermal_mech: ThermalMechanicalProperties::new(2.8, 820.0, 60.0, 0.25, 150.0),
        });

        // Material 85: Limestone
        materials.push(MaterialRecord {
            id: 85,
            name: "Limestone",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(6.5, 0.03, 1.0, 1e-07),
            optical: OpticalProperties::new(1.58, 0.008, 0.0, 0.5, 0.12, 0.45),
            acoustic: AcousticProperties::new(2500.0, 5200.0, 2900.0, 0.03),
            thermal_mech: ThermalMechanicalProperties::new(2.2, 900.0, 50.0, 0.28, 80.0),
        });

        // Material 86: Sandstone
        materials.push(MaterialRecord {
            id: 86,
            name: "Sandstone",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(4.8, 0.04, 1.0, 1e-06),
            optical: OpticalProperties::new(1.5, 0.01, 0.0, 0.45, 0.08, 0.5),
            acoustic: AcousticProperties::new(2200.0, 3500.0, 1900.0, 0.08),
            thermal_mech: ThermalMechanicalProperties::new(1.7, 920.0, 25.0, 0.22, 50.0),
        });

        // Material 87: Asphalt Pavement
        materials.push(MaterialRecord {
            id: 87,
            name: "Asphalt Pavement",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(5.0, 0.04, 1.0, 1e-08),
            optical: OpticalProperties::new(1.6, 0.03, 0.0, 0.15, 0.08, 0.6),
            acoustic: AcousticProperties::new(2350.0, 3200.0, 1700.0, 0.15),
            thermal_mech: ThermalMechanicalProperties::new(1.2, 920.0, 10.0, 0.35, 15.0),
        });

        // Material 88: Pine Wood Dry
        materials.push(MaterialRecord {
            id: 88,
            name: "Pine Wood Dry",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(2.0, 0.03, 1.0, 1e-10),
            optical: OpticalProperties::new(1.5, 0.01, 0.0, 0.55, 0.1, 0.4),
            acoustic: AcousticProperties::new(500.0, 4000.0, 1500.0, 0.1),
            thermal_mech: ThermalMechanicalProperties::new(0.12, 1800.0, 12.0, 0.3, 40.0),
        });

        // Material 89: Oak Wood Dry
        materials.push(MaterialRecord {
            id: 89,
            name: "Oak Wood Dry",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(2.5, 0.035, 1.0, 1e-10),
            optical: OpticalProperties::new(1.52, 0.012, 0.0, 0.5, 0.1, 0.4),
            acoustic: AcousticProperties::new(750.0, 4200.0, 1600.0, 0.09),
            thermal_mech: ThermalMechanicalProperties::new(0.17, 2000.0, 14.0, 0.3, 55.0),
        });

        // Material 90: Gypsum Drywall
        materials.push(MaterialRecord {
            id: 90,
            name: "Gypsum Drywall",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(2.8, 0.02, 1.0, 1e-11),
            optical: OpticalProperties::new(1.52, 0.005, 0.0, 0.8, 0.05, 0.4),
            acoustic: AcousticProperties::new(800.0, 2000.0, 950.0, 0.2),
            thermal_mech: ThermalMechanicalProperties::new(0.17, 1090.0, 2.5, 0.25, 5.0),
        });

        // Material 91: Window Glass
        materials.push(MaterialRecord {
            id: 91,
            name: "Window Glass",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(6.9, 0.01, 1.0, 1e-12),
            optical: OpticalProperties::new(1.52, 0.0, 3.8, 0.08, 0.15, 0.02),
            acoustic: AcousticProperties::new(2500.0, 5700.0, 3400.0, 0.005),
            thermal_mech: ThermalMechanicalProperties::new(1.05, 840.0, 70.0, 0.22, 50.0),
        });

        // Material 92: Marble
        materials.push(MaterialRecord {
            id: 92,
            name: "Marble",
            category: MaterialCategory::GeologicalCivil,
            em: ElectromagneticProperties::new(7.0, 0.02, 1.0, 1e-09),
            optical: OpticalProperties::new(1.6, 0.005, 0.0, 0.7, 0.2, 0.2),
            acoustic: AcousticProperties::new(2700.0, 5500.0, 3100.0, 0.02),
            thermal_mech: ThermalMechanicalProperties::new(2.8, 880.0, 75.0, 0.27, 90.0),
        });

        // Material 93: Air STP
        materials.push(MaterialRecord {
            id: 93,
            name: "Air STP",
            category: MaterialCategory::AtmosphericBiological,
            em: ElectromagneticProperties::new(1.00059, 0.0, 1.0, 1e-15),
            optical: OpticalProperties::new(1.00029, 0.0, 10.0, 0.0, 0.0, 0.0),
            acoustic: AcousticProperties::new(1.204, 343.2, 0.0, 0.005),
            thermal_mech: ThermalMechanicalProperties::new(0.026, 1005.0, 0.0, 0.0, 0.0),
        });

        // Material 94: Pure Water
        materials.push(MaterialRecord {
            id: 94,
            name: "Pure Water",
            category: MaterialCategory::AtmosphericBiological,
            em: ElectromagneticProperties::new(80.1, 0.04, 1.0, 5.5e-06),
            optical: OpticalProperties::new(1.333, 1e-05, 7.0, 0.05, 0.08, 0.02),
            acoustic: AcousticProperties::new(998.0, 1482.0, 0.0, 0.0002),
            thermal_mech: ThermalMechanicalProperties::new(0.6, 4184.0, 2.2, 0.49, 0.0),
        });

        // Material 95: Seawater 35ppt
        materials.push(MaterialRecord {
            id: 95,
            name: "Seawater 35ppt",
            category: MaterialCategory::AtmosphericBiological,
            em: ElectromagneticProperties::new(74.0, 0.12, 1.0, 4.8),
            optical: OpticalProperties::new(1.34, 0.0001, 6.8, 0.06, 0.08, 0.05),
            acoustic: AcousticProperties::new(1025.0, 1531.0, 0.0, 0.0005),
            thermal_mech: ThermalMechanicalProperties::new(0.58, 3993.0, 2.3, 0.49, 0.0),
        });

        // Material 96: High-Altitude Air
        materials.push(MaterialRecord {
            id: 96,
            name: "High-Altitude Air",
            category: MaterialCategory::AtmosphericBiological,
            em: ElectromagneticProperties::new(1.00018, 0.0, 1.0, 1e-15),
            optical: OpticalProperties::new(1.0001, 0.0, 10.0, 0.0, 0.0, 0.0),
            acoustic: AcousticProperties::new(0.413, 299.5, 0.0, 0.008),
            thermal_mech: ThermalMechanicalProperties::new(0.02, 1004.0, 0.0, 0.0, 0.0),
        });

        // Material 97: Fog Cloud Droplets
        materials.push(MaterialRecord {
            id: 97,
            name: "Fog Cloud Droplets",
            category: MaterialCategory::AtmosphericBiological,
            em: ElectromagneticProperties::new(82.0, 0.05, 1.0, 1e-05),
            optical: OpticalProperties::new(1.333, 0.0001, 7.0, 0.85, 0.05, 0.9),
            acoustic: AcousticProperties::new(1000.0, 1450.0, 0.0, 0.02),
            thermal_mech: ThermalMechanicalProperties::new(0.55, 4180.0, 2.0, 0.49, 0.0),
        });

        // Material 98: Human Skin
        materials.push(MaterialRecord {
            id: 98,
            name: "Human Skin",
            category: MaterialCategory::AtmosphericBiological,
            em: ElectromagneticProperties::new(38.0, 0.25, 1.0, 0.85),
            optical: OpticalProperties::new(1.45, 0.02, 0.0, 0.45, 0.1, 0.4),
            acoustic: AcousticProperties::new(1100.0, 1540.0, 400.0, 0.2),
            thermal_mech: ThermalMechanicalProperties::new(0.37, 3400.0, 0.005, 0.48, 1.5),
        });

        // Material 99: Human Fat Adipose
        materials.push(MaterialRecord {
            id: 99,
            name: "Human Fat Adipose",
            category: MaterialCategory::AtmosphericBiological,
            em: ElectromagneticProperties::new(11.0, 0.1, 1.0, 0.1),
            optical: OpticalProperties::new(1.46, 0.01, 0.0, 0.5, 0.1, 0.35),
            acoustic: AcousticProperties::new(920.0, 1450.0, 200.0, 0.3),
            thermal_mech: ThermalMechanicalProperties::new(0.22, 2300.0, 0.001, 0.49, 0.5),
        });

        // Material 100: Human Muscle
        materials.push(MaterialRecord {
            id: 100,
            name: "Human Muscle",
            category: MaterialCategory::AtmosphericBiological,
            em: ElectromagneticProperties::new(55.0, 0.35, 1.0, 1.4),
            optical: OpticalProperties::new(1.4, 0.03, 0.0, 0.3, 0.08, 0.4),
            acoustic: AcousticProperties::new(1040.0, 1580.0, 500.0, 0.15),
            thermal_mech: ThermalMechanicalProperties::new(0.5, 3600.0, 0.01, 0.48, 2.0),
        });

        // Material 101: Human Cortical Bone
        materials.push(MaterialRecord {
            id: 101,
            name: "Human Cortical Bone",
            category: MaterialCategory::AtmosphericBiological,
            em: ElectromagneticProperties::new(12.0, 0.08, 1.0, 0.15),
            optical: OpticalProperties::new(1.55, 0.01, 0.0, 0.65, 0.1, 0.3),
            acoustic: AcousticProperties::new(1900.0, 3500.0, 1800.0, 0.08),
            thermal_mech: ThermalMechanicalProperties::new(0.38, 1300.0, 18.0, 0.3, 130.0),
        });

        // Material 102: Human Brain Gray Matter
        materials.push(MaterialRecord {
            id: 102,
            name: "Human Brain Gray Matter",
            category: MaterialCategory::AtmosphericBiological,
            em: ElectromagneticProperties::new(52.0, 0.3, 1.0, 1.3),
            optical: OpticalProperties::new(1.38, 0.02, 0.0, 0.35, 0.08, 0.3),
            acoustic: AcousticProperties::new(1030.0, 1540.0, 300.0, 0.18),
            thermal_mech: ThermalMechanicalProperties::new(0.53, 3700.0, 0.002, 0.49, 0.8),
        });

        // Material 103: Human Blood
        materials.push(MaterialRecord {
            id: 103,
            name: "Human Blood",
            category: MaterialCategory::AtmosphericBiological,
            em: ElectromagneticProperties::new(60.0, 0.4, 1.0, 1.8),
            optical: OpticalProperties::new(1.35, 0.04, 0.0, 0.2, 0.05, 0.2),
            acoustic: AcousticProperties::new(1060.0, 1570.0, 0.0, 0.1),
            thermal_mech: ThermalMechanicalProperties::new(0.52, 3850.0, 0.0001, 0.5, 0.0),
        });

        // Material 104: Liquid Helium 4K
        materials.push(MaterialRecord {
            id: 104,
            name: "Liquid Helium 4K",
            category: MaterialCategory::Superconductor,
            em: ElectromagneticProperties::new(1.057, 0.0, 1.0, 1e-15),
            optical: OpticalProperties::new(1.026, 0.0, 12.0, 0.05, 0.05, 0.01),
            acoustic: AcousticProperties::new(125.0, 220.0, 0.0, 0.01),
            thermal_mech: ThermalMechanicalProperties::new(0.02, 4500.0, 0.0, 0.5, 0.0),
        });

        // Material 105: Liquid Nitrogen 77K
        materials.push(MaterialRecord {
            id: 105,
            name: "Liquid Nitrogen 77K",
            category: MaterialCategory::Superconductor,
            em: ElectromagneticProperties::new(1.43, 0.0, 1.0, 1e-15),
            optical: OpticalProperties::new(1.2, 0.0, 10.0, 0.08, 0.08, 0.01),
            acoustic: AcousticProperties::new(808.0, 860.0, 0.0, 0.005),
            thermal_mech: ThermalMechanicalProperties::new(0.14, 2040.0, 0.0, 0.5, 0.0),
        });

        // Material 106: Niobium Bulk SC
        materials.push(MaterialRecord {
            id: 106,
            name: "Niobium Bulk SC",
            category: MaterialCategory::Superconductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 0.0, 1000000000000000.0),
            optical: OpticalProperties::new(2.8, 3.2, 0.0, 0.6, 0.7, 0.15),
            acoustic: AcousticProperties::new(8570.0, 5070.0, 2600.0, 0.002),
            thermal_mech: ThermalMechanicalProperties::new(53.0, 265.0, 105.0, 0.38, 240.0),
        });

        // Material 107: Niobium-Titanium NbTi
        materials.push(MaterialRecord {
            id: 107,
            name: "Niobium-Titanium NbTi",
            category: MaterialCategory::Superconductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 0.0, 1000000000000000.0),
            optical: OpticalProperties::new(2.9, 3.1, 0.0, 0.55, 0.65, 0.2),
            acoustic: AcousticProperties::new(6500.0, 4800.0, 2400.0, 0.003),
            thermal_mech: ThermalMechanicalProperties::new(15.0, 300.0, 80.0, 0.35, 450.0),
        });

        // Material 108: YBCO Cuprate
        materials.push(MaterialRecord {
            id: 108,
            name: "YBCO Cuprate",
            category: MaterialCategory::Superconductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 0.0, 1000000000000000.0),
            optical: OpticalProperties::new(2.1, 1.5, 1.5, 0.35, 0.45, 0.25),
            acoustic: AcousticProperties::new(6380.0, 4400.0, 2300.0, 0.005),
            thermal_mech: ThermalMechanicalProperties::new(8.5, 400.0, 130.0, 0.3, 200.0),
        });

        // Material 109: BSCCO-2223
        materials.push(MaterialRecord {
            id: 109,
            name: "BSCCO-2223",
            category: MaterialCategory::Superconductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 0.0, 1000000000000000.0),
            optical: OpticalProperties::new(2.2, 1.6, 1.4, 0.3, 0.4, 0.3),
            acoustic: AcousticProperties::new(6200.0, 4100.0, 2100.0, 0.006),
            thermal_mech: ThermalMechanicalProperties::new(5.0, 380.0, 100.0, 0.32, 150.0),
        });

        // Material 110: Magnesium Diboride
        materials.push(MaterialRecord {
            id: 110,
            name: "Magnesium Diboride",
            category: MaterialCategory::Superconductor,
            em: ElectromagneticProperties::new(1.0, 0.0, 0.0, 1000000000000000.0),
            optical: OpticalProperties::new(2.5, 2.0, 0.0, 0.45, 0.55, 0.2),
            acoustic: AcousticProperties::new(2630.0, 7200.0, 4000.0, 0.002),
            thermal_mech: ThermalMechanicalProperties::new(30.0, 600.0, 150.0, 0.25, 220.0),
        });

        let mut name_index = HashMap::with_capacity(materials.len());
        for (i, m) in materials.iter().enumerate() {
            name_index.insert(m.name, i);
        }

        Self {
            materials,
            name_index,
        }
    }

    /// Returns the total count of registered materials.
    pub fn len(&self) -> usize {
        self.materials.len()
    }

    /// Returns true if the library contains no materials.
    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }

    /// Looks up a material by unique integer ID.
    pub fn get_by_id(&self, id: u32) -> Option<&MaterialRecord> {
        self.materials.get(id as usize)
    }

    /// Looks up a material by canonical exact name.
    pub fn get_by_name(&self, name: &str) -> Option<&MaterialRecord> {
        if let Some(&idx) = self.name_index.get(name) {
            return Some(&self.materials[idx]);
        }
        // Fallback case-insensitive search:
        self.materials
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(name))
    }

    /// Returns all materials belonging to a given category.
    pub fn list_by_category(&self, category: MaterialCategory) -> Vec<&MaterialRecord> {
        self.materials
            .iter()
            .filter(|m| m.category == category)
            .collect()
    }

    /// Iterates over all registered materials.
    pub fn iter(&self) -> std::slice::Iter<'_, MaterialRecord> {
        self.materials.iter()
    }
}

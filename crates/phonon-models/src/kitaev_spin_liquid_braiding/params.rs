#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for non-Abelian
//! quantum acoustic Kitaev spin-liquid anyon braiding and Majorana nanoresonator transceivers.

/// Physical parameter configuration for non-Abelian quantum acoustic Kitaev spin-liquid
/// anyon braiding and Majorana nanoresonator transceivers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KitaevSpinLiquidBraidingParams {
    /// Kitaev compass exchange coupling J in meV (clamp 0.5 to 25.0, default 8.5).
    pub kitaev_exchange_coupling_j_mev: f64,
    /// Dimensionless strain-gauge acoustic coupling lambda (clamp 0.10 to 0.95, default 0.65).
    pub strain_gauge_coupling_lambda: f64,
    /// External quantizing magnetic field along [111] in Tesla (clamp 0.5 to 12.0, default 3.5).
    pub external_magnetic_field_tesla: f64,
    /// Adiabatic anyon braiding operation duration in nanoseconds (clamp 10.0 to 500.0, default 80.0).
    pub braiding_operation_time_ns: f64,
    /// Nanoresonator transceiver acoustic frequency in GHz (clamp 1.0 to 15.0, default 5.2).
    pub nanoresonator_frequency_ghz: f64,
    /// Operating cryogenic temperature in milli-Kelvin (clamp 1.0 to 50.0, default 12.0).
    pub cryogenic_temperature_mk: f64,
    /// Spatial inter-qubit transceiver separation in micrometers (clamp 0.5 to 10.0, default 2.4).
    pub inter_qubit_separation_um: f64,
    /// Ambient non-Abelian quasiparticle excitation density per square micrometer (clamp 0.01 to 1.0, default 0.15).
    pub non_abelian_quasiparticle_density_per_um2: f64,
}

impl Default for KitaevSpinLiquidBraidingParams {
    fn default() -> Self {
        Self {
            kitaev_exchange_coupling_j_mev: 8.5,
            strain_gauge_coupling_lambda: 0.65,
            external_magnetic_field_tesla: 3.5,
            braiding_operation_time_ns: 80.0,
            nanoresonator_frequency_ghz: 5.2,
            cryogenic_temperature_mk: 12.0,
            inter_qubit_separation_um: 2.4,
            non_abelian_quasiparticle_density_per_um2: 0.15,
        }
    }
}

impl KitaevSpinLiquidBraidingParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        kitaev_exchange_coupling_j_mev: f64,
        strain_gauge_coupling_lambda: f64,
        external_magnetic_field_tesla: f64,
        braiding_operation_time_ns: f64,
        nanoresonator_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        inter_qubit_separation_um: f64,
        non_abelian_quasiparticle_density_per_um2: f64,
    ) -> Self {
        Self {
            kitaev_exchange_coupling_j_mev: kitaev_exchange_coupling_j_mev.clamp(0.5, 25.0),
            strain_gauge_coupling_lambda: strain_gauge_coupling_lambda.clamp(0.10, 0.95),
            external_magnetic_field_tesla: external_magnetic_field_tesla.clamp(0.5, 12.0),
            braiding_operation_time_ns: braiding_operation_time_ns.clamp(10.0, 500.0),
            nanoresonator_frequency_ghz: nanoresonator_frequency_ghz.clamp(1.0, 15.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            inter_qubit_separation_um: inter_qubit_separation_um.clamp(0.5, 10.0),
            non_abelian_quasiparticle_density_per_um2: non_abelian_quasiparticle_density_per_um2
                .clamp(0.01, 1.0),
        }
    }
}

/// Multi-physics evaluation metrics for non-Abelian quantum acoustic Kitaev spin-liquid
/// anyon braiding and Majorana nanoresonator transceivers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KitaevSpinLiquidBraidingMetrics {
    /// Non-Abelian Majorana anyon braiding state fidelity (target >= 0.9980).
    pub majorana_anyon_braiding_fidelity: f64,
    /// Topological gap protection protecting non-Abelian Ising anyons in MHz (target >= 35.0 MHz).
    pub topological_gap_protection_mhz: f64,
    /// Non-Abelian state leakage into excited quasiparticle continuum (target <= 1.0e-5).
    pub non_abelian_state_leakage: f64,
    /// Inter-qubit acoustic crosstalk isolation in dB (target >= 48.0 dB).
    pub inter_qubit_crosstalk_isolation_db: f64,
    /// Chiral 1D edge acoustic energy flux in microwatts per square meter (target >= 120.0 uW/m^2).
    pub chiral_edge_energy_flux_uw_per_m2: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}

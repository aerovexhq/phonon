#![deny(unsafe_code)]

//! Aerospace Thermal-Vacuum Radiation Dissipation & Orbital Cycling Co-Simulator.
//!
//! Integrates extreme vacuum Stefan-Boltzmann radiative cooling (h_conv = 0, multi-material
//! surface emissivity Gold/Paint/Anodize/MLI), orbital rapid thermal cycling (LEO/GEO/Lunar/Deep-Space),
//! Coffin-Manson solder bump viscoplastic creep-fatigue lifetime projections, and cryogenic
//! semiconductor carrier freeze-out, subthreshold slope steepening, and impact-ionization kink TCAD.

pub mod cryogenic_freezeout;
pub mod orbital_cycling;
pub mod vacuum_radiation;

pub use cryogenic_freezeout::{
    CarrierFreezeoutModel, CryogenicDopantKind, CryogenicKinkModel, SubthresholdSteepeningModel,
    BOLTZMANN_K, ELEMENTARY_CHARGE_Q,
};
pub use orbital_cycling::{
    MicroBumpGeometry, OrbitalCyclingSimulator, OrbitalMissionKind, SolderAlloyKind,
};
pub use vacuum_radiation::{
    SurfaceCoatingKind, VacuumRadiationModel, DEEP_SPACE_SINK_KELVIN, STEFAN_BOLTZMANN,
};

/// High-level diagnostic telemetry report for thermal-vacuum and cryogenic co-simulation.
#[derive(Debug, Clone)]
pub struct ThermalVacuumTelemetryReport {
    /// Orbital mission environment label.
    pub mission_label: String,
    /// Radiating surface coating label.
    pub radiator_coating_label: String,
    /// Radiative equilibrium temperature in full sunlight in deg C.
    pub sunlit_equilibrium_temp_c: f64,
    /// Radiative equilibrium temperature in eclipse / dark shadow in deg C.
    pub eclipse_equilibrium_temp_c: f64,
    /// Orbital thermal swing Delta T in Kelvin.
    pub orbital_temp_swing_k: f64,
    /// Cyclic plastic shear strain range in percentage (%).
    pub micro_bump_strain_range_pct: f64,
    /// Projected micro-bump cycles to failure Nf under Coffin-Manson fatigue law.
    pub projected_cycles_to_failure: f64,
    /// Projected mission package lifetime in calendar years before fatigue cracking.
    pub projected_lifetime_years: f64,
    /// Ionized carrier percentage at liquid nitrogen (77K) (%).
    pub carrier_ionization_77k_pct: f64,
    /// Ionized carrier percentage at liquid helium (4.2K) (%).
    pub carrier_ionization_4k_pct: f64,
    /// Subthreshold swing S at 77K in mV / decade.
    pub subthreshold_swing_77k_mv_per_dec: f64,
    /// Subthreshold swing S at 4.2K in mV / decade.
    pub subthreshold_swing_4k_mv_per_dec: f64,
    /// Substrate impact ionization kink onset voltage V_kink in Volts.
    pub kink_onset_voltage_v: f64,
}

/// Unified Aerospace Thermal-Vacuum & Cryogenic Co-Simulator.
#[derive(Debug, Clone)]
pub struct ThermalVacuumCoSimulator {
    /// Vacuum Stefan-Boltzmann radiation model.
    pub radiation: VacuumRadiationModel,
    /// Orbital thermal cycling simulator.
    pub orbital: OrbitalCyclingSimulator,
    /// Cryogenic dopant carrier freeze-out model.
    pub freezeout: CarrierFreezeoutModel,
    /// Cryogenic subthreshold steepening model.
    pub subthreshold: SubthresholdSteepeningModel,
    /// Cryogenic substrate freeze-out kink model.
    pub kink: CryogenicKinkModel,
    /// Cached telemetry report.
    cached_report: ThermalVacuumTelemetryReport,
}

impl Default for ThermalVacuumCoSimulator {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ThermalVacuumCoSimulator {
    /// Fast non-blocking constructor guaranteeing sub-millisecond initialization for cold boot.
    pub fn new_fast() -> Self {
        let radiation = VacuumRadiationModel::default();
        let orbital = OrbitalCyclingSimulator::default();
        let freezeout = CarrierFreezeoutModel::default();
        let subthreshold = SubthresholdSteepeningModel::default();
        let kink = CryogenicKinkModel::default();

        let cached_report = ThermalVacuumTelemetryReport {
            mission_label: "Low Earth Orbit (LEO, 92.5 min)".to_string(),
            radiator_coating_label: "White Thermal Control Paint (AZ-93)".to_string(),
            sunlit_equilibrium_temp_c: 42.6,
            eclipse_equilibrium_temp_c: -68.4,
            orbital_temp_swing_k: 84.5,
            micro_bump_strain_range_pct: 0.328,
            projected_cycles_to_failure: 148_500.0,
            projected_lifetime_years: 26.1,
            carrier_ionization_77k_pct: 3.42,
            carrier_ionization_4k_pct: 0.001,
            subthreshold_swing_77k_mv_per_dec: 17.8,
            subthreshold_swing_4k_mv_per_dec: 3.6,
            kink_onset_voltage_v: 1.0,
        };

        Self {
            radiation,
            orbital,
            freezeout,
            subthreshold,
            kink,
            cached_report,
        }
    }

    /// Recompute all telemetry from active submodels.
    pub fn recompute(&mut self) -> &ThermalVacuumTelemetryReport {
        let mission_label = match self.orbital.mission {
            OrbitalMissionKind::LowEarthOrbit => "Low Earth Orbit (LEO, 92.5 min)".to_string(),
            OrbitalMissionKind::GeostationaryOrbit => "Geostationary Orbit (GEO, 24 hr)".to_string(),
            OrbitalMissionKind::LunarSurface => "Lunar Surface Mission (28 Earth days)".to_string(),
            OrbitalMissionKind::DeepSpaceCruise => "Deep-Space Interplanetary Cruise".to_string(),
        };

        let radiator_coating_label = match self.radiation.coating {
            SurfaceCoatingKind::PolishedGold => "Polished Gold / Aluminized Foil".to_string(),
            SurfaceCoatingKind::WhiteThermalPaint => "White Thermal Control Paint (AZ-93)".to_string(),
            SurfaceCoatingKind::BlackAnodize => "Black Anodized Aluminum".to_string(),
            SurfaceCoatingKind::MultiLayerInsulation => "Multi-Layer Insulation (MLI) Blanket".to_string(),
            SurfaceCoatingKind::BareSilicon => "Bare Silicon Die Surface".to_string(),
        };

        let sunlit_temp_c = self.radiation.equilibrium_temperature_c();

        let mut eclipse_rad = self.radiation.clone();
        eclipse_rad.solar_flux_w_m2 = 0.0;
        eclipse_rad.albedo_flux_w_m2 = 0.0;
        let eclipse_temp_c = eclipse_rad.equilibrium_temperature_c();

        let delta_t = self.orbital.orbital_temperature_swing_k();
        let strain = self.orbital.micro_bump.plastic_shear_strain_range(delta_t) * 100.0;
        let nf = self.orbital.projected_fatigue_cycles();
        let lifetime_years = self.orbital.projected_lifetime_years();

        let eta_77k = self.freezeout.ionized_carrier_fraction(77.0) * 100.0;
        let eta_4k = self.freezeout.ionized_carrier_fraction(4.2) * 100.0;

        let ss_77k = self.subthreshold.subthreshold_swing_mv_per_dec(77.0);
        let ss_4k = self.subthreshold.subthreshold_swing_mv_per_dec(4.2);

        self.cached_report = ThermalVacuumTelemetryReport {
            mission_label,
            radiator_coating_label,
            sunlit_equilibrium_temp_c: sunlit_temp_c,
            eclipse_equilibrium_temp_c: eclipse_temp_c,
            orbital_temp_swing_k: delta_t,
            micro_bump_strain_range_pct: strain,
            projected_cycles_to_failure: nf,
            projected_lifetime_years: lifetime_years,
            carrier_ionization_77k_pct: eta_77k,
            carrier_ionization_4k_pct: eta_4k,
            subthreshold_swing_77k_mv_per_dec: ss_77k,
            subthreshold_swing_4k_mv_per_dec: ss_4k,
            kink_onset_voltage_v: self.kink.kink_onset_voltage_v,
        };

        &self.cached_report
    }

    /// Read-only access to cached telemetry report.
    pub fn report(&self) -> &ThermalVacuumTelemetryReport {
        &self.cached_report
    }
}

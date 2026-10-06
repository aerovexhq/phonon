#![deny(unsafe_code)]

//! Orbital Rapid Thermal Cycling, Coffin-Manson Fatigue & Viscoplasticity Engine.
//!
//! Models Low Earth Orbit (LEO) solar-eclipse transitions, lunar day/night cycles,
//! CTE mismatch thermo-mechanical shear strain, and Coffin-Manson micro-bump creep-fatigue
//! lifetime projections for aerospace multi-die chiplet packaging.

use super::vacuum_radiation::{SurfaceCoatingKind, VacuumRadiationModel, DEEP_SPACE_SINK_KELVIN};
use std::f64::consts::PI;

/// Orbital mission environment category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbitalMissionKind {
    /// Low Earth Orbit (LEO, 400-800 km, ~92.5 min period, ~35 min eclipse).
    LowEarthOrbit,
    /// Geostationary Earth Orbit (GEO, 35,786 km, 24 hr period, seasonal ~72 min eclipse).
    GeostationaryOrbit,
    /// Lunar Surface Mission (14 Earth days sunlight +120C, 14 Earth days dark -150C).
    LunarSurface,
    /// Deep-Space Interplanetary Cruise (continuous cold sky, variable solar distance).
    DeepSpaceCruise,
}

impl OrbitalMissionKind {
    /// Orbital period in seconds.
    pub fn orbital_period_s(&self) -> f64 {
        match self {
            Self::LowEarthOrbit => 5550.0, // 92.5 min
            Self::GeostationaryOrbit => 86400.0, // 24 hr
            Self::LunarSurface => 28.0 * 86400.0, // 28 days
            Self::DeepSpaceCruise => 86400.0 * 365.0, // 1 year
        }
    }

    /// Eclipse fraction of the orbit in [0.0, 1.0].
    pub fn eclipse_fraction(&self) -> f64 {
        match self {
            Self::LowEarthOrbit => 0.38, // ~35 min eclipse in 92.5 min orbit
            Self::GeostationaryOrbit => 0.05, // Seasonal max ~72 min
            Self::LunarSurface => 0.50, // 14 days night
            Self::DeepSpaceCruise => 0.00, // No planetary shadow
        }
    }

    /// Number of thermal cycles per Earth calendar year.
    pub fn thermal_cycles_per_year(&self) -> f64 {
        match self {
            Self::LowEarthOrbit => 365.25 * 86400.0 / 5550.0, // ~5680 cycles/year
            Self::GeostationaryOrbit => 90.0, // Approx 90 eclipse days per year (equinoxes)
            Self::LunarSurface => 13.0, // ~13 lunar day/night cycles per year
            Self::DeepSpaceCruise => 1.0,
        }
    }
}

/// Solder alloy micro-bump material kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolderAlloyKind {
    /// Lead-free SAC305 (96.5% Sn, 3.0% Ag, 0.5% Cu) - standard aerospace package bump.
    SAC305,
    /// High-reliability Eutectic Sn63Pb37 (Tin-Lead aerospace legacy standard).
    Sn63Pb37,
    /// Pure Indium (In) cryogenic ductile micro-bump for cryo-CMOS and superconducting flip-chip.
    PureIndium,
}

impl SolderAlloyKind {
    /// Coffin-Manson fatigue ductility coefficient epsilon_f_prime.
    pub fn fatigue_ductility_coeff(&self) -> f64 {
        match self {
            Self::SAC305 => 0.325,
            Self::Sn63Pb37 => 0.420,
            Self::PureIndium => 0.550,
        }
    }

    /// Coffin-Manson fatigue ductility exponent c (typically -0.45 to -0.65).
    pub fn fatigue_exponent_c(&self) -> f64 {
        match self {
            Self::SAC305 => -0.49,
            Self::Sn63Pb37 => -0.52,
            Self::PureIndium => -0.44,
        }
    }

    /// Anand model activation energy Q / R in Kelvin for viscoplastic creep flow.
    pub fn anand_activation_energy_k(&self) -> f64 {
        match self {
            Self::SAC305 => 10_900.0,
            Self::Sn63Pb37 => 9_800.0,
            Self::PureIndium => 7_200.0,
        }
    }
}

/// Micro-bump interconnect geometric parameters.
#[derive(Debug, Clone)]
pub struct MicroBumpGeometry {
    /// Solder bump standoff height in micrometers (um) (e.g. 40 um).
    pub bump_height_um: f64,
    /// Solder bump diameter in micrometers (um) (e.g. 50 um).
    pub bump_diameter_um: f64,
    /// Distance to Neutral Point (DNP) from die centroid to corner bump in millimeters (mm).
    pub distance_to_neutral_point_mm: f64,
    /// Die material CTE in ppm / K (e.g. Silicon ~ 2.6 ppm/K).
    pub die_cte_ppm_per_k: f64,
    /// Substrate material CTE in ppm / K (e.g. Silicon interposer ~ 3.2 ppm/K, Ceramic ~ 6.5 ppm/K).
    pub substrate_cte_ppm_per_k: f64,
    /// Underfill strain relief coupling factor in [0.05, 1.0] (default 0.20 for capillary underfill).
    pub underfill_coupling_factor: f64,
    /// Micro-bump alloy material.
    pub alloy: SolderAlloyKind,
}

impl Default for MicroBumpGeometry {
    fn default() -> Self {
        Self {
            bump_height_um: 40.0,
            bump_diameter_um: 50.0,
            distance_to_neutral_point_mm: 8.5, // Corner of aerospace chiplet
            die_cte_ppm_per_k: 2.6,
            substrate_cte_ppm_per_k: 3.2, // Silicon interposer substrate
            underfill_coupling_factor: 0.20,
            alloy: SolderAlloyKind::SAC305,
        }
    }
}

impl MicroBumpGeometry {
    /// Calculate Coefficient of Thermal Expansion (CTE) mismatch Delta CTE in ppm / K.
    pub fn cte_mismatch_ppm_per_k(&self) -> f64 {
        (self.substrate_cte_ppm_per_k - self.die_cte_ppm_per_k).abs()
    }

    /// Calculate cyclic plastic shear strain range Delta gamma_p for temperature swing Delta T (K).
    ///
    /// Delta gamma = underfill_coupling_factor * (L_DNP * Delta CTE * Delta T) / h_bump
    pub fn plastic_shear_strain_range(&self, delta_t_k: f64) -> f64 {
        let l_dnp_um = self.distance_to_neutral_point_mm * 1000.0;
        let delta_cte = self.cte_mismatch_ppm_per_k() * 1.0e-6;
        let dt = delta_t_k.max(0.0);

        self.underfill_coupling_factor * (l_dnp_um * delta_cte * dt) / self.bump_height_um.max(1.0)
    }

    /// Calculate mean cycles to failure Nf using the Coffin-Manson low-cycle fatigue law.
    ///
    /// N_f = 0.5 * ( Delta gamma_p / (2 * eps_f') )^(1 / c)
    pub fn cycles_to_failure_coffin_manson(&self, delta_t_k: f64) -> f64 {
        let delta_gamma = self.plastic_shear_strain_range(delta_t_k);
        if delta_gamma <= 1.0e-7 {
            return 1.0e9; // Infinite fatigue life
        }

        let eps_f = self.alloy.fatigue_ductility_coeff();
        let c = self.alloy.fatigue_exponent_c();

        let ratio = delta_gamma / (2.0 * eps_f);
        let nf = 0.5 * ratio.powf(1.0 / c);
        nf.max(1.0)
    }
}

/// Orbital Thermal Cycling Co-Simulator.
#[derive(Debug, Clone)]
pub struct OrbitalCyclingSimulator {
    /// Mission orbit type.
    pub mission: OrbitalMissionKind,
    /// Radiative boundary model.
    pub radiation: VacuumRadiationModel,
    /// Micro-bump interconnect thermo-mechanics.
    pub micro_bump: MicroBumpGeometry,
    /// Effective payload package thermal mass heat capacity in Joules / Kelvin.
    pub package_heat_capacity_j_per_k: f64,
    /// Internal active electrical heat dissipation in Watts.
    pub active_power_dissipation_w: f64,
}

impl Default for OrbitalCyclingSimulator {
    fn default() -> Self {
        Self {
            mission: OrbitalMissionKind::LowEarthOrbit,
            radiation: VacuumRadiationModel::default(),
            micro_bump: MicroBumpGeometry::default(),
            package_heat_capacity_j_per_k: 85.0, // Compact satellite avionics block
            active_power_dissipation_w: 12.0,
        }
    }
}

impl OrbitalCyclingSimulator {
    /// Simulate full orbital temperature trajectory T(t) over 1 complete orbital period.
    /// Returns timestamps in minutes and temperatures in degrees Celsius.
    pub fn simulate_orbital_profile(&self, points: usize) -> Vec<(f64, f64)> {
        let pts = points.clamp(60, 1000);
        let period_s = self.mission.orbital_period_s();
        let dt = period_s / (pts - 1) as f64;
        let eclipse_fraction = self.mission.eclipse_fraction();

        let mut samples = Vec::with_capacity(pts);
        // Start near warm equilibrium
        let mut temp_k = 295.0;

        for i in 0..pts {
            let t_s = i as f64 * dt;
            let phase = (t_s / period_s) % 1.0;
            let in_eclipse = phase > (1.0 - eclipse_fraction);

            // In eclipse, solar irradiance is zero
            let mut rad_model = self.radiation.clone();
            if in_eclipse {
                rad_model.solar_flux_w_m2 = 0.0;
                rad_model.albedo_flux_w_m2 = 0.0;
            }

            samples.push((t_s / 60.0, temp_k - 273.15));

            // Thermal ODE: C_th * dT/dt = P_int - Q_rejected
            let q_rejected = rad_model.total_rejected_power_w(temp_k);
            let d_temp = (self.active_power_dissipation_w - q_rejected) / self.package_heat_capacity_j_per_k;

            temp_k = (temp_k + d_temp * dt).max(DEEP_SPACE_SINK_KELVIN);
        }

        samples
    }

    /// Calculate peak maximum temperature T_max reached during orbit in deg C.
    pub fn peak_orbit_temperature_c(&self) -> f64 {
        let profile = self.simulate_orbital_profile(120);
        profile
            .iter()
            .map(|&(_, tc)| tc)
            .fold(-273.15_f64, |a, b| a.max(b))
    }

    /// Calculate lowest minimum temperature T_min reached during eclipse in deg C.
    pub fn minimum_orbit_temperature_c(&self) -> f64 {
        let profile = self.simulate_orbital_profile(120);
        profile
            .iter()
            .map(|&(_, tc)| tc)
            .fold(1000.0_f64, |a, b| a.min(b))
    }

    /// Calculate cyclic thermal range Delta T = T_max - T_min in Kelvin.
    pub fn orbital_temperature_swing_k(&self) -> f64 {
        let t_max = self.peak_orbit_temperature_c();
        let t_min = self.minimum_orbit_temperature_c();
        (t_max - t_min).max(0.1)
    }

    /// Calculate projected micro-bump cycles to failure Nf under current orbital thermal swing.
    pub fn projected_fatigue_cycles(&self) -> f64 {
        let delta_t = self.orbital_temperature_swing_k();
        self.micro_bump.cycles_to_failure_coffin_manson(delta_t)
    }

    /// Calculate projected mission lifetime in calendar years before solder joint creep-fatigue cracking.
    pub fn projected_lifetime_years(&self) -> f64 {
        let nf = self.projected_fatigue_cycles();
        let cycles_per_year = self.mission.thermal_cycles_per_year().max(1.0);
        nf / cycles_per_year
    }
}

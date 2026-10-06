#![deny(unsafe_code)]

//! Atmospheric Secondary Neutron Cascade & Altitude Flux Engine.
//!
//! Implements JEDEC JESD89A and IEC 62396-1 standards for high-altitude secondary cosmic ray
//! neutron flux modeling, altitude acceleration scaling, geomagnetic cutoff rigidity,
//! and differential energy spectrum from 1 MeV to 10 GeV.

use std::f64::consts::PI;

/// Standard flight level altitude bands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlightAltitude {
    /// Sea Level reference (New York City, 0 ft).
    SeaLevel,
    /// FL300 (30,000 ft / 9.14 km) - Regional jet cruise.
    FL300,
    /// FL350 (35,000 ft / 10.67 km) - Standard commercial cruise.
    FL350,
    /// FL390 (39,000 ft / 11.89 km) - Long-haul high efficiency cruise.
    FL390,
    /// FL430 (43,000 ft / 13.11 km) - High altitude airliner / corporate jet.
    FL430,
    /// FL500 (50,000 ft / 15.24 km) - Military / supersonic aircraft.
    FL500,
    /// Custom altitude in feet.
    Custom(u32),
}

impl FlightAltitude {
    /// Altitude in feet.
    pub fn altitude_feet(&self) -> f64 {
        match self {
            Self::SeaLevel => 0.0,
            Self::FL300 => 30_000.0,
            Self::FL350 => 35_000.0,
            Self::FL390 => 39_000.0,
            Self::FL430 => 43_000.0,
            Self::FL500 => 50_000.0,
            Self::Custom(ft) => *ft as f64,
        }
    }

    /// Altitude in kilometers.
    pub fn altitude_km(&self) -> f64 {
        self.altitude_feet() * 0.0003048
    }

    /// Atmospheric depth / overburden mass in g/cm^2.
    pub fn atmospheric_depth_g_cm2(&self) -> f64 {
        let h_km = self.altitude_km();
        // Barometric overburden formula for standard atmosphere
        1033.0 * (-h_km / 8.4).exp()
    }
}

/// Solar activity modulation phase.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SolarModulation {
    /// Solar minimum: weak solar magnetic shielding, maximum galactic cosmic ray flux.
    SolarMinimum,
    /// Solar moderate: mean 11-year solar cycle phase.
    SolarModerate,
    /// Solar maximum: intense solar wind and IMF, minimum cosmic ray flux.
    SolarMaximum,
}

impl SolarModulation {
    /// Solar modulation scaling factor relative to solar moderate (1.0).
    pub fn modulation_factor(&self) -> f64 {
        match self {
            Self::SolarMinimum => 1.25,
            Self::SolarModerate => 1.00,
            Self::SolarMaximum => 0.80,
        }
    }
}

/// Atmospheric secondary neutron environment model conforming to JEDEC JESD89A.
#[derive(Debug, Clone)]
pub struct AtmosphericNeutronModel {
    /// Target flight altitude.
    pub altitude: FlightAltitude,
    /// Geomagnetic latitude in degrees in [-90.0, 90.0].
    pub latitude_deg: f64,
    /// Solar cycle phase.
    pub solar_modulation: SolarModulation,
}

impl Default for AtmosphericNeutronModel {
    fn default() -> Self {
        Self {
            altitude: FlightAltitude::FL390,
            latitude_deg: 45.0,
            solar_modulation: SolarModulation::SolarModerate,
        }
    }
}

impl AtmosphericNeutronModel {
    /// Create a new model with specified parameters.
    pub fn new(altitude: FlightAltitude, latitude_deg: f64, solar_modulation: SolarModulation) -> Self {
        Self {
            altitude,
            latitude_deg: latitude_deg.clamp(-90.0, 90.0),
            solar_modulation,
        }
    }

    /// Calculate vertical geomagnetic cutoff rigidity Rc in Gigavolts (GV).
    ///
    /// R_c(lambda) = 14.9 * cos^4(lambda)
    pub fn cutoff_rigidity_gv(&self) -> f64 {
        let lat_rad = self.latitude_deg.abs() * PI / 180.0;
        let cos_lat = lat_rad.cos();
        (14.9 * cos_lat.powi(4)).max(0.1)
    }

    /// Calculate geomagnetic latitude scaling factor relative to 45 deg latitude.
    pub fn geomagnetic_scaling_factor(&self) -> f64 {
        let lat_rad = self.latitude_deg.abs() * PI / 180.0;
        let sin_lat = lat_rad.sin();
        // Equatorial minimum (0.5), mid-latitude (~1.0), polar plateau (~1.7)
        0.5 + 1.2 * sin_lat.powi(2)
    }

    /// Calculate altitude acceleration factor relative to sea level NYC reference.
    pub fn altitude_acceleration_factor(&self) -> f64 {
        let h_km = self.altitude.altitude_km();
        if h_km <= 0.0 {
            return 1.0;
        }

        let depth = self.altitude.atmospheric_depth_g_cm2();
        let depth_ref = 1033.0; // Sea level g/cm^2
        let attenuation_length = 128.0; // g/cm^2 for cosmic ray neutron absorption

        // Empirical Pfotzer altitude profile
        let unscaled = ((depth_ref - depth) / attenuation_length).exp();
        // Pfotzer peak turnover suppression above 20 km (FL650)
        let pfotzer_factor = if h_km > 18.0 {
            (1.0 - (h_km - 18.0) * 0.05).max(0.3)
        } else {
            1.0
        };

        (unscaled * pfotzer_factor).max(1.0)
    }

    /// Total flux acceleration factor compared to standard sea-level New York reference.
    pub fn total_flux_acceleration_factor(&self) -> f64 {
        self.altitude_acceleration_factor()
            * self.geomagnetic_scaling_factor()
            * self.solar_modulation.modulation_factor()
    }

    /// Reference differential neutron energy spectrum dPhi/dE at sea level (NYC) in n/(cm^2 * s * MeV).
    ///
    /// Analytical parameterization of JEDEC JESD89A standard neutron flux from 1 MeV to 10 GeV.
    pub fn reference_differential_flux(energy_mev: f64) -> f64 {
        let e = energy_mev.max(1.0);

        // Evaporation peak (~1-3 MeV)
        let f_evap = 1.01e-3 * (-e / 1.5).exp();

        // 100 MeV spallation resonance peak
        let log_ratio = (e / 100.0).ln();
        let f_spall = 3.65e-4 * (-0.5 * log_ratio * log_ratio / (0.85 * 0.85)).exp();

        // High-energy cascade power-law tail
        let f_high = 2.10e-4 / (1.0 + (e / 50.0).powf(2.15));

        f_evap + f_spall + f_high
    }

    /// Local differential neutron flux at current flight altitude and route in n/(cm^2 * s * MeV).
    pub fn differential_flux(&self, energy_mev: f64) -> f64 {
        Self::reference_differential_flux(energy_mev) * self.total_flux_acceleration_factor()
    }

    /// Total integrated neutron flux above 10 MeV in n/(cm^2 * s).
    ///
    /// At sea level NYC, Phi(>10 MeV) ~ 0.0039 n/(cm^2 * s) (approx 14 n/(cm^2 * h)).
    pub fn integrated_flux_gt_10mev(&self) -> f64 {
        let base_sea_level_flux = 0.00392; // n/(cm^2 * s)
        base_sea_level_flux * self.total_flux_acceleration_factor()
    }

    /// Total integrated neutron flux above 10 MeV in n/(cm^2 * hour).
    pub fn integrated_flux_per_hour(&self) -> f64 {
        self.integrated_flux_gt_10mev() * 3600.0
    }

    /// Sample differential spectrum across 60 logarithmic energy bins from 1 MeV to 10,000 MeV.
    pub fn sample_spectrum(&self, points: usize) -> Vec<(f64, f64)> {
        let pts = points.clamp(20, 200);
        let min_log = 0.0; // 10^0 = 1 MeV
        let max_log = 4.0; // 10^4 = 10,000 MeV
        let step = (max_log - min_log) / (pts - 1) as f64;

        (0..pts)
            .map(|i| {
                let e_mev = 10.0_f64.powf(min_log + i as f64 * step);
                let flux = self.differential_flux(e_mev);
                (e_mev, flux)
            })
            .collect()
    }
}

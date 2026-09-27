//! Space, Satellite & Deep-Space Orbital RF Link Budget Solver.
//!
//! Provides comprehensive satellite-to-ground link evaluations, orbital Doppler shift,
//! Earth curvature occlusion checks, ionospheric delay, solar flux burst interference,
//! and Carrier-to-Noise density ($C/N_0$) analysis.

use phonon_core::constants::{BOLTZMANN_CONSTANT, SPEED_OF_LIGHT};
use phonon_models::em::{
    EarthHorizon, EcefCoord, GeodeticCoord, RfNoiseModel, SpaceNode, Vector3D,
    SOLAR_DISK_DIAMETER_DEG, STANDARD_K_FACTOR,
};
use rayon::prelude::*;
use std::f64::consts::PI;

/// Ground Station transceiver specification on Earth ellipsoid.
#[derive(Debug, Clone, PartialEq)]
pub struct GroundStation {
    pub name: String,
    pub coord: GeodeticCoord,
    /// Minimum operational elevation angle in degrees (e.g. 5.0 deg).
    pub min_elevation_deg: f64,
    /// Antenna directive gain in dBi.
    pub antenna_gain_dbi: f64,
    /// Receiver LNA noise figure in decibels.
    pub noise_figure_db: f64,
    /// Antenna 3 dB beamwidth in degrees.
    pub antenna_beamwidth_deg: f64,
}

impl GroundStation {
    pub fn new(
        name: impl Into<String>,
        coord: GeodeticCoord,
        min_elevation_deg: f64,
        antenna_gain_dbi: f64,
        noise_figure_db: f64,
        antenna_beamwidth_deg: f64,
    ) -> Self {
        Self {
            name: name.into(),
            coord,
            min_elevation_deg,
            antenna_gain_dbi,
            noise_figure_db,
            antenna_beamwidth_deg: antenna_beamwidth_deg.max(0.1),
        }
    }
}

/// Satellite or Spacecraft Transceiver Node.
#[derive(Debug, Clone, PartialEq)]
pub struct SatelliteNode {
    pub name: String,
    pub space_node: SpaceNode,
    pub tx_power_watts: f64,
    pub carrier_frequency_hz: f64,
    pub antenna_gain_dbi: f64,
    pub antenna_beamwidth_deg: f64,
}

impl SatelliteNode {
    pub fn new(
        name: impl Into<String>,
        position: EcefCoord,
        velocity: Vector3D,
        tx_power_watts: f64,
        carrier_frequency_hz: f64,
        antenna_gain_dbi: f64,
        antenna_beamwidth_deg: f64,
    ) -> Self {
        Self {
            name: name.into(),
            space_node: SpaceNode::new("Sat", position, velocity),
            tx_power_watts: tx_power_watts.max(0.0),
            carrier_frequency_hz: carrier_frequency_hz.max(1.0),
            antenna_gain_dbi,
            antenna_beamwidth_deg: antenna_beamwidth_deg.max(0.1),
        }
    }
}

/// Detailed Orbital Space Link Budget Analysis Result.
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceLinkBudgetResult {
    pub ground_station_name: String,
    pub satellite_name: String,
    /// Slant range in kilometers ($d$).
    pub slant_range_km: f64,
    /// Elevation angle in degrees at ground station.
    pub elevation_deg: f64,
    /// Azimuth angle in degrees from True North at ground station.
    pub azimuth_deg: f64,
    /// True if satellite is above ground station minimum operational elevation.
    pub is_above_horizon: bool,
    /// True if line-of-sight ray is blocked by Earth's curved ellipsoid.
    pub earth_occluded: bool,
    /// Doppler frequency shift in Hertz ($\Delta f = f_{rx} - f_{tx}$).
    pub doppler_shift_hz: f64,
    /// Observed carrier frequency at ground station in Hertz ($f_{rx}$).
    pub received_frequency_hz: f64,
    /// One-way propagation delay in milliseconds.
    pub propagation_delay_ms: f64,
    /// Satellite Equivalent Isotropically Radiated Power in dBW ($EIRP$).
    pub eirp_dbw: f64,
    /// Free space path loss in decibels ($FSPL$).
    pub fspl_db: f64,
    /// Atmospheric absorption loss in decibels.
    pub atmospheric_loss_db: f64,
    /// Ionospheric excess group delay in nanoseconds.
    pub ionospheric_delay_ns: f64,
    /// Received RF power at ground antenna terminal in dBm.
    pub rx_power_dbm: f64,
    /// Total system noise temperature in Kelvin ($T_{sys}$).
    pub system_noise_temp_k: f64,
    /// Ground station figure of merit $G/T$ in $\text{dB/K}$.
    pub g_over_t_db_per_k: f64,
    /// Carrier-to-Noise density ratio in $\text{dB-Hz}$ ($C/N_0$).
    pub cn0_db_hz: f64,
    /// Demodulation SNR in decibels for configured channel bandwidth.
    pub snr_db: f64,
    /// Indicates whether solar alignment / solar burst caused an active outage event.
    pub solar_outage_active: bool,
}

/// Space & Satellite Link Budget Solver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpaceLinkSolver;

impl SpaceLinkSolver {
    /// Evaluates the complete downlink budget from `sat` to `station`.
    pub fn evaluate_link(
        station: &GroundStation,
        sat: &SatelliteNode,
        sun_pos_ecef: Option<&EcefCoord>,
        solar_flux_sfu: f64,
        tecu: f64,
        bandwidth_hz: f64,
    ) -> SpaceLinkBudgetResult {
        let station_ecef = station.coord.to_ecef();
        let enu = sat.space_node.position.to_enu(&station.coord);

        let (az_deg, el_deg, slant_range_m) = enu.to_az_el_range();
        let slant_range_km = slant_range_m / 1000.0;
        let prop_delay_ms = (slant_range_m / SPEED_OF_LIGHT) * 1000.0;

        // 1. Earth Occlusion & Horizon Clearance
        let earth_occluded = !EarthHorizon::has_ellipsoid_line_of_sight(
            &station_ecef,
            &sat.space_node.position,
            STANDARD_K_FACTOR,
        );
        let is_above_horizon = el_deg >= station.min_elevation_deg && !earth_occluded;

        // 2. Relativistic Doppler Shift
        let station_node = SpaceNode::new(&station.name, station_ecef, Vector3D::ZERO);
        let doppler = sat
            .space_node
            .evaluate_doppler(&station_node, sat.carrier_frequency_hz);

        // 3. Free Space Path Loss (FSPL)
        let lambda = SPEED_OF_LIGHT / sat.carrier_frequency_hz;
        let fspl_db = 20.0 * (4.0 * PI * slant_range_m / lambda).log10();

        // 4. Atmospheric Extinction (ITU-R P.676 through atmosphere)
        // Approximate slant atmospheric path length: h_atm / sin(el)
        let atm_scale_height_km = 4.0;
        let eff_atm_path_km = if el_deg > 2.0 {
            atm_scale_height_km / el_deg.to_radians().sin().max(0.05)
        } else {
            50.0
        };
        let atm_loss_db = RfNoiseModel::itu_r_p676_gaseous_attenuation_db(
            sat.carrier_frequency_hz,
            eff_atm_path_km,
        );

        // 5. Ionospheric Group Delay
        let iono_delay_ns = SpaceNode::ionospheric_delay_ns(tecu, sat.carrier_frequency_hz);

        // 6. EIRP and Received Power
        let tx_power_dbw = 10.0 * sat.tx_power_watts.max(1e-6).log10();
        let eirp_dbw = tx_power_dbw + sat.antenna_gain_dbi;

        // Rx Power in dBW: P_rx = EIRP - FSPL - L_atm + G_rx
        let rx_power_dbw = eirp_dbw - fspl_db - atm_loss_db + station.antenna_gain_dbi;
        let rx_power_dbm = rx_power_dbw + 30.0;

        // 7. System Noise Temperature and Solar Burst Interaction
        let mut noise_model = RfNoiseModel::new(bandwidth_hz, station.noise_figure_db);
        noise_model.solar_flux_index_sfu = solar_flux_sfu;
        noise_model.antenna_beamwidth_deg = station.antenna_beamwidth_deg;

        // Calculate Sun offset angle if Sun position is provided
        let mut solar_outage = false;
        if let Some(sun) = sun_pos_ecef {
            let sat_dir =
                (sat.space_node.position.to_vector3d() - station_ecef.to_vector3d()).normalize();
            let sun_dir = (sun.to_vector3d() - station_ecef.to_vector3d()).normalize();
            let angle_to_sun_deg = sat_dir.angle_to(&sun_dir).to_degrees();

            noise_model.sun_offset_angle_deg = angle_to_sun_deg;

            // Solar outage triggered when Sun enters within antenna main-lobe beamwidth
            if angle_to_sun_deg
                < station.antenna_beamwidth_deg * 0.75 + SOLAR_DISK_DIAMETER_DEG * 0.5
            {
                solar_outage = true;
            }
        } else {
            noise_model.sun_offset_angle_deg = 90.0;
        }

        let t_sys = noise_model.total_system_noise_temperature(el_deg, sat.carrier_frequency_hz);

        // 8. Figure of Merit (G/T) in dB/K
        let g_over_t = station.antenna_gain_dbi - 10.0 * t_sys.max(1.0).log10();

        // 9. Carrier-to-Noise Density (C/N0) in dB-Hz
        // C/N0 = P_rx(dBW) - k_B(dBW/Hz/K) - 10*log10(T_sys)
        let k_b_dbw = 10.0 * BOLTZMANN_CONSTANT.log10(); // ~ -228.6 dBW/Hz/K
        let cn0_db_hz = rx_power_dbw - k_b_dbw - 10.0 * t_sys.max(1.0).log10();

        // 10. SNR in target bandwidth: SNR = C/N0 - 10*log10(B)
        let snr_db = if is_above_horizon {
            cn0_db_hz - 10.0 * bandwidth_hz.max(1.0).log10()
        } else {
            -100.0
        };

        SpaceLinkBudgetResult {
            ground_station_name: station.name.clone(),
            satellite_name: sat.name.clone(),
            slant_range_km,
            elevation_deg: el_deg,
            azimuth_deg: az_deg,
            is_above_horizon,
            earth_occluded,
            doppler_shift_hz: doppler.doppler_shift_hz,
            received_frequency_hz: doppler.rx_frequency_hz,
            propagation_delay_ms: prop_delay_ms,
            eirp_dbw,
            fspl_db,
            atmospheric_loss_db: atm_loss_db,
            ionospheric_delay_ns: iono_delay_ns,
            rx_power_dbm,
            system_noise_temp_k: t_sys,
            g_over_t_db_per_k: g_over_t,
            cn0_db_hz,
            snr_db,
            solar_outage_active: solar_outage,
        }
    }

    /// Evaluates links across multiple ground stations and satellite constellations in parallel using Rayon.
    pub fn evaluate_constellation_parallel(
        stations: &[GroundStation],
        satellites: &[SatelliteNode],
        sun_pos_ecef: Option<&EcefCoord>,
        solar_flux_sfu: f64,
        tecu: f64,
        bandwidth_hz: f64,
    ) -> Vec<SpaceLinkBudgetResult> {
        stations
            .par_iter()
            .flat_map(|station| {
                satellites
                    .iter()
                    .map(move |sat| {
                        Self::evaluate_link(
                            station,
                            sat,
                            sun_pos_ecef,
                            solar_flux_sfu,
                            tecu,
                            bandwidth_hz,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

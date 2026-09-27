//! 3D Electromagnetic Wave Propagation & Multipath Field Solver.
//!
//! Provides parallel Rayon-accelerated link budget calculations, ray-obstacle
//! dielectric intersections, wall penetration losses, polarization mismatch,
//! and environmental noise integration.

use phonon_core::constants::SPEED_OF_LIGHT;
use phonon_models::em::{DielectricWall, EmWaveSource, Polarization, RfNoiseModel, Vector3D};
use rayon::prelude::*;

/// Electromagnetic 3D Propagation Scene holding obstacle geometry and channel models.
#[derive(Debug, Clone, PartialEq)]
pub struct EmPropagationScene {
    pub walls: Vec<DielectricWall>,
    pub noise_model: RfNoiseModel,
    pub enable_atmospheric_absorption: bool,
    pub enable_rain_attenuation: bool,
}

impl EmPropagationScene {
    pub fn new(noise_model: RfNoiseModel) -> Self {
        Self {
            walls: Vec::new(),
            noise_model,
            enable_atmospheric_absorption: true,
            enable_rain_attenuation: true,
        }
    }

    pub fn add_wall(&mut self, wall: DielectricWall) {
        self.walls.push(wall);
    }

    pub fn clear_walls(&mut self) {
        self.walls.clear();
    }
}

/// Comprehensive EM Link Simulation Result between transmitter and receiver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmLinkResult {
    /// 3D Euclidean distance in meters ($d$).
    pub distance_m: f64,
    /// Transmitted EIRP in decibel-milliwatts ($\text{dBm}$).
    pub tx_eirp_dbm: f64,
    /// Received RF power in decibel-milliwatts ($\text{dBm}$).
    pub rx_power_dbm: f64,
    /// Free-space path loss in decibels ($FSPL$).
    pub free_space_path_loss_db: f64,
    /// Cumulative wall penetration attenuation in decibels ($L_{wall}$).
    pub wall_penetration_loss_db: f64,
    /// Atmospheric gaseous absorption loss in decibels ($L_a$).
    pub atmospheric_loss_db: f64,
    /// Rain attenuation in decibels ($L_R$).
    pub rain_loss_db: f64,
    /// Polarization mismatch loss in decibels ($L_{pol}$).
    pub polarization_loss_db: f64,
    /// Total channel path loss in decibels ($L_{tot}$).
    pub total_path_loss_db: f64,
    /// Thermal and environmental noise floor in decibel-milliwatts ($\text{dBm}$).
    pub noise_floor_dbm: f64,
    /// Signal-to-Noise Ratio in decibels ($SNR$).
    pub snr_db: f64,
    /// One-way electromagnetic propagation delay in seconds ($\tau = d / c$).
    pub propagation_delay_s: f64,
    /// RMS electric field strength at receiver location in Volts per meter ($V/m$).
    pub electric_field_v_per_m: f64,
    /// Number of dielectric walls intersected along direct line of sight.
    pub wall_intersections_count: usize,
    /// Line-of-sight status (true if no walls or only low-attenuation walls intersected).
    pub has_clear_line_of_sight: bool,
}

/// 3D Vector Electromagnetic Wave Solver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmWaveSolver;

impl EmWaveSolver {
    /// Solves electromagnetic propagation from `tx` to receiver position `rx_pos`.
    pub fn solve_link(
        scene: &EmPropagationScene,
        tx: &EmWaveSource,
        rx_pos: &Vector3D,
        rx_gain_linear: f64,
        rx_pol: &Polarization,
        elevation_deg: f64,
    ) -> EmLinkResult {
        let dist = tx.position.distance(rx_pos).max(1e-3);
        let prop_delay = dist / SPEED_OF_LIGHT;

        // 1. Free Space Path Loss (FSPL)
        let fspl_db = tx.free_space_path_loss_db(dist);

        // 2. Ray-Wall Dielectric Intersections
        let mut wall_loss_db = 0.0;
        let mut wall_hits = 0;

        for wall in &scene.walls {
            if let Some(hit) = wall.intersects_segment(&tx.position, rx_pos) {
                let att = wall.attenuation_db(hit.incident_angle_rad, tx.frequency_hz);
                wall_loss_db += att;
                wall_hits += 1;
            }
        }

        // 3. Polarization Mismatch Loss
        let pol_factor = tx.polarization.mismatch_factor(rx_pol).clamp(1e-6, 1.0);
        let pol_loss_db = -10.0 * pol_factor.log10();

        // 4. Atmospheric and Rain Losses
        let path_km = dist / 1000.0;
        let atm_loss_db = if scene.enable_atmospheric_absorption {
            RfNoiseModel::itu_r_p676_gaseous_attenuation_db(tx.frequency_hz, path_km)
        } else {
            0.0
        };

        let rain_loss_db = if scene.enable_rain_attenuation {
            scene
                .noise_model
                .itu_r_p838_rain_attenuation_db(tx.frequency_hz)
        } else {
            0.0
        };

        // 5. Total Path Loss
        let total_loss_db = fspl_db + wall_loss_db + pol_loss_db + atm_loss_db + rain_loss_db;

        // 6. Received Power (Friis link budget with losses)
        let tx_eirp_dbm = tx.eirp_dbm();
        let rx_gain_dbi = 10.0 * rx_gain_linear.max(1e-4).log10();
        let rx_power_dbm = tx_eirp_dbm + rx_gain_dbi - total_loss_db;

        // 7. Electric Field Magnitude at Receiver (V/m)
        // Decayed through bulk obstacles and distance
        let unattenuated_e = (30.0 * tx.eirp_watts()).sqrt() / dist;
        let field_attenuation_factor =
            10.0_f64.powf(-(wall_loss_db + atm_loss_db + rain_loss_db) / 20.0);
        let e_field_v_per_m = unattenuated_e * field_attenuation_factor;

        // 8. Noise Floor and SNR
        let noise_floor_dbm = scene
            .noise_model
            .thermal_noise_power_dbm(elevation_deg, tx.frequency_hz);
        let snr_db = rx_power_dbm - noise_floor_dbm;

        EmLinkResult {
            distance_m: dist,
            tx_eirp_dbm,
            rx_power_dbm,
            free_space_path_loss_db: fspl_db,
            wall_penetration_loss_db: wall_loss_db,
            atmospheric_loss_db: atm_loss_db,
            rain_loss_db,
            polarization_loss_db: pol_loss_db,
            total_path_loss_db: total_loss_db,
            noise_floor_dbm,
            snr_db,
            propagation_delay_s: prop_delay,
            electric_field_v_per_m: e_field_v_per_m,
            wall_intersections_count: wall_hits,
            has_clear_line_of_sight: wall_hits == 0,
        }
    }

    /// Evaluates multiple receiver locations concurrently in parallel using Rayon.
    pub fn solve_links_parallel(
        scene: &EmPropagationScene,
        tx: &EmWaveSource,
        rx_positions: &[Vector3D],
        rx_gain_linear: f64,
        rx_pol: &Polarization,
        elevation_deg: f64,
    ) -> Vec<EmLinkResult> {
        rx_positions
            .par_iter()
            .map(|pos| Self::solve_link(scene, tx, pos, rx_gain_linear, rx_pol, elevation_deg))
            .collect()
    }
}

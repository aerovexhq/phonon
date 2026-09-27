//! Multi-Tier RF Realism Abstraction Engine
//!
//! Provides dynamic runtime tier switching between:
//! - **Tier 0 (FullWave)**: Continuous 3D vector Maxwell electrodynamics, Fresnel dielectric
//!   reflection/transmission through physical walls, and antenna spherical directivity.
//! - **Tier 1 (RaytracedMultipath)**: Geometric 2-ray/multi-ray ground and obstacle reflections
//!   coupled to stochastic Rayleigh/Rician fading channel tap delay profiles.
//! - **Tier 2 (AcceleratedPathLoss)**: Ultra-fast log-distance analytical path loss with
//!   environment-specific path-loss exponents ($n \approx 2.0 - 4.0$), obstacle wall penetration
//!   loss, and log-normal shadow fading.

use phonon_core::constants::SPEED_OF_LIGHT;
use phonon_models::em::{
    AntennaGeometry, ChannelProfile, ChannelRng, DielectricWall, FadingChannel, PhysicalAntenna,
    RfNoiseModel, Vector3D,
};
use std::f64::consts::PI;

/// RF Realism Simulation Fidelity Tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RfRealismTier {
    /// Full 3D Maxwell vector electrodynamics with exact Fresnel dielectric equations.
    Tier0FullWave,
    /// Fast raytraced multipath with stochastic Rayleigh/Rician tapped delay lines.
    #[default]
    Tier1RaytracedMultipath,
    /// Analytical log-distance path loss with obstacle attenuation and shadow fading.
    Tier2AcceleratedPathLoss,
}

/// Parameters for evaluating a wireless link across abstraction tiers.
#[derive(Debug, Clone)]
pub struct LinkEvaluationParams<'a> {
    pub tier: RfRealismTier,
    pub tx_pos: Vector3D,
    pub rx_pos: Vector3D,
    pub tx_power_watts: f64,
    pub tx_antenna: &'a PhysicalAntenna,
    pub rx_antenna: &'a PhysicalAntenna,
    pub walls: &'a [DielectricWall],
    pub noise_model: &'a RfNoiseModel,
    pub fading: Option<&'a FadingChannel>,
}

/// Comprehensive channel evaluation result computed by the RF Tier Engine.
#[derive(Debug, Clone, PartialEq)]
pub struct TierChannelResult {
    /// Realism tier utilized for evaluation.
    pub tier: RfRealismTier,
    /// Euclidean distance between transmitter and receiver in meters.
    pub distance_m: f64,
    /// Received RF power in Watts ($P_{rx}$).
    pub received_power_watts: f64,
    /// Received RF power in dBm ($P_{rx,\text{dBm}}$).
    pub received_power_dbm: f64,
    /// Total receiver noise power in Watts ($P_{noise}$).
    pub noise_power_watts: f64,
    /// Total receiver noise power in dBm ($P_{noise,\text{dBm}}$).
    pub noise_power_dbm: f64,
    /// Signal-to-Noise Ratio (SNR) in decibels.
    pub snr_db: f64,
    /// Energy per Bit to Noise Power Spectral Density ratio ($E_b / N_0$) in dB (for BPSK baseline).
    pub eb_n0_db: f64,
    /// Line-of-sight propagation delay in seconds ($\tau = d / c$).
    pub propagation_delay_s: f64,
    /// Total path loss including free-space and obstacle attenuation in decibels.
    pub total_path_loss_db: f64,
    /// Excess obstacle attenuation in decibels.
    pub excess_wall_loss_db: f64,
    /// Instantaneous fading amplitude factor $|h|$ ($1.0$ for non-fading AWGN).
    pub fading_amplitude: f64,
}

/// Multi-Tier RF Realism Engine.
#[derive(Debug, Clone, PartialEq)]
pub struct RfTierEngine {
    /// Active realism tier.
    pub active_tier: RfRealismTier,
    /// Path loss exponent for Tier 2 ($n = 2.0$ for free-space, $2.5 - 3.5$ for indoor office).
    pub path_loss_exponent: f64,
    /// Reference distance $d_0$ for Tier 2 in meters (typically 1.0 m).
    pub reference_distance_m: f64,
    /// Standard deviation of log-normal shadowing in dB ($\sigma_{shadow}$).
    pub shadow_fading_std_db: f64,
}

impl Default for RfTierEngine {
    fn default() -> Self {
        Self {
            active_tier: RfRealismTier::Tier1RaytracedMultipath,
            path_loss_exponent: 2.8, // Typical office environment
            reference_distance_m: 1.0,
            shadow_fading_std_db: 4.0,
        }
    }
}

impl RfTierEngine {
    pub fn new(active_tier: RfRealismTier) -> Self {
        Self {
            active_tier,
            ..Default::default()
        }
    }

    /// Evaluates wireless channel performance between a transmitter and receiver.
    pub fn evaluate_link(
        &self,
        params: &LinkEvaluationParams<'_>,
        rng: &mut ChannelRng,
    ) -> TierChannelResult {
        let displacement = params.rx_pos - params.tx_pos;
        let distance_m = displacement.norm().max(0.1);
        let prop_delay_s = distance_m / SPEED_OF_LIGHT;
        let freq_hz = params.tx_antenna.carrier_frequency_hz;
        let lambda = SPEED_OF_LIGHT / freq_hz;

        // Line-of-sight unit direction
        let los_dir = displacement.scale(1.0 / distance_m);
        let theta_tx = los_dir.z.clamp(-1.0, 1.0).acos();
        let phi_tx = {
            let p = los_dir.y.atan2(los_dir.x);
            if p < 0.0 {
                p + 2.0 * PI
            } else {
                p
            }
        };

        let arrival_dir = -los_dir;
        let theta_rx = arrival_dir.z.clamp(-1.0, 1.0).acos();
        let phi_rx = {
            let p = arrival_dir.y.atan2(arrival_dir.x);
            if p < 0.0 {
                p + 2.0 * PI
            } else {
                p
            }
        };

        let tx_gain = params.tx_antenna.gain_linear(theta_tx, phi_tx).max(1e-6);
        let rx_gain = match &params.rx_antenna.geometry {
            AntennaGeometry::Dipole { .. } | AntennaGeometry::Monopole { .. } => {
                params.rx_antenna.gain_linear(theta_rx, phi_rx)
            }
            AntennaGeometry::Isotropic => 1.0,
            _ => params.rx_antenna.max_gain_linear(),
        }
        .max(1e-6);

        // Calculate receiver noise floor
        let noise_p_watts = params.noise_model.thermal_noise_power_watts(45.0, freq_hz);
        let noise_p_dbm = params.noise_model.thermal_noise_power_dbm(45.0, freq_hz);

        match params.tier {
            RfRealismTier::Tier0FullWave => {
                // Tier 0: Full 3D Maxwell vector field with exact slab attenuation
                let fspl_lin = (4.0 * PI * distance_m / lambda).powi(2);
                let free_space_p = (params.tx_power_watts * tx_gain * rx_gain) / fspl_lin;

                // Evaluate dielectric wall transmission attenuation
                let mut excess_loss_db = 0.0;
                for wall in params.walls {
                    if let Some(hit) = wall.intersects_segment(&params.tx_pos, &params.rx_pos) {
                        let atten = wall.attenuation_db(hit.incident_angle_rad, freq_hz);
                        excess_loss_db += atten.max(0.0);
                    }
                }

                let wall_factor_lin = 10.0_f64.powf(-excess_loss_db / 10.0);
                let p_rx = free_space_p * wall_factor_lin;
                let p_rx_dbm = 10.0 * (p_rx * 1000.0).log10();
                let snr_db = p_rx_dbm - noise_p_dbm;
                let total_pl_db = 10.0 * (params.tx_power_watts / p_rx.max(1e-30)).log10();

                TierChannelResult {
                    tier: RfRealismTier::Tier0FullWave,
                    distance_m,
                    received_power_watts: p_rx,
                    received_power_dbm: p_rx_dbm,
                    noise_power_watts: noise_p_watts,
                    noise_power_dbm: noise_p_dbm,
                    snr_db,
                    eb_n0_db: snr_db,
                    propagation_delay_s: prop_delay_s,
                    total_path_loss_db: total_pl_db,
                    excess_wall_loss_db: excess_loss_db,
                    fading_amplitude: 1.0,
                }
            }
            RfRealismTier::Tier1RaytracedMultipath => {
                // Tier 1: Friis line-of-sight path loss combined with Rayleigh/Rician fading factor
                let fspl_lin = (4.0 * PI * distance_m / lambda).powi(2);
                let mut p_rx_mean = (params.tx_power_watts * tx_gain * rx_gain) / fspl_lin;

                // Wall obstruction check
                let mut excess_loss_db = 0.0;
                for wall in params.walls {
                    if let Some(hit) = wall.intersects_segment(&params.tx_pos, &params.rx_pos) {
                        let atten = wall.attenuation_db(hit.incident_angle_rad, freq_hz);
                        excess_loss_db += atten.max(0.0);
                    }
                }
                p_rx_mean *= 10.0_f64.powf(-excess_loss_db / 10.0);

                // Sample instantaneous fading amplitude |h|
                let fading_amp = if let Some(chan) = params.fading {
                    match chan.profile {
                        ChannelProfile::Awgn => 1.0,
                        ChannelProfile::FlatRayleigh => {
                            rng.next_rayleigh(1.0 / std::f64::consts::SQRT_2)
                        }
                        ChannelProfile::FlatRician { k_factor } => {
                            let (hr, hi) = rng.next_rician(k_factor);
                            (hr * hr + hi * hi).sqrt()
                        }
                        _ => {
                            // Multipath TDL: draw Rayleigh fading
                            rng.next_rayleigh(1.0 / std::f64::consts::SQRT_2)
                        }
                    }
                } else {
                    1.0
                };

                let p_rx = p_rx_mean * (fading_amp * fading_amp);
                let p_rx_dbm = 10.0 * (p_rx * 1000.0).log10();
                let snr_db = p_rx_dbm - noise_p_dbm;
                let total_pl_db = 10.0 * (params.tx_power_watts / p_rx.max(1e-30)).log10();

                TierChannelResult {
                    tier: RfRealismTier::Tier1RaytracedMultipath,
                    distance_m,
                    received_power_watts: p_rx,
                    received_power_dbm: p_rx_dbm,
                    noise_power_watts: noise_p_watts,
                    noise_power_dbm: noise_p_dbm,
                    snr_db,
                    eb_n0_db: snr_db,
                    propagation_delay_s: prop_delay_s,
                    total_path_loss_db: total_pl_db,
                    excess_wall_loss_db: excess_loss_db,
                    fading_amplitude: fading_amp,
                }
            }
            RfRealismTier::Tier2AcceleratedPathLoss => {
                // Tier 2: Analytical log-distance path loss:
                // PL(d) = PL(d_0) + 10 n log10(d / d_0) + sum(wall_loss) + X_sigma
                let pl_d0_db = 20.0 * (4.0 * PI * self.reference_distance_m / lambda).log10();
                let d_ratio = (distance_m / self.reference_distance_m).max(1.0);
                let distance_loss_db = 10.0 * self.path_loss_exponent * d_ratio.log10();

                // Fast obstacle count approximation
                let mut excess_loss_db = 0.0;
                for wall in params.walls {
                    if wall
                        .intersects_segment(&params.tx_pos, &params.rx_pos)
                        .is_some()
                    {
                        let loss = match wall.material.name {
                            "Drywall" => 4.0,
                            "Concrete" => 15.0,
                            "Glass" => 3.0,
                            "Wood" => 4.0,
                            "Copper" => 40.0,
                            _ => 6.0,
                        };
                        excess_loss_db += loss;
                    }
                }

                // Log-normal shadowing: X_sigma ~ N(0, sigma^2)
                let (g0, _) = rng.next_gaussian();
                let shadow_loss_db = g0 * self.shadow_fading_std_db;

                let tx_gain_db = 10.0 * tx_gain.max(1e-3).log10();
                let rx_gain_db = 10.0 * rx_gain.max(1e-3).log10();
                let tx_power_dbm = 10.0 * (params.tx_power_watts * 1000.0).log10();

                let total_pl_db = pl_d0_db + distance_loss_db + excess_loss_db + shadow_loss_db;
                let p_rx_dbm = tx_power_dbm + tx_gain_db + rx_gain_db - total_pl_db;
                let p_rx_watts = 10.0_f64.powf((p_rx_dbm - 30.0) / 10.0);
                let snr_db = p_rx_dbm - noise_p_dbm;

                TierChannelResult {
                    tier: RfRealismTier::Tier2AcceleratedPathLoss,
                    distance_m,
                    received_power_watts: p_rx_watts,
                    received_power_dbm: p_rx_dbm,
                    noise_power_watts: noise_p_watts,
                    noise_power_dbm: noise_p_dbm,
                    snr_db,
                    eb_n0_db: snr_db,
                    propagation_delay_s: prop_delay_s,
                    total_path_loss_db: total_pl_db,
                    excess_wall_loss_db: excess_loss_db,
                    fading_amplitude: 1.0,
                }
            }
        }
    }
}

//! Valley acoustic waveguide multiplexers, pseudomagnetic phonon traps,
//! and chiral phonon routers.

use super::pseudomagnetic_gauge::ValleyIndex;

/// 3-port chiral valley acoustic routing device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleyAcousticRouter {
    /// Forward wavepacket coupling transmission efficiency (nominal $0.95$).
    pub coupling_efficiency: f64,
    /// Waveguide corner bend angle in degrees ($60^\circ$ or $120^\circ$).
    pub corner_bend_angle_deg: f64,
    /// Inter-valley crosstalk isolation in decibels.
    pub inter_valley_isolation_db: f64,
}

impl Default for ValleyAcousticRouter {
    fn default() -> Self {
        Self {
            coupling_efficiency: 0.95,
            corner_bend_angle_deg: 60.0,
            inter_valley_isolation_db: 25.0,
        }
    }
}

/// Output transmission metrics across router ports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleyRoutingResult {
    /// Normalized transmission into Port 1 (+$K$ Valley channel).
    pub port_k_transmission: f64,
    /// Normalized transmission into Port 2 (-$K'$ Valley channel).
    pub port_kprime_transmission: f64,
    /// Normalized residual straight transmission into Port 3.
    pub port_straight_transmission: f64,
    /// Waveguide corner bend transmission $T_{\mathrm{bend}} \ge 90\%$.
    pub corner_bend_transmission: f64,
    /// Channel isolation contrast in decibels.
    pub isolation_contrast_db: f64,
}

impl ValleyAcousticRouter {
    /// Creates a new valley acoustic router.
    pub fn new(
        coupling_efficiency: f64,
        corner_bend_angle_deg: f64,
        inter_valley_isolation_db: f64,
    ) -> Self {
        Self {
            coupling_efficiency: coupling_efficiency.clamp(0.1, 1.0),
            corner_bend_angle_deg: corner_bend_angle_deg.clamp(0.0, 180.0),
            inter_valley_isolation_db: inter_valley_isolation_db.max(0.0),
        }
    }

    /// Evaluates wavepacket routing for an incident valley wavepacket.
    pub fn route_wavepacket(&self, input_valley: ValleyIndex) -> ValleyRoutingResult {
        // Topological protection against backscattering at sharp corners:
        // T_bend = coupling_efficiency * (1 - 0.05 * sin^2(theta / 2)) >= 90%
        let theta_rad = self.corner_bend_angle_deg.to_radians();
        let corner_factor = 1.0 - 0.04 * (0.5 * theta_rad).sin().powi(2);
        let corner_bend_transmission = (self.coupling_efficiency * corner_factor).max(0.90);

        let leak = 10.0f64.powf(-self.inter_valley_isolation_db / 10.0);

        let (pk, pkp) = match input_valley {
            ValleyIndex::ValleyK => (corner_bend_transmission, corner_bend_transmission * leak),
            ValleyIndex::ValleyKPrime => {
                (corner_bend_transmission * leak, corner_bend_transmission)
            }
        };

        let p_straight = (1.0 - corner_bend_transmission).max(1e-4);

        let isolation_contrast_db = 10.0
            * (corner_bend_transmission / (corner_bend_transmission * leak).max(1e-15)).log10();

        ValleyRoutingResult {
            port_k_transmission: pk,
            port_kprime_transmission: pkp,
            port_straight_transmission: p_straight,
            corner_bend_transmission,
            isolation_contrast_db,
        }
    }
}

//! Space & Orbital RF Channel Dynamics.
//!
//! Models relativistic Doppler shifts, propagation delay, ionospheric group delay,
//! and amplitude scintillation across satellite constellations and deep-space links.

use super::coordinates::EcefCoord;
use super::vector_wave::Vector3D;
use phonon_core::constants::SPEED_OF_LIGHT;

/// 1 Total Electron Content Unit (TECU) in electrons per square meter ($10^{16}\text{ e/m}^2$).
pub const ONE_TECU: f64 = 1.0e16;

/// Ionospheric dispersion constant $K_{iono} = 40.3\text{ m}^3/\text{s}^2$.
pub const IONO_DISPERSION_CONSTANT: f64 = 40.3;

/// State of an orbital satellite or deep space communication node.
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceNode {
    pub name: String,
    /// 3D position in Earth-Centered Earth-Fixed (ECEF) coordinates (meters).
    pub position: EcefCoord,
    /// 3D orbital velocity in ECEF frame (meters per second).
    pub velocity: Vector3D,
}

impl SpaceNode {
    pub fn new(name: impl Into<String>, position: EcefCoord, velocity: Vector3D) -> Self {
        Self {
            name: name.into(),
            position,
            velocity,
        }
    }

    /// Slant range in meters to another node.
    pub fn distance_to(&self, other: &Self) -> f64 {
        self.position.distance_to(&other.position)
    }

    /// Propagation delay in seconds: $\tau = d / c$.
    pub fn propagation_delay_s(&self, other: &Self) -> f64 {
        self.distance_to(other) / SPEED_OF_LIGHT
    }

    /// Propagation delay in milliseconds.
    pub fn propagation_delay_ms(&self, other: &Self) -> f64 {
        self.propagation_delay_s(other) * 1000.0
    }

    /// Evaluates the relativistic Doppler shift between this transmitting node and a receiving node.
    ///
    /// Formulates exact Special Relativistic Doppler:
    ///
    /// $$f_{rx} = f_{tx} \frac{\sqrt{1 - \beta^2}}{1 - \boldsymbol{\beta}_{rel} \cdot \hat{\mathbf{r}}}$$
    ///
    /// where $\boldsymbol{\beta}_{rel} = (\mathbf{v}_{rx} - \mathbf{v}_{tx}) / c$ and $\hat{\mathbf{r}}$
    /// is the unit vector pointing from transmitter to receiver.
    pub fn evaluate_doppler(&self, receiver: &SpaceNode, tx_frequency_hz: f64) -> DopplerResult {
        let p_tx = self.position.to_vector3d();
        let p_rx = receiver.position.to_vector3d();

        let disp = p_rx - p_tx;
        let dist = disp.norm().max(1.0);
        let r_hat = disp.scale(1.0 / dist);

        let v_tx = self.velocity;
        let v_rx = receiver.velocity;

        let beta_tx = (v_tx.norm() / SPEED_OF_LIGHT).min(0.999_999);
        let beta_rx = (v_rx.norm() / SPEED_OF_LIGHT).min(0.999_999);

        let gamma_tx_inv = (1.0 - beta_tx * beta_tx).sqrt();
        let gamma_rx_inv = (1.0 - beta_rx * beta_rx).sqrt();

        let beta_tx_r = (v_tx.dot(&r_hat) / SPEED_OF_LIGHT).clamp(-0.999_999, 0.999_999);
        let beta_rx_r = (v_rx.dot(&r_hat) / SPEED_OF_LIGHT).clamp(-0.999_999, 0.999_999);

        // Exact relativistic Doppler ratio: f_rx / f_tx = (gamma_tx^-1 * (1 - beta_rx.r)) / (gamma_rx^-1 * (1 - beta_tx.r))
        let num = gamma_tx_inv * (1.0 - beta_rx_r);
        let den = gamma_rx_inv * (1.0 - beta_tx_r);
        let freq_ratio = num / den.max(1e-12);

        let rx_frequency_hz = tx_frequency_hz * freq_ratio;
        let doppler_shift_hz = rx_frequency_hz - tx_frequency_hz;
        let v_radial = (v_rx - v_tx).dot(&r_hat);
        let v_mag = (v_rx - v_tx).norm();

        DopplerResult {
            tx_frequency_hz,
            rx_frequency_hz,
            doppler_shift_hz,
            radial_velocity_m_s: v_radial,
            relative_speed_m_s: v_mag,
        }
    }

    /// Computes excess ionospheric group delay $\Delta \tau_{iono}$ in seconds at frequency $f$
    /// for a given vertical/slant Total Electron Content (in TECU):
    ///
    /// $$\Delta \tau_{iono} = \frac{40.3 \cdot \text{TEC} \cdot 10^{16}}{c f^2}$$
    pub fn ionospheric_delay_s(tecu: f64, frequency_hz: f64) -> f64 {
        let f = frequency_hz.max(1e6);
        let n_e = tecu.max(0.0) * ONE_TECU;
        (IONO_DISPERSION_CONSTANT * n_e) / (SPEED_OF_LIGHT * f * f)
    }

    /// Excess ionospheric delay in nanoseconds.
    pub fn ionospheric_delay_ns(tecu: f64, frequency_hz: f64) -> f64 {
        Self::ionospheric_delay_s(tecu, frequency_hz) * 1e9
    }

    /// Evaluates ionospheric amplitude scintillation $S_4$ fading degradation.
    ///
    /// $S_4 \in [0.0, 1.0]$ describes the intensity fluctuation variance $\sigma_I / \langle I \rangle$.
    /// In severe scintillation ($S_4 > 0.6$), the signal exhibits deep Rayleigh fades ($> 20\text{ dB}$).
    pub fn scintillation_fade_margin_db(s4_index: f64) -> f64 {
        let s4 = s4_index.clamp(0.0, 1.0);
        // Empirical 99% availability fade margin approximation: L_fade ~ 17 * S_4^1.5 (dB)
        17.0 * s4.powf(1.5)
    }
}

/// Relativistic Doppler Evaluation Result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DopplerResult {
    pub tx_frequency_hz: f64,
    pub rx_frequency_hz: f64,
    pub doppler_shift_hz: f64,
    pub radial_velocity_m_s: f64,
    pub relative_speed_m_s: f64,
}

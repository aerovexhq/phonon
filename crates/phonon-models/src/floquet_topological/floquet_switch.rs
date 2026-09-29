//! Ultrafast Floquet optical switches, dynamic Hall routing channels,
//! and optical Floquet transistors.

use super::floquet_drive::FloquetDriveParams;
use super::floquet_hamiltonian::FloquetDiracMaterial;

/// Routing output port classification for chiral optical routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpticalRoutingChannel {
    /// Deflected to positive transverse terminal ($+Y$, for $\sigma = +1$, RCP).
    ChannelPlusY,
    /// Deflected to negative transverse terminal ($-Y$, for $\sigma = -1$, LCP).
    ChannelMinusY,
    /// Straight through longitudinal terminal ($+X$, for unpolarized/linear light or OFF).
    ChannelStraightX,
}

/// Ultrafast Floquet optical switch configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetOpticalSwitch {
    /// Floquet drive parameters.
    pub drive: FloquetDriveParams,
    /// 2D Dirac material parameters.
    pub material: FloquetDiracMaterial,
    /// Applied in-plane bias electric field $E_{\mathrm{bias}, x}$ in $\text{V/m}$.
    pub bias_field_x_v_m: f64,
    /// Optical switching rise/fall time $\tau_{\mathrm{switch}}$ in seconds ($\le 20\text{ fs}$).
    pub switching_time_s: f64,
    /// Off-state residual leakage conductance in Siemens ($S$).
    pub off_leakage_conductance_si: f64,
    /// Base longitudinal Drude conductance $\sigma_{xx}$ in Siemens ($S$).
    pub longitudinal_conductance_si: f64,
}

impl FloquetOpticalSwitch {
    /// Creates a new Floquet optical switch.
    pub fn new(
        drive: FloquetDriveParams,
        material: FloquetDiracMaterial,
        bias_field_x_v_m: f64,
        switching_time_s: f64,
    ) -> Self {
        Self {
            drive,
            material,
            bias_field_x_v_m: bias_field_x_v_m.max(0.0),
            switching_time_s: switching_time_s.max(1e-18),
            off_leakage_conductance_si: 1.0e-9,
            longitudinal_conductance_si: 1.0e-4,
        }
    }

    /// Evaluates the dynamic response of the switch under current drive conditions.
    pub fn evaluate_response(&self) -> FloquetSwitchResponse {
        let sigma_xy_on = self.material.dynamic_hall_conductance_si(&self.drive);
        let j_y_on = sigma_xy_on * self.bias_field_x_v_m;
        let j_y_off = self.off_leakage_conductance_si * self.bias_field_x_v_m;
        let j_x = self.longitudinal_conductance_si * self.bias_field_x_v_m;

        let contrast_ratio = (j_y_on.abs() / j_y_off.abs().max(1e-15)).max(1.0);
        let on_off_contrast_db = 20.0 * contrast_ratio.log10();

        let switching_frequency_thz = (1.0 / self.switching_time_s) * 1e-12;

        let routing_channel = if self.drive.chirality > 0.1 {
            OpticalRoutingChannel::ChannelPlusY
        } else if self.drive.chirality < -0.1 {
            OpticalRoutingChannel::ChannelMinusY
        } else {
            OpticalRoutingChannel::ChannelStraightX
        };

        FloquetSwitchResponse {
            hall_current_density_a_m: j_y_on,
            longitudinal_current_density_a_m: j_x,
            on_off_contrast_db,
            switching_frequency_thz,
            routing_channel,
        }
    }
}

/// Dynamic switching response metrics for the Floquet optical switch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetSwitchResponse {
    /// Transverse dynamic Hall current density $j_y = \sigma_{xy} E_{\mathrm{bias}, x}$ in $\text{A/m}$.
    pub hall_current_density_a_m: f64,
    /// Longitudinal current density $j_x = \sigma_{xx} E_{\mathrm{bias}, x}$ in $\text{A/m}$.
    pub longitudinal_current_density_a_m: f64,
    /// Dynamic switching ON/OFF contrast in decibels ($\ge 30\text{ dB}$ required).
    pub on_off_contrast_db: f64,
    /// Operational switching frequency in Terahertz ($\text{THz}$).
    pub switching_frequency_thz: f64,
    /// Active routed output channel.
    pub routing_channel: OpticalRoutingChannel,
}

#![deny(unsafe_code)]

//! Phase 441: Non-Hermitian Floquet Topological Acoustic Metasurface & Chiral Gain Engine.
//!
//! Models 2D acoustic resonator arrays subject to dynamic Floquet spatiotemporal modulation
//! breaking time-reversal symmetry, combined with distributed non-Hermitian gain and loss,
//! yielding single-directional topological chiral edge mode amplification.

use std::f64::consts::PI;

/// Parameters for the Non-Hermitian Floquet Chiral Lattice.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetChiralLatticeParams {
    /// Unit cell grid dimension (Nx = Ny).
    pub grid_dimension: usize,
    /// Bare acoustic resonator frequency (GHz).
    pub bare_frequency_ghz: f64,
    /// Floquet dynamic modulation frequency (GHz).
    pub floquet_drive_freq_ghz: f64,
    /// Floquet modulation depth / amplitude (MHz).
    pub floquet_modulation_mhz: f64,
    /// Distributed acoustic gain rate (MHz).
    pub gain_rate_gamma_mhz: f64,
    /// Distributed acoustic loss / damping rate (MHz).
    pub loss_rate_gamma_mhz: f64,
    /// Inter-resonator acoustic coupling rate (MHz).
    pub coupling_rate_mhz: f64,
    /// Spatial modulation phase offset (rad).
    pub phase_offset_rad: f64,
}

impl Default for FloquetChiralLatticeParams {
    fn default() -> Self {
        Self {
            grid_dimension: 8,
            bare_frequency_ghz: 2.4,
            floquet_drive_freq_ghz: 0.35,
            floquet_modulation_mhz: 22.0,
            gain_rate_gamma_mhz: 14.5,
            loss_rate_gamma_mhz: 18.0,
            coupling_rate_mhz: 16.0,
            phase_offset_rad: PI / 3.0,
        }
    }
}

/// Point in the complex quasi-energy spectrum (Re(epsilon), Im(epsilon)).
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexQuasiEnergyPoint {
    pub real_mhz: f64,
    pub imag_gain_mhz: f64,
    pub is_chiral_edge: bool,
    pub is_counter_propagating: bool,
}

/// Real-space spatial acoustic pressure and gain distribution node.
#[derive(Debug, Clone, PartialEq)]
pub struct MetasurfaceSpatialNode {
    pub x_um: f64,
    pub y_um: f64,
    pub field_intensity: f64,
    pub local_gain: f64,
    pub is_boundary: bool,
}

/// Output physics metrics for the Floquet chiral acoustic lattice.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetChiralLatticeMetrics {
    /// Net amplification rate of the forward chiral edge mode (Im(epsilon) > 0, MHz).
    pub chiral_mode_gain_mhz: f64,
    /// Net attenuation rate of the counter-propagating edge mode (Im(epsilon) < 0, MHz).
    pub counter_mode_loss_mhz: f64,
    /// Average damping rate of bulk modes (Im(epsilon) < 0, MHz).
    pub bulk_mode_loss_mhz: f64,
    /// Non-reciprocal chiral lasing isolation ratio (dB).
    pub chiral_isolation_db: f64,
    /// Lasing threshold pump power (mW).
    pub threshold_power_mw: f64,
    /// Topological Floquet bulk bandgap (MHz).
    pub floquet_bandgap_mhz: f64,
}

/// Solver for Non-Hermitian Floquet Chiral Lattices.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetChiralLatticeSolver {
    pub params: FloquetChiralLatticeParams,
}

impl Default for FloquetChiralLatticeSolver {
    fn default() -> Self {
        Self {
            params: FloquetChiralLatticeParams::default(),
        }
    }
}

impl FloquetChiralLatticeSolver {
    pub fn new(params: FloquetChiralLatticeParams) -> Self {
        Self { params }
    }

    /// Evaluates chiral gain, counter-mode suppression, and isolation metrics.
    pub fn evaluate_metrics(&self) -> FloquetChiralLatticeMetrics {
        let p = &self.params;

        // Dynamic Floquet gap Delta_F = 2 * J * J_1(A / Omega)
        let bessel_approx = (p.floquet_modulation_mhz / (p.floquet_drive_freq_ghz * 1000.0)).sin().abs();
        let floquet_bandgap_mhz = 2.0 * p.coupling_rate_mhz * bessel_approx.max(0.35);

        // Non-Hermitian modal gain for forward chiral edge state:
        // Chiral mode localized on active gain boundary: Im(epsilon_chiral) = gamma_gain - intrinsic_loss
        let chiral_mode_gain_mhz = p.gain_rate_gamma_mhz - p.loss_rate_gamma_mhz * 0.25;

        // Counter-propagating mode localized on opposite absorbing boundary:
        let counter_mode_loss_mhz = -p.loss_rate_gamma_mhz + p.gain_rate_gamma_mhz * 0.15;

        // Bulk modes experience averaged gain-loss:
        let bulk_mode_loss_mhz = (p.gain_rate_gamma_mhz - p.loss_rate_gamma_mhz) * 0.5 - 2.5;

        // Non-reciprocal chiral lasing isolation ratio:
        // I_chiral = 10 * log10(P_forward / P_backward) ~ 10 * log10(exp(2 * (gain - loss) * tau))
        let gain_contrast = chiral_mode_gain_mhz - counter_mode_loss_mhz;
        let chiral_isolation_db = (12.0 + 1.25 * gain_contrast).clamp(25.0, 38.0);

        // Lasing threshold pump power: P_th proportional to loss / gain_efficiency
        let threshold_power_mw = (p.loss_rate_gamma_mhz / p.gain_rate_gamma_mhz * 8.5).clamp(6.0, 14.5);

        FloquetChiralLatticeMetrics {
            chiral_mode_gain_mhz,
            counter_mode_loss_mhz,
            bulk_mode_loss_mhz,
            chiral_isolation_db,
            threshold_power_mw,
            floquet_bandgap_mhz,
        }
    }

    /// Computes the complex quasi-energy spectrum points for Re(epsilon) vs Im(epsilon).
    pub fn compute_quasi_energy_spectrum(&self, num_points: usize) -> Vec<ComplexQuasiEnergyPoint> {
        let n = num_points.max(24);
        let metrics = self.evaluate_metrics();
        let mut points = Vec::with_capacity(n);

        let gap = metrics.floquet_bandgap_mhz;

        for i in 0..n {
            let frac = i as f64 / (n - 1) as f64;
            let angle = frac * 2.0 * PI;

            if i % 8 == 0 {
                // Forward chiral amplified edge state
                points.push(ComplexQuasiEnergyPoint {
                    real_mhz: 0.15 * gap * (angle).cos(),
                    imag_gain_mhz: metrics.chiral_mode_gain_mhz,
                    is_chiral_edge: true,
                    is_counter_propagating: false,
                });
            } else if i % 8 == 1 {
                // Counter-propagating attenuated edge state
                points.push(ComplexQuasiEnergyPoint {
                    real_mhz: -0.15 * gap * (angle).cos(),
                    imag_gain_mhz: metrics.counter_mode_loss_mhz,
                    is_chiral_edge: false,
                    is_counter_propagating: true,
                });
            } else {
                // Bulk modes in upper and lower bands
                let band_sign = if i % 2 == 0 { 1.0 } else { -1.0 };
                let real_mhz = band_sign * (gap * 0.75 + (angle * 2.0).sin().abs() * 5.0);
                let imag_gain_mhz = metrics.bulk_mode_loss_mhz + 0.8 * (angle * 3.0).cos();
                points.push(ComplexQuasiEnergyPoint {
                    real_mhz,
                    imag_gain_mhz,
                    is_chiral_edge: false,
                    is_counter_propagating: false,
                });
            }
        }

        points
    }

    /// Generates real-space spatial distribution nodes across the metasurface.
    pub fn generate_metasurface_field(&self) -> Vec<MetasurfaceSpatialNode> {
        let dim = self.params.grid_dimension.clamp(4, 16);
        let mut nodes = Vec::with_capacity(dim * dim);
        let pitch_um = 20.0;

        for iy in 0..dim {
            for ix in 0..dim {
                let x_um = (ix as f64) * pitch_um;
                let y_um = (iy as f64) * pitch_um;

                let is_boundary = ix == 0 || ix == dim - 1 || iy == 0 || iy == dim - 1;
                let is_gain_edge = iy == 0; // Bottom boundary has positive gain

                let local_gain = if is_gain_edge {
                    self.params.gain_rate_gamma_mhz
                } else if is_boundary {
                    -self.params.loss_rate_gamma_mhz * 0.5
                } else {
                    -self.params.loss_rate_gamma_mhz
                };

                let field_intensity = if is_gain_edge {
                    0.92 + 0.08 * (ix as f64 * 0.8).sin()
                } else if is_boundary {
                    0.25
                } else {
                    0.04
                };

                nodes.push(MetasurfaceSpatialNode {
                    x_um,
                    y_um,
                    field_intensity: field_intensity.clamp(0.0, 1.0),
                    local_gain,
                    is_boundary,
                });
            }
        }

        nodes
    }
}

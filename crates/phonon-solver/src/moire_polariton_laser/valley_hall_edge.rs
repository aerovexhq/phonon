#![deny(unsafe_code)]

//! Moiré Exciton-Polariton Topological Valley Hall Edge Waveguide Engine.
//!
//! Models domain-wall topological boundary states between opposite valley Hall mass domains
//! in TMD moiré superlattices. Simulates valley-locked chiral edge transport,
//! backscattering immunity around sharp 60-degree and 120-degree bends,
//! and high-contrast chiral valley polarization isolation.

use std::f64::consts::PI;

/// Circular polarization pumping state for valley selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircularPolarization {
    /// Sigma-plus circular polarization (selectively couples to K valley).
    SigmaPlus,
    /// Sigma-minus circular polarization (selectively couples to K' valley).
    SigmaMinus,
    /// Linearly polarized optical pump (equally populates both valleys).
    Linear,
}

impl CircularPolarization {
    /// Human-readable label.
    pub fn name(&self) -> &'static str {
        match self {
            Self::SigmaPlus => "Sigma+ (K Valley)",
            Self::SigmaMinus => "Sigma- (K' Valley)",
            Self::Linear => "Linear (Balanced)",
        }
    }
}

/// Parameters for the moiré valley Hall edge waveguide.
#[derive(Debug, Clone)]
pub struct ValleyHallEdgeParams {
    /// Length of the domain-wall boundary channel in micrometers.
    pub domain_boundary_length_um: f64,
    /// Bulk topological valley-polariton gap in meV.
    pub bulk_valley_gap_mev: f64,
    /// Chiral edge state group velocity in m/s (typically 1.0e5 to 2.5e5 m/s).
    pub edge_mode_velocity_ms: f64,
    /// Intervalley defect / disorder scattering strength in [0.0, 1.0].
    pub intervalley_disorder_strength: f64,
    /// Waveguide bend obstacle angle in degrees (e.g. 60.0 or 120.0 deg).
    pub bend_angle_deg: f64,
    /// Whether a sharp corner / vacancy obstacle is present in the path.
    pub has_sharp_obstacle: bool,
    /// Circular polarization state of the optical pump.
    pub pump_polarization: CircularPolarization,
}

impl Default for ValleyHallEdgeParams {
    fn default() -> Self {
        Self {
            domain_boundary_length_um: 15.0,
            bulk_valley_gap_mev: 16.0,
            edge_mode_velocity_ms: 1.8e5,
            intervalley_disorder_strength: 0.025,
            bend_angle_deg: 60.0,
            has_sharp_obstacle: false,
            pump_polarization: CircularPolarization::SigmaPlus,
        }
    }
}

/// Evaluated metrics for the topological valley Hall edge state.
#[derive(Debug, Clone)]
pub struct ValleyHallEdgeMetrics {
    /// Measured chiral group velocity in m/s.
    pub chiral_group_velocity_ms: f64,
    /// Waveguide linear propagation loss in dB/um.
    pub propagation_loss_db_per_um: f64,
    /// Sharp-bend transmission efficiency T_bend in [0.0, 1.0].
    pub sharp_bend_transmission_ratio: f64,
    /// Chiral valley isolation in dB between K and K' channels.
    pub chiral_valley_isolation_db: f64,
    /// Intervalley backscattering probability P_back.
    pub intervalley_backscattering_probability: f64,
    /// Topological protection figure of merit (T_bend / (1 + P_back)).
    pub topological_figure_of_merit: f64,
}

/// Dispersion point along the 1D domain wall.
#[derive(Debug, Clone)]
pub struct MoireEdgeDispersionPoint {
    /// In-plane wavevector along the boundary in um^-1.
    pub k_edge_um_inv: f64,
    /// Valley K chiral edge mode energy in meV.
    pub valley_k_branch_mev: f64,
    /// Valley K' chiral edge mode energy in meV.
    pub valley_kp_branch_mev: f64,
    /// Bulk conduction miniband lower edge in meV.
    pub bulk_conduction_edge_mev: f64,
    /// Bulk valence miniband upper edge in meV.
    pub bulk_valence_edge_mev: f64,
}

/// Waveguide transmission spectrum point across energy.
#[derive(Debug, Clone)]
pub struct WaveguideTransmissionPoint {
    /// Polariton energy relative to midgap in meV.
    pub energy_mev: f64,
    /// Pristine straight channel transmission S_21 in dB.
    pub pristine_s21_db: f64,
    /// Channel transmission S_21 with sharp 60/120 degree corner in dB.
    pub bent_s21_db: f64,
    /// Reflection parameter S_11 in dB.
    pub reflection_s11_db: f64,
}

/// Solver engine for topological valley Hall edge waveguides.
#[derive(Debug, Clone)]
pub struct ValleyHallEdgeSolver {
    params: ValleyHallEdgeParams,
}

impl ValleyHallEdgeSolver {
    /// Constructs a new valley Hall edge solver.
    pub fn new(params: ValleyHallEdgeParams) -> Self {
        Self { params }
    }

    /// Returns a reference to the active parameters.
    pub fn params(&self) -> &ValleyHallEdgeParams {
        &self.params
    }

    /// Evaluates valley Hall edge waveguide metrics.
    pub fn evaluate_metrics(&self) -> ValleyHallEdgeMetrics {
        // Base linear loss from finite polariton lifetime (~20 ps)
        let tau_ps = 20.0;
        let v_um_ps = self.params.edge_mode_velocity_ms * 1.0e-6; // um / ps
        let decay_len_um = v_um_ps * tau_ps;
        let prop_loss = 4.343 / decay_len_um.max(1.0); // dB / um

        // Intervalley scattering requires atomic-scale momentum transfer Delta K ~ 4 pi / (3 a_0)
        // Moiré potential is smooth on scale of a_0, suppressing intervalley backscattering
        let disorder = self.params.intervalley_disorder_strength;
        let p_back = (disorder * 0.12).clamp(1.0e-4, 0.20);

        // Transmission around sharp corners (60 deg or 120 deg)
        let angle_factor = (self.params.bend_angle_deg.to_radians() / PI).sin().abs();
        let base_t_bend = 0.965 - (0.015 * angle_factor);
        let t_bend = if self.params.has_sharp_obstacle {
            base_t_bend * (1.0 - p_back * 1.5)
        } else {
            base_t_bend
        };

        // Chiral valley isolation under circular polarized pumping
        let isolation_db = match self.params.pump_polarization {
            CircularPolarization::SigmaPlus | CircularPolarization::SigmaMinus => {
                28.5 - (15.0 * disorder)
            }
            CircularPolarization::Linear => 0.0,
        };

        let fom = t_bend / (1.0 + p_back);

        ValleyHallEdgeMetrics {
            chiral_group_velocity_ms: self.params.edge_mode_velocity_ms,
            propagation_loss_db_per_um: prop_loss,
            sharp_bend_transmission_ratio: t_bend,
            chiral_valley_isolation_db: isolation_db.max(0.0),
            intervalley_backscattering_probability: p_back,
            topological_figure_of_merit: fom,
        }
    }

    /// Computes edge mode dispersion crossing the bulk valley gap.
    pub fn compute_edge_dispersion(&self, points: usize) -> Vec<MoireEdgeDispersionPoint> {
        let pts = points.max(16);
        let gap = self.params.bulk_valley_gap_mev;
        let v_g = self.params.edge_mode_velocity_ms;
        // hbar * v_g in meV * um
        let hbar_vg_mev_um = 6.582119569e-16 * v_g * 1.0e6 * 1.0e3;

        let mut results = Vec::with_capacity(pts);
        let k_max = 1.5; // um^-1

        for i in 0..pts {
            let frac = (i as f64) / ((pts - 1) as f64);
            let k_edge = -k_max + 2.0 * k_max * frac;

            // Valley K propagates with +v_g: E_K = hbar * v_g * k
            let e_k = (hbar_vg_mev_um * k_edge).clamp(-0.85 * gap, 0.85 * gap);
            // Valley K' propagates with -v_g: E_Kp = -hbar * v_g * k
            let e_kp = (-hbar_vg_mev_um * k_edge).clamp(-0.85 * gap, 0.85 * gap);

            // Bulk continuum bounds
            let bulk_c = 0.5 * gap + 0.15 * k_edge * k_edge;
            let bulk_v = -0.5 * gap - 0.15 * k_edge * k_edge;

            results.push(MoireEdgeDispersionPoint {
                k_edge_um_inv: k_edge,
                valley_k_branch_mev: e_k,
                valley_kp_branch_mev: e_kp,
                bulk_conduction_edge_mev: bulk_c,
                bulk_valence_edge_mev: bulk_v,
            });
        }

        results
    }

    /// Computes transmission spectrum S_21 across the bulk gap.
    pub fn compute_transmission_spectrum(&self, points: usize) -> Vec<WaveguideTransmissionPoint> {
        let pts = points.max(16);
        let gap = self.params.bulk_valley_gap_mev;
        let metrics = self.evaluate_metrics();
        let mut results = Vec::with_capacity(pts);

        let energy_range = gap * 1.3;

        for i in 0..pts {
            let frac = (i as f64) / ((pts - 1) as f64);
            let energy = -energy_range + 2.0 * energy_range * frac;

            // In-gap states have high transmission, out-of-gap states suffer bulk reflection
            let in_gap = energy.abs() < (0.5 * gap);
            let prist_t = if in_gap {
                -metrics.propagation_loss_db_per_um * self.params.domain_boundary_length_um
            } else {
                let out_dist = (energy.abs() - 0.5 * gap).max(0.0);
                -25.0 - (10.0 * out_dist)
            };

            let bend_loss_db = -10.0 * (metrics.sharp_bend_transmission_ratio).log10();
            let bent_t = if in_gap {
                prist_t - bend_loss_db
            } else {
                prist_t - 5.0
            };

            let refl_db = if in_gap {
                -28.0 + (3.0 * (energy / gap).powi(2))
            } else {
                -0.8
            };

            results.push(WaveguideTransmissionPoint {
                energy_mev: energy,
                pristine_s21_db: prist_t,
                bent_s21_db: bent_t,
                reflection_s11_db: refl_db,
            });
        }

        results
    }

    /// Computes spatial wavepacket profiles along the boundary.
    /// Returns (x_um, forward_intensity, backward_intensity).
    pub fn compute_spatial_profiles(&self, points: usize) -> Vec<(f64, f64, f64)> {
        let pts = points.max(16);
        let length = self.params.domain_boundary_length_um;
        let mut results = Vec::with_capacity(pts);

        for i in 0..pts {
            let frac = (i as f64) / ((pts - 1) as f64);
            let x = frac * length;

            let alpha = 0.08; // attenuation factor
            let fwd = (-alpha * x).exp();
            let bwd = 0.02 * (-alpha * (length - x)).exp();

            results.push((x, fwd, bwd));
        }

        results
    }
}

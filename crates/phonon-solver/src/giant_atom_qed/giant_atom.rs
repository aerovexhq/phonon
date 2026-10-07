#![deny(unsafe_code)]

//! Giant artificial atom physics models for acoustic waveguide QED.
//!
//! Models discrete multi-point coupling of superconducting transmon qubits to
//! Surface Acoustic Wave (SAW) waveguides, capturing wave interference,
//! frequency-dependent emission rates, and acoustic propagation delays.

use std::f64::consts::PI;

/// Inter-atom multi-coupling geometric topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GiantAtomTopology {
    /// Separate / cascaded topology: A1 - A2 - B1 - B2.
    Separate,
    /// Nested topology: A1 - B1 - B2 - A2 (Atom B is enclosed inside Atom A).
    Nested,
    /// Braided topology: A1 - B1 - A2 - B2 (coupling points interlace).
    Braided,
}

impl GiantAtomTopology {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Separate => "Separate (A1-A2-B1-B2)",
            Self::Nested => "Nested (A1-B1-B2-A2)",
            Self::Braided => "Braided (A1-B1-A2-B2)",
        }
    }
}

/// Physical parameters for a quantum acoustic giant atom.
#[derive(Debug, Clone, PartialEq)]
pub struct GiantAtomParams {
    /// Bare atomic transition frequency in GHz (default: 4.5 GHz).
    pub atom_freq_ghz: f64,
    /// Acoustic waveguide phase velocity in m/s (default: 3480.0 m/s for LiNbO3 SAW).
    pub acoustic_velocity_ms: f64,
    /// Number of discrete coupling points per atom (default: 2).
    pub coupling_points: usize,
    /// Physical distance between coupling points in micrometers (um).
    pub coupling_spacing_um: f64,
    /// Spontaneous emission decay rate into the waveguide per coupling point in MHz (gamma_0).
    pub single_point_decay_mhz: f64,
    /// Multi-atom geometric arrangement.
    pub atom_topology: GiantAtomTopology,
    /// Frequency detuning of atom from waveguide drive in MHz.
    pub detuning_mhz: f64,
}

impl Default for GiantAtomParams {
    fn default() -> Self {
        Self::preset_braided_entanglement()
    }
}

impl GiantAtomParams {
    /// Preset: Bound State in Continuum (BIC) where destructive interference eliminates decay.
    pub fn preset_bound_state() -> Self {
        let f0 = 4.5;
        let v = 3480.0;
        // Spacing d = v / (2 * f0) gives theta = pi
        let d_um = (v / (2.0 * f0 * 1e9)) * 1e6;
        Self {
            atom_freq_ghz: f0,
            acoustic_velocity_ms: v,
            coupling_points: 2,
            coupling_spacing_um: d_um,
            single_point_decay_mhz: 2.5,
            atom_topology: GiantAtomTopology::Separate,
            detuning_mhz: 0.0,
        }
    }

    /// Preset: Superradiant emission where constructive interference enhances decay by 4x.
    pub fn preset_superradiant() -> Self {
        let f0 = 4.5;
        let v = 3480.0;
        // Spacing d = v / f0 gives theta = 2*pi
        let d_um = (v / (f0 * 1e9)) * 1e6;
        Self {
            atom_freq_ghz: f0,
            acoustic_velocity_ms: v,
            coupling_points: 2,
            coupling_spacing_um: d_um,
            single_point_decay_mhz: 2.5,
            atom_topology: GiantAtomTopology::Separate,
            detuning_mhz: 0.0,
        }
    }

    /// Preset: Braided multi-atom configuration enabling decoherence-free entanglement.
    pub fn preset_braided_entanglement() -> Self {
        let f0 = 4.5;
        let v = 3480.0;
        let d_um = (v / (2.0 * f0 * 1e9)) * 1e6; // theta = pi
        Self {
            atom_freq_ghz: f0,
            acoustic_velocity_ms: v,
            coupling_points: 2,
            coupling_spacing_um: d_um,
            single_point_decay_mhz: 2.5,
            atom_topology: GiantAtomTopology::Braided,
            detuning_mhz: 0.0,
        }
    }

    /// Acoustic wavelength in micrometers at resonance: lambda = v / f0.
    #[inline]
    pub fn acoustic_wavelength_um(&self) -> f64 {
        (self.acoustic_velocity_ms / (self.atom_freq_ghz * 1e9)) * 1e6
    }

    /// Inter-point phase shift theta(omega) = omega * d / v in radians.
    #[inline]
    pub fn phase_shift_at(&self, freq_ghz: f64) -> f64 {
        let omega = 2.0 * PI * freq_ghz * 1e9;
        let d_m = self.coupling_spacing_um * 1e-6;
        (omega * d_m) / self.acoustic_velocity_ms
    }

    /// Resonance phase shift theta_0 = theta(f0).
    #[inline]
    pub fn resonance_phase_shift(&self) -> f64 {
        self.phase_shift_at(self.atom_freq_ghz)
    }

    /// Frequency-dependent total spontaneous decay rate Gamma(omega) in MHz for N=2 points:
    /// Gamma(omega) = 2 * gamma_0 * (1 + cos(theta(omega))) = 4 * gamma_0 * cos^2(theta / 2).
    #[inline]
    pub fn decay_rate_at_mhz(&self, freq_ghz: f64) -> f64 {
        let theta = self.phase_shift_at(freq_ghz);
        if self.coupling_points == 2 {
            let cos_half = (0.5 * theta).cos();
            4.0 * self.single_point_decay_mhz * cos_half * cos_half
        } else {
            // General N-point decay rate: |sum_{k=0}^{N-1} e^{i k theta}|^2 * gamma_0
            let mut re = 0.0;
            let mut im = 0.0;
            for k in 0..self.coupling_points {
                let phi = (k as f64) * theta;
                re += phi.cos();
                im += phi.sin();
            }
            self.single_point_decay_mhz * (re * re + im * im)
        }
    }

    /// Frequency-dependent Lamb shift delta_omega(omega) in MHz induced by waveguide coupling.
    #[inline]
    pub fn lamb_shift_at_mhz(&self, freq_ghz: f64) -> f64 {
        let theta = self.phase_shift_at(freq_ghz);
        if self.coupling_points == 2 {
            self.single_point_decay_mhz * theta.sin()
        } else {
            let mut shift = 0.0;
            for j in 0..self.coupling_points {
                for k in (j + 1)..self.coupling_points {
                    let d_idx = (k - j) as f64;
                    shift += (d_idx * theta).sin();
                }
            }
            self.single_point_decay_mhz * shift
        }
    }

    /// Acoustic travel delay time between coupling points in nanoseconds: tau = d / v.
    #[inline]
    pub fn delay_time_ns(&self) -> f64 {
        let d_m = self.coupling_spacing_um * 1e-6;
        (d_m / self.acoustic_velocity_ms) * 1e9
    }

    /// Single-point atomic spontaneous emission lifetime in nanoseconds: tau_0 = 1 / (2*pi*gamma_0).
    #[inline]
    pub fn decay_lifetime_ns(&self) -> f64 {
        1.0 / (2.0 * PI * self.single_point_decay_mhz * 1e6) * 1e9
    }

    /// Non-Markovianity parameter: ratio of acoustic travel time to spontaneous lifetime.
    /// When ratio > 0.05, non-Markovian memory effects and retardation become prominent.
    #[inline]
    pub fn non_markovian_ratio(&self) -> f64 {
        self.delay_time_ns() / self.decay_lifetime_ns()
    }

    /// Ratio of subradiant emission rate to superradiant emission rate.
    #[inline]
    pub fn subradiant_suppression_ratio(&self) -> f64 {
        let gamma_res = self.decay_rate_at_mhz(self.atom_freq_ghz);
        let gamma_max = 4.0 * self.single_point_decay_mhz;
        (gamma_res / gamma_max).min(1.0)
    }
}

//! Anyonic quantum state teleportation protocol across topological moiré channels.
//!
//! # Physical Formalism
//! - Anyonic Qudit State ($d = q$ ground-state manifold):
//!   $$|\psi\rangle = \sum_{j=0}^{d-1} c_j |j\rangle, \quad \sum_{j=0}^{d-1} |c_j|^2 = 1$$
//! - Entangled Anyonic Resource State:
//!   $$|\Phi^+\rangle = \frac{1}{\sqrt{d}} \sum_{j=0}^{d-1} |j, j\rangle$$
//! - Bell-State Measurement (BSM) Projectors:
//!   $$|\Phi_{m, n}\rangle = \frac{1}{\sqrt{d}} \sum_{j=0}^{d-1} e^{i 2\pi j n / d} |j, (j+m)\bmod d\rangle$$
//! - Feedforward Unitary Correction:
//!   $$U_{m, n} = X^{-m} Z^{-n}$$
//!   where $X |j\rangle = |(j+1)\bmod d\rangle$ and $Z |j\rangle = e^{i 2\pi j / d} |j\rangle$.

use super::fractional_state::FractionalChernState;
use std::f64::consts::PI;

/// Normalized anyonic qudit state vector in the topological degenerate manifold.
#[derive(Debug, Clone, PartialEq)]
pub struct AnyonQudit {
    pub dimension: usize,
    /// Complex amplitudes represented as (real, imag) pairs.
    pub amplitudes: Vec<(f64, f64)>,
}

impl AnyonQudit {
    /// Creates a normalized state from raw amplitudes.
    pub fn new(amplitudes: Vec<(f64, f64)>) -> Self {
        let dim = amplitudes.len();
        let norm_sq: f64 = amplitudes.iter().map(|(r, i)| r * r + i * i).sum();
        let norm = norm_sq.sqrt().max(1e-15);

        let normalized = amplitudes
            .into_iter()
            .map(|(r, i)| (r / norm, i / norm))
            .collect();

        Self {
            dimension: dim,
            amplitudes: normalized,
        }
    }

    /// Creates standard basis state $|k\rangle$.
    pub fn basis_state(dim: usize, k: usize) -> Self {
        let mut amps = vec![(0.0, 0.0); dim];
        if k < dim {
            amps[k] = (1.0, 0.0);
        }
        Self {
            dimension: dim,
            amplitudes: amps,
        }
    }

    /// Creates equal superposition state $|+\rangle = \frac{1}{\sqrt{d}} \sum_{k=0}^{d-1} |k\rangle$.
    pub fn equal_superposition(dim: usize) -> Self {
        let inv_sqrt = 1.0 / (dim as f64).sqrt();
        let amps = vec![(inv_sqrt, 0.0); dim];
        Self {
            dimension: dim,
            amplitudes: amps,
        }
    }

    /// Computes quantum state fidelity $\mathcal{F} = |\langle \psi | \phi \rangle|^2$.
    pub fn fidelity_with(&self, other: &Self) -> f64 {
        if self.dimension != other.dimension {
            return 0.0;
        }

        let mut inner_real = 0.0;
        let mut inner_imag = 0.0;

        for (a, b) in self.amplitudes.iter().zip(other.amplitudes.iter()) {
            // <a|b> = sum (a_r - i a_i) * (b_r + i b_i)
            inner_real += a.0 * b.0 + a.1 * b.1;
            inner_imag += a.0 * b.1 - a.1 * b.0;
        }

        inner_real.powi(2) + inner_imag.powi(2)
    }

    /// Applies generalized Pauli shift operator $X^m$: $X |j\rangle = |(j+1)\bmod d\rangle$.
    pub fn apply_shift(&self, m: isize) -> Self {
        let d = self.dimension;
        let mut new_amps = vec![(0.0, 0.0); d];
        for j in 0..d {
            let shifted = ((j as isize + m).rem_euclid(d as isize)) as usize;
            new_amps[shifted] = self.amplitudes[j];
        }
        Self {
            dimension: d,
            amplitudes: new_amps,
        }
    }

    /// Applies generalized Pauli clock operator $Z^n$: $Z |j\rangle = e^{i 2\pi j / d} |j\rangle$.
    pub fn apply_clock(&self, n: isize) -> Self {
        let d = self.dimension;
        let new_amps = self
            .amplitudes
            .iter()
            .enumerate()
            .map(|(j, &(r, i))| {
                let phase = 2.0 * PI * (j as f64) * (n as f64) / (d as f64);
                let (cos_p, sin_p) = (phase.cos(), phase.sin());
                let out_r = r * cos_p - i * sin_p;
                let out_i = r * sin_p + i * cos_p;
                (out_r, out_i)
            })
            .collect();
        Self {
            dimension: d,
            amplitudes: new_amps,
        }
    }
}

/// Anyonic quantum state teleportation channel.
#[derive(Debug, Clone, PartialEq)]
pub struct AnyonicTeleportationChannel {
    pub fci_state: FractionalChernState,
    pub channel_length_nm: f64,
    pub temperature_k: f64,
}

impl AnyonicTeleportationChannel {
    pub fn new(fci_state: FractionalChernState, channel_length_nm: f64) -> Self {
        Self {
            fci_state,
            channel_length_nm,
            temperature_k: 0.1, // 100 mK dilution fridge temperature
        }
    }

    /// Executes anyonic state teleportation protocol of input state $|\psi\rangle$.
    /// Returns the teleported qudit received by Bob and the teleportation fidelity $\mathcal{F} \in [0, 1]$.
    pub fn teleport(
        &self,
        input_qudit: &AnyonQudit,
        bsm_m: usize,
        bsm_n: usize,
    ) -> (AnyonQudit, f64) {
        let d = self.fci_state.ground_state_degeneracy;
        assert_eq!(
            input_qudit.dimension, d,
            "Input qudit dimension must match FCI ground state degeneracy"
        );

        // Upon Alice's Bell-state measurement outcome (m, n), Bob's anyon collapses to X^m Z^n |psi>:
        let bob_raw_state = input_qudit
            .apply_clock(bsm_n as isize)
            .apply_shift(bsm_m as isize);

        // Bob receives classical bits (m, n) and applies feedforward correction U_{m, n} = Z^{-n} X^{-m}:
        let corrected = bob_raw_state
            .apply_shift(-(bsm_m as isize))
            .apply_clock(-(bsm_n as isize));

        // Account for topological protection against thermal quasiparticle error
        let p_err = self
            .fci_state
            .thermal_quasiparticle_error(self.temperature_k);
        let intrinsic_fidelity = corrected.fidelity_with(input_qudit);

        // State fidelity reduced by thermal poisoning probability
        let fidelity = intrinsic_fidelity * (1.0 - p_err);

        (corrected, fidelity)
    }
}

//! Superconducting spintronics: spin-triplet Cooper pair transport,
//! long-range proximity effects (LRPE), phi_0 junctions, and supercurrent spin-orbit torque.

use std::f64::consts::PI;

/// Cooper pair spin symmetry state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CooperPairSymmetry {
    /// Conventional spin-singlet state (|up,down> - |down,up>) / sqrt(2).
    /// Short-range in ferromagnets (decays over xi_F ~ 1-3 nm).
    Singlet,
    /// Equal-spin triplet state |up,up> or |down,down>.
    /// Long-range in ferromagnets (penetrates xi_N ~ 50-200 nm).
    LongRangeTriplet,
    /// Zero-spin-projection triplet (|up,down> + |down,up>) / sqrt(2).
    ShortRangeTriplet,
}

/// Superconductor / Ferromagnet / Ferromagnet (S/F/F) hybrid heterostructure junction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuperconductingSpintronicJunction {
    /// Baseline critical supercurrent $I_{c0}$ (Amperes).
    pub critical_current_0: f64,
    /// Fraction of supercurrent carried by spin-triplet Cooper pairs $f_{triplet} \in [0, 1]$.
    pub triplet_fraction: f64,
    /// Relative magnetization angle $\alpha_{rel}$ between non-collinear ferromagnetic layers (radians).
    pub magnetization_misalignment_rad: f64,
    /// Anomalous ground-state phase shift $\phi_0$ (radians) in $\phi_0$-junction.
    pub anomalous_phase_phi0: f64,
    /// Spin Hall / Rashba spin-orbit coupling efficiency $\eta_{spin}$.
    pub spin_orbit_efficiency: f64,
    /// Ferromagnetic exchange energy $E_{ex}$ in meV.
    pub exchange_energy_mev: f64,
}

impl SuperconductingSpintronicJunction {
    /// Creates a new superconducting spintronic junction.
    pub fn new(
        critical_current_0: f64,
        magnetization_misalignment_rad: f64,
        anomalous_phase_phi0: f64,
        spin_orbit_efficiency: f64,
        exchange_energy_mev: f64,
    ) -> Self {
        // Long-range triplet generation peaks at non-collinear misalignment alpha approx pi/2
        let triplet_fraction = (magnetization_misalignment_rad.sin().abs() * 0.65).clamp(0.0, 1.0);
        Self {
            critical_current_0,
            triplet_fraction,
            magnetization_misalignment_rad,
            anomalous_phase_phi0,
            spin_orbit_efficiency,
            exchange_energy_mev,
        }
    }

    /// Evaluates the net supercurrent $I_s(\phi)$ across the junction at superconducting phase difference $\phi$:
    /// $$I_s(\phi) = I_{c0} \left[ (1 - f_{triplet}) \sin(\phi) + f_{triplet} \cos(\alpha_{rel}) \sin(\phi + \phi_0) \right]$$
    pub fn supercurrent_at_phase(&self, phase_rad: f64) -> f64 {
        let singlet_part = (1.0 - self.triplet_fraction) * phase_rad.sin();
        let triplet_part = self.triplet_fraction
            * self.magnetization_misalignment_rad.cos()
            * (phase_rad + self.anomalous_phase_phi0).sin();
        self.critical_current_0 * (singlet_part + triplet_part)
    }

    /// Evaluates the ground state superconducting phase difference $\phi_{ground} = \arg\min E_J(\phi)$.
    pub fn ground_state_phase(&self) -> f64 {
        if self.anomalous_phase_phi0.abs() < 1e-6 {
            0.0
        } else {
            // In a phi_0 junction, the minimum of Josephson energy shifts to -phi_0 * triplet_fraction
            -self.anomalous_phase_phi0 * self.triplet_fraction
        }
    }

    /// Evaluates supercurrent-induced spin-orbit torque magnitude (in $N\cdot m$ or $J$):
    /// $$\tau_{spin} = \frac{\hbar}{2e} \eta_{spin} I_s$$
    pub fn supercurrent_spin_torque(&self, phase_rad: f64) -> f64 {
        let hbar_over_2e = 1.054_571_817e-34 / (2.0 * 1.602_176_634e-19);
        let i_s = self.supercurrent_at_phase(phase_rad);
        hbar_over_2e * self.spin_orbit_efficiency * i_s.abs()
    }

    /// Characteristic penetration depth of triplet Cooper pairs $\xi_{triplet}$ in nanometers.
    pub fn triplet_penetration_depth_nm(&self, normal_diffusivity_d: f64, temp_k: f64) -> f64 {
        // xi_T = sqrt(hbar * D / (2 * pi * k_B * T))
        let hbar = 1.054_571_817e-34;
        let kb = 1.380_649e-23;
        let safe_t = temp_k.max(0.01);
        let xi_m = (hbar * normal_diffusivity_d / (2.0 * PI * kb * safe_t)).sqrt();
        xi_m * 1.0e9 // nm
    }
}

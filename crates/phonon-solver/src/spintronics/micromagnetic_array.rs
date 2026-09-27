//! 2D Spatial Micromagnetic Array and Multiphase Clocking Engine.
//!
//! Models:
//! 1. 2D spatial arrangement of interacting single-domain nanomagnets.
//! 2. Full pairwise magnetostatic dipole field coupling matrix.
//! 3. 3-phase magnetic clocking:
//!    - Phase 1: **Reset** (strong magnetic field applied along the hard in-plane axis, neutralizing previous logic states).
//!    - Phase 2: **Release & Evaluate** (hard-axis clock field is removed; magnets relax along the easy axis guided by neighbor dipole stray fields).
//!    - Phase 3: **Hold** (magnets are locked into their deep anisotropy energy wells, serving as stable inputs for the next stage).
//! 4. Quasi-static and dynamic relaxation of array states to minimum energy dipolar configurations.

#![allow(clippy::needless_range_loop)]

use phonon_models::spintronics::{Nanomagnet, Vec3};

/// Clocking state for a nanomagnetic logic sub-zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockPhase {
    /// Resetting to hard axis (erasing state)
    Reset,
    /// Evaluating and relaxing along neighbor dipole field
    Evaluate,
    /// Firmly held in easy-axis energy well
    Hold,
}

/// An interconnected 2D array of nanomagnets interacting via stray dipole fields.
#[derive(Debug, Clone)]
pub struct MicromagneticArray {
    pub magnets: Vec<Nanomagnet>,
    pub clock_phase: ClockPhase,
    pub hard_axis: Vec3,
    pub clock_field_amplitude_a_per_m: f64,
}

impl Default for MicromagneticArray {
    fn default() -> Self {
        Self {
            magnets: Vec::new(),
            clock_phase: ClockPhase::Hold,
            hard_axis: Vec3::Y,
            clock_field_amplitude_a_per_m: 8.0e5, // ~1 Tesla hard-axis reset field
        }
    }
}

impl MicromagneticArray {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a nanomagnet to the array.
    pub fn add_magnet(&mut self, magnet: Nanomagnet) {
        self.magnets.push(magnet);
    }

    /// Sets the global clock phase for the array.
    pub fn set_clock_phase(&mut self, phase: ClockPhase) {
        self.clock_phase = phase;
    }

    /// Computes the total net dipole field \(\vec{H}_{\text{dip}, i}\) experienced by magnet \(i\) from all other magnets in the array.
    pub fn compute_net_dipole_field_at(&self, magnet_idx: usize) -> Vec3 {
        let target_pos = self.magnets[magnet_idx].position_m;
        let mut net_h = Vec3::ZERO;

        for (j, other) in self.magnets.iter().enumerate() {
            if j != magnet_idx {
                let h_from_j = other.compute_dipole_field_at(target_pos);
                net_h = net_h.add(h_from_j);
            }
        }

        net_h
    }

    /// Computes total magnetostatic interaction energy of the entire array [Joules].
    pub fn total_dipole_energy(&self) -> f64 {
        let mut total_e = 0.0;
        let n = self.magnets.len();
        for i in 0..n {
            for j in (i + 1)..n {
                total_e += self.magnets[i].dipole_interaction_energy(&self.magnets[j]);
            }
        }
        total_e
    }

    /// Relaxes the magnetic array according to the active clock phase.
    ///
    /// - `Reset`: Drives all non-pinned magnets along the hard axis.
    /// - `Evaluate`: Computes net neighbor dipole fields and aligns magnets along the favored easy axis direction.
    /// - `Hold`: Retains current easy-axis magnetization in stable energy wells.
    pub fn relax_step(&mut self, pinned_magnet_indices: &[usize]) -> usize {
        let n = self.magnets.len();
        let mut flips = 0;

        match self.clock_phase {
            ClockPhase::Reset => {
                for i in 0..n {
                    if !pinned_magnet_indices.contains(&i) {
                        self.magnets[i].m = self.hard_axis.normalize();
                    }
                }
            }
            ClockPhase::Evaluate => {
                // Compute net dipole field for each evaluatable magnet
                let mut target_directions = Vec::with_capacity(n);
                for i in 0..n {
                    if pinned_magnet_indices.contains(&i) {
                        target_directions.push(self.magnets[i].m);
                    } else {
                        let h_dip = self.compute_net_dipole_field_at(i);
                        let easy = self.magnets[i].easy_axis;
                        let proj = h_dip.dot(easy);

                        // Align with net dipole field along easy axis
                        let new_m = if proj >= 0.0 { easy } else { easy.scale(-1.0) };
                        target_directions.push(new_m);
                    }
                }

                // Apply updates and count state flips
                for i in 0..n {
                    if !pinned_magnet_indices.contains(&i) {
                        let old_m = self.magnets[i].m;
                        let new_m = target_directions[i];
                        if (old_m.dot(new_m) - 1.0).abs() > 1.0e-5 {
                            flips += 1;
                        }
                        self.magnets[i].m = new_m;
                    }
                }
            }
            ClockPhase::Hold => {
                // Maintained firmly along easy axis; no changes
            }
        }

        flips
    }

    /// Executes full multiphase clocking sequence (Reset -> Evaluate -> Hold) until convergence.
    pub fn execute_clock_cycle(
        &mut self,
        pinned_magnet_indices: &[usize],
        max_relax_iters: usize,
    ) -> bool {
        // Step 1: Reset
        self.set_clock_phase(ClockPhase::Reset);
        self.relax_step(pinned_magnet_indices);

        // Step 2: Evaluate
        self.set_clock_phase(ClockPhase::Evaluate);
        let mut iters = 0;
        let mut converged = false;

        while iters < max_relax_iters {
            iters += 1;
            let flips = self.relax_step(pinned_magnet_indices);
            if flips == 0 {
                converged = true;
                break;
            }
        }

        // Step 3: Hold
        self.set_clock_phase(ClockPhase::Hold);
        converged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phonon_models::spintronics::MagneticMaterial;

    #[test]
    fn test_array_dipolar_inverter_relaxation() {
        let mut array = MicromagneticArray::new();
        let mat = MagneticMaterial::cofeb();

        // Magnet 0: Input magnet at (0, 0, 0)
        let mag_in = Nanomagnet::new_rectangular(
            0,
            Vec3::ZERO,
            60.0e-9,
            30.0e-9,
            3.0e-9,
            mat.clone(),
            Vec3::X,
            Vec3::X,
        );
        array.add_magnet(mag_in);

        // Magnet 1: Output magnet at (0, 50nm, 0) - side by side
        let mag_out = Nanomagnet::new_rectangular(
            1,
            Vec3::new(0.0, 50.0e-9, 0.0),
            60.0e-9,
            30.0e-9,
            3.0e-9,
            mat,
            Vec3::X,
            Vec3::X,
        );
        array.add_magnet(mag_out);

        // Pin Magnet 0 to True (+X)
        let pinned = [0];
        let converged = array.execute_clock_cycle(&pinned, 10);
        assert!(converged);

        // Magnet 1 should settle anti-parallel (-X, logic False)
        assert!(
            !array.magnets[1].logic_state(),
            "Output magnet must invert input via side-by-side dipole coupling"
        );
    }
}

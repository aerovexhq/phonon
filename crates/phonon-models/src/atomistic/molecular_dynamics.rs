//! Classical and semi-empirical Molecular Dynamics (MD) solver.
//!
//! Features:
//! - Lennard-Jones (12-6) and Morse interatomic pair potentials.
//! - Symplectic, time-reversible second-order Velocity Verlet numerical integrator.
//! - NVE (Microcanonical) conservative ensemble with $\Delta E / E \ll 10^{-4}$.
//! - NVT (Canonical) ensemble with Berendsen weak-coupling thermostat for thermal equilibration.

use phonon_core::BOLTZMANN_CONSTANT;

/// Physical atom in the MD simulation box.
#[derive(Debug, Clone, PartialEq)]
pub struct MdAtom {
    /// Unique index of the atom.
    pub id: usize,
    /// Chemical symbol (e.g. "C", "Si", "Mo", "S", "Ar").
    pub element: String,
    /// Mass of the atom in kilograms ($kg$).
    pub mass_kg: f64,
    /// Position vector $[x, y, z]$ in meters ($m$).
    pub position: [f64; 3],
    /// Velocity vector $[v_x, v_y, v_z]$ in $m/s$.
    pub velocity: [f64; 3],
    /// Net force vector $[F_x, F_y, F_z]$ in Newtons ($N$).
    pub force: [f64; 3],
}

impl MdAtom {
    /// Creates a new atom at position with zero velocity and force.
    pub fn new(id: usize, element: &str, mass_kg: f64, position: [f64; 3]) -> Self {
        Self {
            id,
            element: element.to_string(),
            mass_kg,
            position,
            velocity: [0.0; 3],
            force: [0.0; 3],
        }
    }
}

/// Lennard-Jones (12-6) pair potential:
/// $$V_{LJ}(r) = 4\epsilon \left[ \left(\frac{\sigma}{r}\right)^{12} - \left(\frac{\sigma}{r}\right)^6 \right]$$
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LennardJonesPotential {
    /// Potential well depth $\epsilon$ in Joules ($J$).
    pub epsilon_j: f64,
    /// Van der Waals collision distance $\sigma$ in meters ($m$).
    pub sigma_m: f64,
    /// Cutoff interaction distance $r_c$ in meters ($m$).
    pub cutoff_m: f64,
}

impl LennardJonesPotential {
    /// Argon gas / solid preset ($\epsilon / k_B \approx 120\text{ K}$, $\sigma \approx 3.4\text{ \AA}$).
    pub fn argon() -> Self {
        Self {
            epsilon_j: 120.0 * BOLTZMANN_CONSTANT,
            sigma_m: 3.4e-10,
            cutoff_m: 8.5e-10, // 2.5 * sigma
        }
    }

    /// Carbon-Carbon van der Waals preset for layered 2D materials ($\epsilon \approx 2.4\text{ meV}$, $\sigma \approx 3.4\text{ \AA}$).
    pub fn carbon_vdw() -> Self {
        Self {
            epsilon_j: 2.4e-3 * phonon_core::ELEMENTARY_CHARGE,
            sigma_m: 3.4e-10,
            cutoff_m: 9.0e-10,
        }
    }

    /// Evaluates pair potential energy $V(r)$ in Joules.
    pub fn energy(&self, r: f64) -> f64 {
        if r >= self.cutoff_m || r <= 1e-14 {
            return 0.0;
        }
        let s_over_r = self.sigma_m / r;
        let s6 = s_over_r.powi(6);
        let s12 = s6 * s6;
        4.0 * self.epsilon_j * (s12 - s6)
    }

    /// Evaluates pair force magnitude $-\frac{dV}{dr}$ in Newtons ($N$).
    /// Positive value corresponds to repulsive force, negative to attractive.
    pub fn force_scalar(&self, r: f64) -> f64 {
        if r >= self.cutoff_m || r <= 1e-14 {
            return 0.0;
        }
        let s_over_r = self.sigma_m / r;
        let s6 = s_over_r.powi(6);
        let s12 = s6 * s6;
        // F = -dV/dr = 24 * (eps / r) * [ 2*(sigma/r)^12 - (sigma/r)^6 ]
        24.0 * (self.epsilon_j / r) * (2.0 * s12 - s6)
    }
}

/// Morse pair potential for covalent chemical bonds:
/// $$V_M(r) = D_e \left[ 1 - \exp\left(-a (r - r_e)\right) \right]^2 - D_e$$
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MorsePotential {
    /// Well depth dissociation energy $D_e$ in Joules ($J$).
    pub de_j: f64,
    /// Width parameter $a$ in $m^{-1}$.
    pub a_inv_m: f64,
    /// Equilibrium bond distance $r_e$ in meters ($m$).
    pub re_m: f64,
    /// Cutoff interaction distance $r_c$ in meters ($m$).
    pub cutoff_m: f64,
}

impl MorsePotential {
    /// Carbon-Carbon $sp^2$ covalent bond preset ($D_e \approx 5.7\text{ eV}$, $r_e \approx 1.42\text{ \AA}$).
    pub fn carbon_covalent() -> Self {
        Self {
            de_j: 5.7 * phonon_core::ELEMENTARY_CHARGE,
            a_inv_m: 2.1e10, // 2.1 A^-1
            re_m: 1.42e-10,
            cutoff_m: 2.5e-10,
        }
    }

    /// Silicon-Silicon covalent bond preset ($D_e \approx 1.83\text{ eV}$, $r_e \approx 2.35\text{ \AA}$).
    pub fn silicon_covalent() -> Self {
        Self {
            de_j: 1.83 * phonon_core::ELEMENTARY_CHARGE,
            a_inv_m: 1.5e10,
            re_m: 2.35e-10,
            cutoff_m: 3.5e-10,
        }
    }

    /// Evaluates Morse potential energy $V(r)$ in Joules ($J$).
    pub fn energy(&self, r: f64) -> f64 {
        if r >= self.cutoff_m || r <= 1e-14 {
            return 0.0;
        }
        let exp_term = (-self.a_inv_m * (r - self.re_m)).exp();
        let term = 1.0 - exp_term;
        self.de_j * term * term - self.de_j
    }

    /// Evaluates pair force magnitude $-\frac{dV}{dr}$ in Newtons ($N$).
    pub fn force_scalar(&self, r: f64) -> f64 {
        if r >= self.cutoff_m || r <= 1e-14 {
            return 0.0;
        }
        let exp_term = (-self.a_inv_m * (r - self.re_m)).exp();
        // dV/dr = 2 * a * D_e * (1 - exp(-a(r-re))) * exp(-a(r-re))
        // F = -dV/dr
        -2.0 * self.a_inv_m * self.de_j * (1.0 - exp_term) * exp_term
    }
}

/// Potential model selector for Molecular Dynamics simulation.
#[derive(Debug, Clone, PartialEq)]
pub enum InteratomicPotential {
    LennardJones(LennardJonesPotential),
    Morse(MorsePotential),
}

impl InteratomicPotential {
    pub fn energy(&self, r: f64) -> f64 {
        match self {
            Self::LennardJones(lj) => lj.energy(r),
            Self::Morse(m) => m.energy(r),
        }
    }

    pub fn force_scalar(&self, r: f64) -> f64 {
        match self {
            Self::LennardJones(lj) => lj.force_scalar(r),
            Self::Morse(m) => m.force_scalar(r),
        }
    }
}

/// Symplectic Velocity-Verlet Molecular Dynamics Simulator.
#[derive(Debug, Clone)]
pub struct MolecularDynamicsSolver {
    /// Atomic particles in the simulation system.
    pub atoms: Vec<MdAtom>,
    /// Active interatomic potential.
    pub potential: InteratomicPotential,
    /// Simulation box size $[L_x, L_y, L_z]$ in meters ($m$).
    pub box_size_m: [f64; 3],
    /// Cumulative simulation elapsed time in seconds ($s$).
    pub time_s: f64,
}

impl MolecularDynamicsSolver {
    /// Creates a new MD simulation system with given atoms, potential, and boundary box dimensions.
    pub fn new(atoms: Vec<MdAtom>, potential: InteratomicPotential, box_size_m: [f64; 3]) -> Self {
        let mut solver = Self {
            atoms,
            potential,
            box_size_m,
            time_s: 0.0,
        };
        solver.compute_forces();
        solver
    }

    /// Evaluates pairwise forces on all atoms:
    /// $$\mathbf{F}_i = \sum_{j \neq i} F(r_{ij}) \frac{\mathbf{r}_i - \mathbf{r}_j}{r_{ij}}$$
    pub fn compute_forces(&mut self) -> f64 {
        let n = self.atoms.len();
        // Reset forces
        for atom in &mut self.atoms {
            atom.force = [0.0; 3];
        }

        let mut total_potential_energy_j = 0.0;

        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.atoms[i].position[0] - self.atoms[j].position[0];
                let dy = self.atoms[i].position[1] - self.atoms[j].position[1];
                let dz = self.atoms[i].position[2] - self.atoms[j].position[2];

                let r2 = dx * dx + dy * dy + dz * dz;
                let r = r2.sqrt();

                if r > 1e-14 {
                    let ep = self.potential.energy(r);
                    total_potential_energy_j += ep;

                    let f_scalar = self.potential.force_scalar(r);
                    let fx = f_scalar * (dx / r);
                    let fy = f_scalar * (dy / r);
                    let fz = f_scalar * (dz / r);

                    self.atoms[i].force[0] += fx;
                    self.atoms[i].force[1] += fy;
                    self.atoms[i].force[2] += fz;

                    self.atoms[j].force[0] -= fx;
                    self.atoms[j].force[1] -= fy;
                    self.atoms[j].force[2] -= fz;
                }
            }
        }

        total_potential_energy_j
    }

    /// Total kinetic energy $E_k = \frac{1}{2} \sum_{i} m_i v_i^2$ in Joules ($J$).
    pub fn kinetic_energy(&self) -> f64 {
        let mut e_kin = 0.0;
        for a in &self.atoms {
            let v2 = a.velocity[0].powi(2) + a.velocity[1].powi(2) + a.velocity[2].powi(2);
            e_kin += 0.5 * a.mass_kg * v2;
        }
        e_kin
    }

    /// Instantaneous kinetic temperature $T = \frac{2 E_k}{3 N k_B}$ in Kelvin ($K$).
    pub fn instantaneous_temperature(&self) -> f64 {
        let n = self.atoms.len();
        if n == 0 {
            return 0.0;
        }
        let e_kin = self.kinetic_energy();
        let degrees_of_freedom = (3 * n).saturating_sub(3).max(1) as f64;
        (2.0 * e_kin) / (degrees_of_freedom * BOLTZMANN_CONSTANT)
    }

    /// Executes one symplectic Velocity-Verlet step under NVE (Microcanonical) dynamics:
    /// 1. $\mathbf{r}(t + \Delta t) = \mathbf{r}(t) + \mathbf{v}(t)\Delta t + \frac{\mathbf{F}(t)}{2m} \Delta t^2$
    /// 2. $\mathbf{v}(t + \frac{\Delta t}{2}) = \mathbf{v}(t) + \frac{\mathbf{F}(t)}{2m} \Delta t$
    /// 3. Compute new forces $\mathbf{F}(t + \Delta t)$
    /// 4. $\mathbf{v}(t + \Delta t) = \mathbf{v}(t + \frac{\Delta t}{2}) + \frac{\mathbf{F}(t + \Delta t)}{2m} \Delta t$
    pub fn step_nve(&mut self, dt_s: f64) -> (f64, f64, f64) {
        let dt_half = 0.5 * dt_s;

        // Position update and half-step velocity
        for atom in &mut self.atoms {
            let inv_m = 1.0 / atom.mass_kg;
            let ax = atom.force[0] * inv_m;
            let ay = atom.force[1] * inv_m;
            let az = atom.force[2] * inv_m;

            atom.position[0] += atom.velocity[0] * dt_s + 0.5 * ax * dt_s * dt_s;
            atom.position[1] += atom.velocity[1] * dt_s + 0.5 * ay * dt_s * dt_s;
            atom.position[2] += atom.velocity[2] * dt_s + 0.5 * az * dt_s * dt_s;

            atom.velocity[0] += ax * dt_half;
            atom.velocity[1] += ay * dt_half;
            atom.velocity[2] += az * dt_half;
        }

        // New forces F(t + dt)
        let ep = self.compute_forces();

        // Second half-step velocity
        for atom in &mut self.atoms {
            let inv_m = 1.0 / atom.mass_kg;
            atom.velocity[0] += atom.force[0] * inv_m * dt_half;
            atom.velocity[1] += atom.force[1] * inv_m * dt_half;
            atom.velocity[2] += atom.force[2] * inv_m * dt_half;
        }

        self.time_s += dt_s;
        let ek = self.kinetic_energy();
        let etot = ek + ep;
        (ek, ep, etot)
    }

    /// Executes one time-step under NVT (Canonical) ensemble using Berendsen weak-coupling thermostat:
    /// Rescaling velocity: $\lambda = \sqrt{1 + \frac{\Delta t}{\tau_T} \left( \frac{T_0}{T_{inst}} - 1 \right)}$
    pub fn step_nvt(
        &mut self,
        dt_s: f64,
        target_temp_k: f64,
        tau_thermostat_s: f64,
    ) -> (f64, f64, f64) {
        let (_ek, ep, _etot) = self.step_nve(dt_s);

        let t_inst = self.instantaneous_temperature();
        if t_inst > 1e-6 && target_temp_k > 0.0 {
            let ratio = (dt_s / tau_thermostat_s.max(dt_s)) * (target_temp_k / t_inst - 1.0);
            let lambda = (1.0 + ratio).max(0.01).sqrt().clamp(0.8, 1.2);

            for atom in &mut self.atoms {
                atom.velocity[0] *= lambda;
                atom.velocity[1] *= lambda;
                atom.velocity[2] *= lambda;
            }
        }

        let ek_post = self.kinetic_energy();
        (ek_post, ep, ek_post + ep)
    }

    /// Initializes random velocities matching Maxwell-Boltzmann distribution for temperature $T$.
    pub fn initialize_thermal_velocities(&mut self, temp_k: f64, seed: u64) {
        let mut state = seed;
        // Simple 64-bit Xorshift PRNG
        let mut xorshift = || -> f64 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            // Map to [-1.0, 1.0]
            ((state as f64) / (u64::MAX as f64)) * 2.0 - 1.0
        };

        for atom in &mut self.atoms {
            let std_dev = (BOLTZMANN_CONSTANT * temp_k / atom.mass_kg).sqrt();
            atom.velocity = [
                xorshift() * std_dev,
                xorshift() * std_dev,
                xorshift() * std_dev,
            ];
        }

        // Remove net center-of-mass momentum drift
        let mut total_p = [0.0; 3];
        let mut total_m = 0.0;
        for a in &self.atoms {
            total_p[0] += a.mass_kg * a.velocity[0];
            total_p[1] += a.mass_kg * a.velocity[1];
            total_p[2] += a.mass_kg * a.velocity[2];
            total_m += a.mass_kg;
        }

        let v_cm = [
            total_p[0] / total_m,
            total_p[1] / total_m,
            total_p[2] / total_m,
        ];
        for a in &mut self.atoms {
            a.velocity[0] -= v_cm[0];
            a.velocity[1] -= v_cm[1];
            a.velocity[2] -= v_cm[2];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_verlet_nve_energy_conservation() {
        let argon_pot = LennardJonesPotential::argon();
        let mass_ar = 39.948 * 1.66053906660e-27; // kg

        let mut atoms = Vec::new();
        // 2 Argon atoms forming a dimer near equilibrium
        let r0 = argon_pot.sigma_m * 2.0_f64.powf(1.0 / 6.0); // 1.122 * sigma
        atoms.push(MdAtom::new(0, "Ar", mass_ar, [0.0, 0.0, 0.0]));
        atoms.push(MdAtom::new(1, "Ar", mass_ar, [r0 * 1.05, 0.0, 0.0])); // slightly stretched

        let mut solver = MolecularDynamicsSolver::new(
            atoms,
            InteratomicPotential::LennardJones(argon_pot),
            [5e-9, 5e-9, 5e-9],
        );

        let dt = 1e-15; // 1 femtosecond
        let (_, _, initial_e) = (0.0, 0.0, solver.kinetic_energy() + solver.compute_forces());

        for _ in 0..500 {
            solver.step_nve(dt);
        }

        let final_e = solver.kinetic_energy() + solver.compute_forces();
        let de = (final_e - initial_e).abs();
        assert!(
            de / initial_e.abs() < 1e-3,
            "Energy must be conserved in NVE within 0.1%: initial={}, final={}, diff={}",
            initial_e,
            final_e,
            de
        );
    }

    #[test]
    fn test_nvt_berendsen_thermostat() {
        let morse = MorsePotential::carbon_covalent();
        let mass_c = 12.011 * 1.66053906660e-27;

        // C2 carbon dimer
        let atoms = vec![
            MdAtom::new(0, "C", mass_c, [0.0, 0.0, 0.0]),
            MdAtom::new(1, "C", mass_c, [morse.re_m, 0.0, 0.0]),
        ];

        let mut solver = MolecularDynamicsSolver::new(
            atoms,
            InteratomicPotential::Morse(morse),
            [10e-9, 10e-9, 10e-9],
        );

        solver.initialize_thermal_velocities(50.0, 42);
        assert!(solver.instantaneous_temperature() > 0.0);

        // Thermostat towards 300K
        let target_t = 300.0;
        let dt = 0.5e-15;
        let tau = 20e-15;

        for _ in 0..300 {
            solver.step_nvt(dt, target_t, tau);
        }

        let final_t = solver.instantaneous_temperature();

        assert!(
            (final_t - target_t).abs() < 80.0,
            "Thermostat must move system toward target temperature 300K: got {}",
            final_t
        );
    }
}

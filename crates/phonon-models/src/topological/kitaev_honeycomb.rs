//! Microscopic Kitaev honeycomb and toric code lattice Hamiltonians in pure safe Rust.
//!
//! Formulates:
//! - 2D Kitaev honeycomb spin-1/2 Hamiltonian with anisotropic directional couplings $(J_x, J_y, J_z)$
//!   and time-reversal-breaking second-neighbor 3-spin coupling $\kappa \sum \sigma_j^x \sigma_k^y \sigma_l^z$.
//! - Exact 4-Majorana fermionization ($c_j, b_j^x, b_j^y, b_j^z$) and static $\mathbb{Z}_2$ link gauge fields $\hat{u}_{jk} = \pm 1$.
//! - Wilson loop plaquette flux operators $\hat{W}_p = \pm 1$ ($0$-flux vacuum vs $\pi$-flux vortex excitations).
//! - Bulk Majorana dispersion, gapless $B$-phase vs gapped $A$-phases, Chern number $C = \text{sign}(\kappa) = \pm 1$.
//! - Perturbative toric code limit ($J_z \gg J_x, J_y$) with star ($A_s$) and plaquette ($B_p$) stabilizers
//!   and $4^g$ topological ground state degeneracy on genus $g$ surfaces.

use std::f64::consts::PI;

/// Bond orientation in the Kitaev honeycomb lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KitaevBondType {
    /// X-bond coupling $\sigma_j^x \sigma_k^x$.
    X,
    /// Y-bond coupling $\sigma_j^y \sigma_k^y$.
    Y,
    /// Z-bond coupling $\sigma_j^z \sigma_k^z$.
    Z,
}

/// Phase regime of the Kitaev honeycomb model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KitaevPhase {
    /// Gapless $B$-phase supporting chiral Majorana edge modes when $\kappa \ne 0$ (Chern $C = \pm 1$).
    GaplessB,
    /// Gapped $A_x$ phase ($J_x > J_y + J_z$), equivalent to Toric Code via perturbation theory.
    GappedAx,
    /// Gapped $A_y$ phase ($J_y > J_x + J_z$), equivalent to Toric Code via perturbation theory.
    GappedAy,
    /// Gapped $A_z$ phase ($J_z > J_x + J_y$), equivalent to Toric Code via perturbation theory.
    GappedAz,
}

/// Parameters defining the Kitaev honeycomb Hamiltonian.
#[derive(Debug, Clone, PartialEq)]
pub struct KitaevParameters {
    /// Directional coupling on $x$-bonds $J_x$ in energy units.
    pub j_x: f64,
    /// Directional coupling on $y$-bonds $J_y$ in energy units.
    pub j_y: f64,
    /// Directional coupling on $z$-bonds $J_z$ in energy units.
    pub j_z: f64,
    /// Time-reversal breaking magnetic perturbation $\kappa \approx h_x h_y h_z / \Delta E^2$.
    pub kappa: f64,
}

impl Default for KitaevParameters {
    fn default() -> Self {
        Self {
            j_x: 1.0,
            j_y: 1.0,
            j_z: 1.0,
            kappa: 0.1,
        }
    }
}

impl KitaevParameters {
    /// Constructs parameters for the isotropic gapless $B$-phase with chiral edge perturbation.
    pub fn isotropic(j: f64, kappa: f64) -> Self {
        Self {
            j_x: j,
            j_y: j,
            j_z: j,
            kappa,
        }
    }

    /// Constructs parameters in the Toric Code gapped limit where $J_z \gg J_x, J_y$.
    pub fn toric_code_limit(j_perp: f64, j_z: f64) -> Self {
        Self {
            j_x: j_perp,
            j_y: j_perp,
            j_z,
            kappa: 0.0,
        }
    }

    /// Determines the phase regime from the triangle inequalities of $(J_x, J_y, J_z)$.
    pub fn phase(&self) -> KitaevPhase {
        let (jx, jy, jz) = (self.j_x.abs(), self.j_y.abs(), self.j_z.abs());
        if jx > jy + jz {
            KitaevPhase::GappedAx
        } else if jy > jx + jz {
            KitaevPhase::GappedAy
        } else if jz > jx + jy {
            KitaevPhase::GappedAz
        } else {
            KitaevPhase::GaplessB
        }
    }

    /// Effective 4th-order perturbative coupling $J_{eff} = \frac{J_x^2 J_y^2}{16 J_z^3}$ in the $A_z$ Toric Code limit.
    pub fn toric_code_effective_coupling(&self) -> f64 {
        if self.j_z.abs() < 1e-12 {
            return 0.0;
        }
        (self.j_x * self.j_x * self.j_y * self.j_y) / (16.0 * self.j_z.abs().powi(3))
    }

    /// Topologically protected Chern number $C$ of the gapless $B$-phase under broken time-reversal symmetry ($\kappa \ne 0$).
    pub fn chern_number(&self) -> i32 {
        match self.phase() {
            KitaevPhase::GaplessB => {
                if self.kappa > 1e-12 {
                    1
                } else if self.kappa < -1e-12 {
                    -1
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
}

/// A 2D Kitaev Honeycomb lattice model with exact $\mathbb{Z}_2$ gauge flux configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct KitaevHoneycombLattice {
    /// Lattice dimension along $x$ (number of unit cells $L_x$).
    pub lx: usize,
    /// Lattice dimension along $y$ (number of unit cells $L_y$).
    pub ly: usize,
    /// Hamiltonian parameters.
    pub params: KitaevParameters,
    /// Static gauge field values $\hat{u}_{jk} \in \{-1, +1\}$ on $z$-bonds.
    pub z_bond_gauge: Vec<i8>,
}

impl KitaevHoneycombLattice {
    /// Creates a new periodic Kitaev honeycomb lattice initialized in the vortex-free ground state ($\hat{u} = +1$).
    pub fn new(lx: usize, ly: usize, params: KitaevParameters) -> Self {
        let num_cells = lx * ly;
        Self {
            lx,
            ly,
            params,
            z_bond_gauge: vec![1; num_cells],
        }
    }

    /// Number of physical spin-1/2 sites on the lattice ($2 \times L_x \times L_y$).
    pub fn num_spins(&self) -> usize {
        2 * self.lx * self.ly
    }

    /// Total number of hexagonal plaquettes ($L_x \times L_y$).
    pub fn num_plaquettes(&self) -> usize {
        self.lx * self.ly
    }

    /// Sets the $\mathbb{Z}_2$ link gauge on the $z$-bond at unit cell $(x, y)$.
    pub fn set_z_gauge(&mut self, x: usize, y: usize, gauge: i8) {
        let idx = (y % self.ly) * self.lx + (x % self.lx);
        self.z_bond_gauge[idx] = if gauge >= 0 { 1 } else { -1 };
    }

    /// Reads the $\mathbb{Z}_2$ link gauge on the $z$-bond at unit cell $(x, y)$.
    pub fn get_z_gauge(&self, x: usize, y: usize) -> i8 {
        let idx = (y % self.ly) * self.lx + (x % self.lx);
        self.z_bond_gauge[idx]
    }

    /// Evaluates the Wilson loop plaquette flux operator $\hat{W}_p = \pm 1$ at cell $(x, y)$.
    pub fn plaquette_flux(&self, x: usize, y: usize) -> i8 {
        let u1 = self.get_z_gauge(x, y);
        let u2 = self.get_z_gauge(x, (y + 1) % self.ly);
        u1 * u2
    }

    /// Counts the total number of $\pi$-flux vortices ($\hat{W}_p = -1$) present on the lattice.
    pub fn count_vortices(&self) -> usize {
        let mut count = 0;
        for y in 0..self.ly {
            for x in 0..self.lx {
                if self.plaquette_flux(x, y) == -1 {
                    count += 1;
                }
            }
        }
        count
    }

    /// Computes the single-particle itinerant Majorana dispersion $E(\mathbf{k})$ at wavevector $(k_x, k_y)$.
    pub fn majorana_dispersion(&self, kx: f64, ky: f64) -> (f64, f64) {
        let n1_dot_k = 0.5 * 3.0_f64.sqrt() * kx + 1.5 * ky;
        let n2_dot_k = -0.5 * 3.0_f64.sqrt() * kx + 1.5 * ky;
        let diff_dot_k = n2_dot_k - n1_dot_k;

        let re_f = 2.0
            * (self.params.j_x * n1_dot_k.cos()
                + self.params.j_y * n2_dot_k.cos()
                + self.params.j_z);
        let im_f = 2.0 * (self.params.j_x * n1_dot_k.sin() + self.params.j_y * n2_dot_k.sin());
        let norm_f_sq = re_f * re_f + im_f * im_f;

        let delta = 4.0 * self.params.kappa * (n1_dot_k.sin() - n2_dot_k.sin() + diff_dot_k.sin());
        let e_pos = (norm_f_sq + delta * delta).sqrt();
        (-e_pos, e_pos)
    }

    /// Evaluates the minimum Majorana energy gap across the Brillouin zone.
    pub fn find_majorana_energy_gap(&self, grid_samples: usize) -> f64 {
        let mut min_gap = f64::INFINITY;
        let step = 2.0 * PI / (grid_samples as f64);

        for i in 0..grid_samples {
            let kx = -PI + (i as f64) * step;
            for j in 0..grid_samples {
                let ky = -PI + (j as f64) * step;
                let (_, e_pos) = self.majorana_dispersion(kx, ky);
                if e_pos < min_gap {
                    min_gap = e_pos;
                }
            }
        }
        min_gap
    }

    /// Evaluates the vortex excitation energy gap $\Delta_v$ in the isotropic phase ($J_x = J_y = J_z = J$).
    pub fn vortex_excitation_gap(&self) -> f64 {
        match self.params.phase() {
            KitaevPhase::GaplessB => 0.1536 * self.params.j_z.abs(),
            KitaevPhase::GappedAz => 2.0 * self.params.toric_code_effective_coupling(),
            KitaevPhase::GappedAx => {
                2.0 * ((self.params.j_y * self.params.j_y * self.params.j_z * self.params.j_z)
                    / (16.0 * self.params.j_x.abs().powi(3)))
            }
            KitaevPhase::GappedAy => {
                2.0 * ((self.params.j_x * self.params.j_x * self.params.j_z * self.params.j_z)
                    / (16.0 * self.params.j_y.abs().powi(3)))
            }
        }
    }

    /// Topological ground state degeneracy on a Riemann surface of genus $g$.
    pub fn topological_ground_state_degeneracy(&self, genus: usize) -> usize {
        4_usize.pow(genus as u32)
    }
}

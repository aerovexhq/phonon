//! Molecular junction tight-binding Hamiltonian, chemical topology, and electrode coupling.
//!
//! Models:
//! 1. Hückel / tight-binding \(\pi\)-electron Hamiltonian for conjugated molecular networks.
//! 2. Para- vs Meta-benzene connectivity and cross-conjugated topologies.
//! 3. Dihedral conformation angle twisting: \(t(\theta) = t_0 \cos(\theta)\).
//! 4. Wide-Band Approximation (WBA) lead self-energies and level broadening (\(\Gamma_L, \Gamma_R\)).
//! 5. Electrostatic gating modulation shifting molecular on-site energies.

#![allow(clippy::needless_range_loop)]

use crate::quantum::Complex;

/// Standard molecular wire and junction topologies.
#[derive(Debug, Clone, PartialEq)]
pub enum MolecularGraphType {
    /// Linear conjugated polyene chain (e.g., polyacetylene, OPV)
    LinearChain { num_sites: usize },
    /// Para-substituted benzene ring (1,4-connectivity, constructive quantum interference)
    ParaBenzene,
    /// Meta-substituted benzene ring (1,3-connectivity, destructive quantum interference)
    MetaBenzene,
    /// Cross-conjugated anthraquinone / quinoid pathway
    CrossConjugatedAnthracene,
    /// Arbitrary user-defined molecular graph
    CustomGraph,
}

/// A molecular junction with contact electrodes and gate control.
#[derive(Debug, Clone)]
pub struct MolecularJunction {
    /// Topology classification
    pub topology: MolecularGraphType,
    /// Total number of atomic orbital sites N
    pub num_sites: usize,
    /// On-site atomic orbital energies \(\epsilon_i\) [eV]
    pub on_site_energies_ev: Vec<f64>,
    /// Real tight-binding hopping matrix \(t_{ij}\) [eV]
    pub hopping_matrix_ev: Vec<Vec<f64>>,
    /// Left lead contact site index (0-indexed)
    pub left_lead_site: usize,
    /// Right lead contact site index (0-indexed)
    pub right_lead_site: usize,
    /// Left lead coupling strength \(\gamma_L\) [eV]
    pub gamma_left_ev: f64,
    /// Right lead coupling strength \(\gamma_R\) [eV]
    pub gamma_right_ev: f64,
    /// Dihedral conformational twist angle [radians]
    pub dihedral_angle_rad: f64,
    /// Electrostatic gate efficiency factor (0.0 to 1.0)
    pub gate_coupling_factor: f64,
    /// Vibronic electron-phonon coupling strength [eV]
    pub vibronic_coupling_ev: f64,
    /// Characteristic vibrational phonon energy \(\hbar \omega_0\) [eV]
    pub phonon_energy_ev: f64,
}

impl MolecularJunction {
    /// Builds a Para-substituted benzene junction (1,4-connection).
    ///
    /// Exhibits constructive quantum interference around the Fermi level.
    pub fn para_benzene(gamma_ev: f64) -> Self {
        let num_sites = 6;
        let beta = -2.7; // Standard C-C pi hopping ~ -2.7 eV
        let mut h = vec![vec![0.0; num_sites]; num_sites];

        // 6-membered aromatic ring
        for i in 0..num_sites {
            let next = (i + 1) % num_sites;
            h[i][next] = beta;
            h[next][i] = beta;
        }

        Self {
            topology: MolecularGraphType::ParaBenzene,
            num_sites,
            on_site_energies_ev: vec![0.0; num_sites],
            hopping_matrix_ev: h,
            left_lead_site: 0,  // Atom 1
            right_lead_site: 3, // Atom 4 (para)
            gamma_left_ev: gamma_ev,
            gamma_right_ev: gamma_ev,
            dihedral_angle_rad: 0.0,
            gate_coupling_factor: 0.35,
            vibronic_coupling_ev: 0.08,
            phonon_energy_ev: 0.18, // 180 meV C=C stretch
        }
    }

    /// Builds a Meta-substituted benzene junction (1,3-connection).
    ///
    /// Exhibits destructive quantum interference with a transmission anti-resonance near E_F.
    pub fn meta_benzene(gamma_ev: f64) -> Self {
        let num_sites = 6;
        let beta = -2.7;
        let mut h = vec![vec![0.0; num_sites]; num_sites];

        for i in 0..num_sites {
            let next = (i + 1) % num_sites;
            h[i][next] = beta;
            h[next][i] = beta;
        }

        Self {
            topology: MolecularGraphType::MetaBenzene,
            num_sites,
            on_site_energies_ev: vec![0.0; num_sites],
            hopping_matrix_ev: h,
            left_lead_site: 0,  // Atom 1
            right_lead_site: 2, // Atom 3 (meta)
            gamma_left_ev: gamma_ev,
            gamma_right_ev: gamma_ev,
            dihedral_angle_rad: 0.0,
            gate_coupling_factor: 0.35,
            vibronic_coupling_ev: 0.08,
            phonon_energy_ev: 0.18,
        }
    }

    /// Builds a linear conjugated molecular wire (polyene chain).
    pub fn linear_chain(num_sites: usize, gamma_ev: f64) -> Self {
        assert!(num_sites >= 2);
        let beta = -2.8;
        let mut h = vec![vec![0.0; num_sites]; num_sites];

        for i in 0..num_sites - 1 {
            h[i][i + 1] = beta;
            h[i + 1][i] = beta;
        }

        Self {
            topology: MolecularGraphType::LinearChain { num_sites },
            num_sites,
            on_site_energies_ev: vec![0.0; num_sites],
            hopping_matrix_ev: h,
            left_lead_site: 0,
            right_lead_site: num_sites - 1,
            gamma_left_ev: gamma_ev,
            gamma_right_ev: gamma_ev,
            dihedral_angle_rad: 0.0,
            gate_coupling_factor: 0.40,
            vibronic_coupling_ev: 0.06,
            phonon_energy_ev: 0.16,
        }
    }

    /// Builds a cross-conjugated pathway with destructive quantum interference anti-resonance.
    pub fn cross_conjugated(gamma_ev: f64) -> Self {
        // Cross-conjugated quinoid structure: 6 sites
        // Backbone: 0 -> 1 -> 2 -> 3. Pendant sites: 4 on site 1, 5 on site 2.
        let num_sites = 6;
        let beta = -2.0;
        let beta_p = -1.5;
        let mut h = vec![vec![0.0; num_sites]; num_sites];

        // Central conjugated backbone
        h[0][1] = beta;
        h[1][0] = beta;
        h[1][2] = beta;
        h[2][1] = beta;
        h[2][3] = beta;
        h[3][2] = beta;

        // Cross-conjugated pendant carbonyl/radical sites
        h[1][4] = beta_p;
        h[4][1] = beta_p;
        h[2][5] = beta_p;
        h[5][2] = beta_p;

        Self {
            topology: MolecularGraphType::CrossConjugatedAnthracene,
            num_sites,
            on_site_energies_ev: vec![0.0; num_sites],
            hopping_matrix_ev: h,
            left_lead_site: 0,
            right_lead_site: 3,
            gamma_left_ev: gamma_ev,
            gamma_right_ev: gamma_ev,
            dihedral_angle_rad: 0.0,
            gate_coupling_factor: 0.65,
            vibronic_coupling_ev: 0.05,
            phonon_energy_ev: 0.18,
        }
    }

    /// Modulates the conformation dihedral angle \(\theta\).
    /// Hopping across twist bond scales as \(t(\theta) = t_0 \cos(\theta)\).
    pub fn set_dihedral_angle(&mut self, angle_rad: f64) {
        self.dihedral_angle_rad = angle_rad;
    }

    /// Applies electrostatic gate voltage \(V_g\) [V], shifting on-site orbital energies.
    pub fn effective_on_site_energies(&self, v_gate_v: f64) -> Vec<f64> {
        let delta_e = -self.gate_coupling_factor * v_gate_v;
        self.on_site_energies_ev
            .iter()
            .map(|&e| e + delta_e)
            .collect()
    }

    /// Evaluates Wide-Band Approximation (WBA) level broadening matrix \(\Gamma_L(E)\).
    pub fn gamma_left(&self) -> Vec<Vec<Complex>> {
        let mut g = vec![vec![Complex::ZERO; self.num_sites]; self.num_sites];
        g[self.left_lead_site][self.left_lead_site] = Complex::new(self.gamma_left_ev, 0.0);
        g
    }

    /// Evaluates Wide-Band Approximation (WBA) level broadening matrix \(\Gamma_R(E)\).
    pub fn gamma_right(&self) -> Vec<Vec<Complex>> {
        let mut g = vec![vec![Complex::ZERO; self.num_sites]; self.num_sites];
        g[self.right_lead_site][self.right_lead_site] = Complex::new(self.gamma_right_ev, 0.0);
        g
    }

    /// Assembles the effective complex matrix \([E \cdot I - H_M - \Sigma_L - \Sigma_R]\).
    pub fn assemble_green_matrix(&self, energy_ev: f64, v_gate_v: f64) -> Vec<Vec<Complex>> {
        let n = self.num_sites;
        let on_site = self.effective_on_site_energies(v_gate_v);
        let cos_theta = self.dihedral_angle_rad.cos();

        let mut a = vec![vec![Complex::ZERO; n]; n];

        for i in 0..n {
            for j in 0..n {
                let re_val;
                let mut im_val = 0.0;

                if i == j {
                    // E - epsilon_i
                    re_val = energy_ev - on_site[i];

                    // - Im(Sigma_L) = -Gamma_L / 2
                    if i == self.left_lead_site {
                        im_val += self.gamma_left_ev * 0.5;
                    }
                    // - Im(Sigma_R) = -Gamma_R / 2
                    if i == self.right_lead_site {
                        im_val += self.gamma_right_ev * 0.5;
                    }
                } else {
                    // - t_ij
                    let t_val = self.hopping_matrix_ev[i][j];
                    // Apply conformation twist to central bond if applicable
                    let t_eff = if (i == 0 && j == 1) || (i == 1 && j == 0) {
                        t_val * cos_theta
                    } else {
                        t_val
                    };
                    re_val = -t_eff;
                }

                a[i][j] = Complex::new(re_val, im_val);
            }
        }

        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_para_vs_meta_benzene_junction_assembly() {
        let para = MolecularJunction::para_benzene(0.5);
        let meta = MolecularJunction::meta_benzene(0.5);

        assert_eq!(para.num_sites, 6);
        assert_eq!(meta.num_sites, 6);
        assert_eq!(para.left_lead_site, 0);
        assert_eq!(para.right_lead_site, 3);
        assert_eq!(meta.left_lead_site, 0);
        assert_eq!(meta.right_lead_site, 2);

        let g_mat = para.assemble_green_matrix(0.0, 0.0);
        assert_eq!(g_mat.len(), 6);
        // Diagonal element for lead site 0 has imaginary broadening Gamma / 2
        assert!((g_mat[0][0].im - 0.25).abs() < 1e-6);
    }
}

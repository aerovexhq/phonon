//! Crystallographic lattice systems, thermal expansion, and atomic density formulations.

use phonon_core::constants::T_REF;

/// Standard semiconductor crystal structure classifications.
#[derive(Debug, Clone, PartialEq)]
pub enum CrystalStructure {
    /// Diamond cubic (e.g. Silicon, Germanium) with 8 atoms per unit cell.
    DiamondCubic { lattice_constant_a: f64 },
    /// Zincblende (e.g. GaAs, InP, InAs) with 4 formula units (8 atoms) per face-centered cubic cell.
    Zincblende { lattice_constant_a: f64 },
    /// Wurtzite hexagonal (e.g. GaN, AlN, InN) with 4 atoms per hexagonal unit cell.
    Wurtzite {
        lattice_constant_a: f64,
        lattice_constant_c: f64,
    },
    /// Hexagonal 4H polytype (e.g. 4H-SiC) with 8 atoms per unit cell.
    Hexagonal4H {
        lattice_constant_a: f64,
        lattice_constant_c: f64,
    },
    /// Custom user-defined crystallographic system.
    Custom {
        name: String,
        lattice_constant_a: f64,
        basis_atoms: usize,
        unit_cell_volume: f64,
    },
}

impl CrystalStructure {
    /// Returns the primary lattice constant $a_0$ in meters at reference temperature (300.15 K).
    pub fn lattice_constant_a_ref(&self) -> f64 {
        match self {
            Self::DiamondCubic { lattice_constant_a } => *lattice_constant_a,
            Self::Zincblende { lattice_constant_a } => *lattice_constant_a,
            Self::Wurtzite {
                lattice_constant_a, ..
            } => *lattice_constant_a,
            Self::Hexagonal4H {
                lattice_constant_a, ..
            } => *lattice_constant_a,
            Self::Custom {
                lattice_constant_a, ..
            } => *lattice_constant_a,
        }
    }

    /// Returns the number of basis atoms residing within a single unit cell.
    pub fn atoms_per_unit_cell(&self) -> usize {
        match self {
            Self::DiamondCubic { .. } => 8,
            Self::Zincblende { .. } => 8,
            Self::Wurtzite { .. } => 4,
            Self::Hexagonal4H { .. } => 8,
            Self::Custom { basis_atoms, .. } => *basis_atoms,
        }
    }
}

/// Physical chemistry crystallographic parameters of a semiconductor crystal.
#[derive(Debug, Clone, PartialEq)]
pub struct Crystallography {
    /// Crystal symmetry and lattice type.
    pub structure: CrystalStructure,
    /// Linear thermal expansion coefficient $\alpha_L$ in $\text{K}^{-1}$.
    pub thermal_expansion_coeff: f64,
    /// Mean molar atomic mass in $\text{g/mol}$ (or atomic mass units $\text{amu}$).
    pub molar_mass_g_mol: f64,
    /// Mass density at reference temperature in $\text{kg/m}^3$.
    pub density_300k: f64,
    /// Covalent radius in meters.
    pub covalent_radius_m: f64,
}

impl Crystallography {
    /// Computes the temperature-dependent lattice constant $a(T)$ in meters:
    /// $$a(T) = a_0 \left[1 + \alpha_L (T - T_0)\right]$$
    pub fn lattice_constant_a(&self, temp_k: f64) -> f64 {
        let a0 = self.structure.lattice_constant_a_ref();
        a0 * (1.0 + self.thermal_expansion_coeff * (temp_k - T_REF))
    }

    /// Computes the secondary lattice constant $c(T)$ in meters for hexagonal / wurtzite crystals.
    pub fn lattice_constant_c(&self, temp_k: f64) -> Option<f64> {
        match self.structure {
            CrystalStructure::Wurtzite {
                lattice_constant_c, ..
            }
            | CrystalStructure::Hexagonal4H {
                lattice_constant_c, ..
            } => Some(lattice_constant_c * (1.0 + self.thermal_expansion_coeff * (temp_k - T_REF))),
            _ => None,
        }
    }

    /// Computes the temperature-dependent unit cell volume $\Omega_{cell}(T)$ in $\text{m}^3$.
    pub fn unit_cell_volume(&self, temp_k: f64) -> f64 {
        let a = self.lattice_constant_a(temp_k);
        match self.structure {
            CrystalStructure::DiamondCubic { .. } | CrystalStructure::Zincblende { .. } => {
                a * a * a
            }
            CrystalStructure::Wurtzite { .. } | CrystalStructure::Hexagonal4H { .. } => {
                let c = self.lattice_constant_c(temp_k).unwrap_or(a * 1.633);
                // Hexagonal prism volume: V = (sqrt(3)/2) * a^2 * c
                0.5 * 3.0f64.sqrt() * a * a * c
            }
            CrystalStructure::Custom {
                unit_cell_volume, ..
            } => {
                let scale = 1.0 + 3.0 * self.thermal_expansion_coeff * (temp_k - T_REF);
                unit_cell_volume * scale
            }
        }
    }

    /// Computes the atomic density $N_{atom}(T)$ in atoms per cubic meter ($\text{m}^{-3}$):
    /// $$N_{atom}(T) = \frac{N_{basis}}{\Omega_{cell}(T)}$$
    pub fn atomic_density(&self, temp_k: f64) -> f64 {
        let n_atoms = self.structure.atoms_per_unit_cell() as f64;
        let vol = self.unit_cell_volume(temp_k);
        n_atoms / vol.max(1e-35)
    }

    /// Computes the temperature-dependent mass density $\rho(T)$ in $\text{kg/m}^3$:
    /// $$\rho(T) = \rho_{300} \cdot \frac{\Omega_{cell}(300)}{\Omega_{cell}(T)}$$
    pub fn mass_density(&self, temp_k: f64) -> f64 {
        let vol_t = self.unit_cell_volume(temp_k);
        let vol_300 = self.unit_cell_volume(T_REF);
        self.density_300k * (vol_300 / vol_t.max(1e-35))
    }

    // =========================================================================
    // Standard Semiconductor Presets
    // =========================================================================

    /// Silicon ($Si$): Diamond cubic, $a_0 = 5.43102 \text{ \AA}$, $\rho = 2329 \text{ kg/m}^3$.
    pub fn silicon() -> Self {
        Self {
            structure: CrystalStructure::DiamondCubic {
                lattice_constant_a: 5.43102e-10,
            },
            thermal_expansion_coeff: 2.6e-6,
            molar_mass_g_mol: 28.0855,
            density_300k: 2329.0,
            covalent_radius_m: 1.11e-10,
        }
    }

    /// Germanium ($Ge$): Diamond cubic, $a_0 = 5.6575 \text{ \AA}$, $\rho = 5323 \text{ kg/m}^3$.
    pub fn germanium() -> Self {
        Self {
            structure: CrystalStructure::DiamondCubic {
                lattice_constant_a: 5.6575e-10,
            },
            thermal_expansion_coeff: 5.9e-6,
            molar_mass_g_mol: 72.630,
            density_300k: 5323.0,
            covalent_radius_m: 1.22e-10,
        }
    }

    /// Gallium Arsenide ($GaAs$): Zincblende, $a_0 = 5.65325 \text{ \AA}$, $\rho = 5317 \text{ kg/m}^3$.
    pub fn gallium_arsenide() -> Self {
        Self {
            structure: CrystalStructure::Zincblende {
                lattice_constant_a: 5.65325e-10,
            },
            thermal_expansion_coeff: 5.73e-6,
            molar_mass_g_mol: 144.645,
            density_300k: 5317.0,
            covalent_radius_m: 1.25e-10,
        }
    }

    /// Gallium Nitride ($GaN$): Wurtzite hexagonal, $a_0 = 3.189 \text{ \AA}, c_0 = 5.185 \text{ \AA}$, $\rho = 6150 \text{ kg/m}^3$.
    pub fn gallium_nitride() -> Self {
        Self {
            structure: CrystalStructure::Wurtzite {
                lattice_constant_a: 3.189e-10,
                lattice_constant_c: 5.185e-10,
            },
            thermal_expansion_coeff: 5.59e-6,
            molar_mass_g_mol: 83.728,
            density_300k: 6150.0,
            covalent_radius_m: 1.10e-10,
        }
    }

    /// Silicon Carbide ($4H-SiC$): Hexagonal 4H polytype, $a_0 = 3.073 \text{ \AA}, c_0 = 10.053 \text{ \AA}$, $\rho = 3210 \text{ kg/m}^3$.
    pub fn silicon_carbide_4h() -> Self {
        Self {
            structure: CrystalStructure::Hexagonal4H {
                lattice_constant_a: 3.073e-10,
                lattice_constant_c: 1.0053e-9,
            },
            thermal_expansion_coeff: 4.3e-6,
            molar_mass_g_mol: 40.096,
            density_300k: 3210.0,
            covalent_radius_m: 1.06e-10,
        }
    }

    /// Indium Phosphide ($InP$): Zincblende, $a_0 = 5.8687 \text{ \AA}$, $\rho = 4810 \text{ kg/m}^3$.
    pub fn indium_phosphide() -> Self {
        Self {
            structure: CrystalStructure::Zincblende {
                lattice_constant_a: 5.8687e-10,
            },
            thermal_expansion_coeff: 4.6e-6,
            molar_mass_g_mol: 145.792,
            density_300k: 4810.0,
            covalent_radius_m: 1.35e-10,
        }
    }
}

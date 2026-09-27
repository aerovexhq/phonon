//! 3D Elastodynamics, Voigt tensor formulations, and piezoelectric material models.
//!
//! Formulates coupled elastodynamic Cauchy momentum equations and Maxwell electrostatics:
//! $$\mathbf{T} = \mathbf{C}^E \mathbf{S} - \mathbf{e}^T \mathbf{E}$$
//! $$\mathbf{D} = \mathbf{e} \mathbf{S} + \boldsymbol{\epsilon}^S \mathbf{E}$$
//! in IEEE-standard Voigt notation.

/// Vacuum permittivity constant $\epsilon_0 \approx 8.8541878128 \times 10^{-12} \text{ F/m}$.
pub const EPSILON_0: f64 = 8.854_187_812_8e-12;

/// 6-component strain vector in Voigt notation:
/// $\mathbf{S} = [S_1, S_2, S_3, S_4, S_5, S_6]^T = [\varepsilon_{xx}, \varepsilon_{yy}, \varepsilon_{zz}, 2\varepsilon_{yz}, 2\varepsilon_{xz}, 2\varepsilon_{xy}]^T$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoigtStrain {
    pub s: [f64; 6],
}

impl VoigtStrain {
    pub const ZERO: Self = Self { s: [0.0; 6] };

    #[inline]
    pub fn new(s1: f64, s2: f64, s3: f64, s4: f64, s5: f64, s6: f64) -> Self {
        Self {
            s: [s1, s2, s3, s4, s5, s6],
        }
    }

    #[inline]
    pub fn from_slice(slice: &[f64]) -> Self {
        let mut s = [0.0; 6];
        s.copy_from_slice(&slice[..6]);
        Self { s }
    }
}

/// 6-component stress vector in Voigt notation:
/// $\mathbf{T} = [T_1, T_2, T_3, T_4, T_5, T_6]^T = [\sigma_{xx}, \sigma_{yy}, \sigma_{zz}, \sigma_{yz}, \sigma_{xz}, \sigma_{xy}]^T$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoigtStress {
    pub t: [f64; 6],
}

impl VoigtStress {
    pub const ZERO: Self = Self { t: [0.0; 6] };

    #[inline]
    pub fn new(t1: f64, t2: f64, t3: f64, t4: f64, t5: f64, t6: f64) -> Self {
        Self {
            t: [t1, t2, t3, t4, t5, t6],
        }
    }
}

/// 3-component electric field vector $\mathbf{E} = [E_x, E_y, E_z]^T$ in $\text{V/m}$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElectricField {
    pub e: [f64; 3],
}

impl ElectricField {
    pub const ZERO: Self = Self { e: [0.0; 3] };

    #[inline]
    pub fn new(ex: f64, ey: f64, ez: f64) -> Self {
        Self { e: [ex, ey, ez] }
    }
}

/// 3-component electric displacement vector $\mathbf{D} = [D_x, D_y, D_z]^T$ in $\text{C/m}^2$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElectricDisplacement {
    pub d: [f64; 3],
}

impl ElectricDisplacement {
    pub const ZERO: Self = Self { d: [0.0; 3] };

    #[inline]
    pub fn new(dx: f64, dy: f64, dz: f64) -> Self {
        Self { d: [dx, dy, dz] }
    }
}

/// 3D Piezoelectric material properties containing elastic, piezoelectric, and permittivity tensors.
#[derive(Debug, Clone, PartialEq)]
pub struct PiezoelectricMaterial {
    pub name: String,
    /// Mass density $\rho$ in $\text{kg/m}^3$.
    pub density: f64,
    /// $6 \times 6$ Elastic stiffness tensor $\mathbf{C}^E$ at constant electric field ($\text{N/m}^2$ or $\text{Pa}$).
    pub c_e: [[f64; 6]; 6],
    /// $3 \times 6$ Piezoelectric coupling tensor $\mathbf{e}$ in $\text{C/m}^2$.
    /// Rows correspond to electric field axes $(x, y, z)$; columns correspond to Voigt stress/strain indices $(1..6)$.
    pub e_piezo: [[f64; 6]; 3],
    /// $3 \times 3$ Dielectric permittivity tensor $\boldsymbol{\epsilon}^S$ at constant strain ($\text{F/m}$).
    pub epsilon_s: [[f64; 3]; 3],
}

impl PiezoelectricMaterial {
    /// Evaluates the mechanical stress $\mathbf{T} = \mathbf{C}^E \mathbf{S} - \mathbf{e}^T \mathbf{E}$.
    pub fn compute_stress(&self, strain: &VoigtStrain, ef: &ElectricField) -> VoigtStress {
        let mut t = [0.0; 6];
        for (i, t_val) in t.iter_mut().enumerate() {
            let mut sum_mech = 0.0;
            for j in 0..6 {
                sum_mech += self.c_e[i][j] * strain.s[j];
            }
            let mut sum_piezo = 0.0;
            for k in 0..3 {
                // e^T has entries (e^T)_{i, k} = e_{k, i}
                sum_piezo += self.e_piezo[k][i] * ef.e[k];
            }
            *t_val = sum_mech - sum_piezo;
        }
        VoigtStress { t }
    }

    /// Evaluates the electric displacement $\mathbf{D} = \mathbf{e} \mathbf{S} + \boldsymbol{\epsilon}^S \mathbf{E}$.
    pub fn compute_displacement(
        &self,
        strain: &VoigtStrain,
        ef: &ElectricField,
    ) -> ElectricDisplacement {
        let mut d = [0.0; 3];
        for (k, d_val) in d.iter_mut().enumerate() {
            let mut sum_piezo = 0.0;
            for j in 0..6 {
                sum_piezo += self.e_piezo[k][j] * strain.s[j];
            }
            let mut sum_diel = 0.0;
            for m in 0..3 {
                sum_diel += self.epsilon_s[k][m] * ef.e[m];
            }
            *d_val = sum_piezo + sum_diel;
        }
        ElectricDisplacement { d }
    }

    /// Longitudinal bulk acoustic wave velocity along the principal axis (z/c-axis):
    /// $v_L = \sqrt{C_{33}^E / \rho}$ ($\text{m/s}$).
    pub fn longitudinal_velocity(&self) -> f64 {
        (self.c_e[2][2] / self.density).sqrt()
    }

    /// Transverse / shear acoustic wave velocity along the principal axis:
    /// $v_T = \sqrt{C_{44}^E / \rho}$ ($\text{m/s}$).
    pub fn shear_velocity(&self) -> f64 {
        (self.c_e[3][3] / self.density).sqrt()
    }

    /// Piezoelectrically stiffened longitudinal acoustic wave velocity:
    /// $v_{stiff} = \sqrt{\frac{C_{33}^E + e_{33}^2 / \epsilon_{33}^S}{\rho}}$ ($\text{m/s}$).
    pub fn stiffened_longitudinal_velocity(&self) -> f64 {
        let e33 = self.e_piezo[2][2];
        let eps33 = self.epsilon_s[2][2];
        let c_stiff = self.c_e[2][2] + (e33 * e33) / eps33;
        (c_stiff / self.density).sqrt()
    }

    /// Electromechanical coupling coefficient for thickness-extensional modes:
    /// $k_t^2 = \frac{e_{33}^2}{C_{33}^E \epsilon_{33}^S + e_{33}^2}$.
    pub fn electromechanical_coupling_kt2(&self) -> f64 {
        let e33 = self.e_piezo[2][2];
        let eps33 = self.epsilon_s[2][2];
        let c33 = self.c_e[2][2];
        let num = e33 * e33;
        let den = c33 * eps33 + num;
        if den > 1e-30 {
            num / den
        } else {
            0.0
        }
    }

    /// Rayleigh surface acoustic wave (SAW) velocity approximation:
    /// $v_R \approx v_T \frac{0.87 + 1.12 \nu}{1 + \nu}$, where $\nu$ is Poisson's ratio.
    pub fn rayleigh_saw_velocity(&self) -> f64 {
        let c11 = self.c_e[0][0];
        let c12 = self.c_e[0][1];
        let nu = c12 / (c11 + c12); // Poisson's ratio approximation
        let vt = self.shear_velocity();
        vt * (0.87 + 1.12 * nu) / (1.0 + nu)
    }

    /// Preset: Aluminium Nitride ($\text{AlN}$) wurtzite $6mm$.
    /// Renowned for high hypersonic velocity ($> 10,000 \text{ m/s}$) and high thermal conductivity.
    pub fn aln() -> Self {
        let density = 3260.0; // kg/m^3
        let mut c_e = [[0.0; 6]; 6];
        // Stiffness in Pa (GPa * 1e9)
        c_e[0][0] = 398.0e9; // c11
        c_e[1][1] = 398.0e9; // c22 = c11
        c_e[0][1] = 142.0e9; // c12
        c_e[1][0] = 142.0e9;
        c_e[0][2] = 112.0e9; // c13
        c_e[2][0] = 112.0e9;
        c_e[1][2] = 112.0e9; // c23 = c13
        c_e[2][1] = 112.0e9;
        c_e[2][2] = 373.0e9; // c33
        c_e[3][3] = 116.0e9; // c44
        c_e[4][4] = 116.0e9; // c55 = c44
        c_e[5][5] = 128.0e9; // c66 = (c11 - c12)/2 = (398 - 142)/2 = 128 GPa

        let mut e_piezo = [[0.0; 6]; 3];
        // e31 = e32 = -0.58 C/m^2
        e_piezo[2][0] = -0.58;
        e_piezo[2][1] = -0.58;
        // e33 = 1.55 C/m^2
        e_piezo[2][2] = 1.55;
        // e15 = e24 = -0.48 C/m^2
        e_piezo[0][4] = -0.48;
        e_piezo[1][3] = -0.48;

        let mut epsilon_s = [[0.0; 3]; 3];
        epsilon_s[0][0] = 9.0 * EPSILON_0;
        epsilon_s[1][1] = 9.0 * EPSILON_0;
        epsilon_s[2][2] = 10.7 * EPSILON_0;

        Self {
            name: "AlN".to_string(),
            density,
            c_e,
            e_piezo,
            epsilon_s,
        }
    }

    /// Preset: Lithium Niobate ($\text{LiNbO}_3$ $128^\circ$ Y-X cut).
    /// Exceptional electromechanical coupling for SAW and BAW devices.
    pub fn linbo3_128_yx() -> Self {
        let density = 4700.0;
        let mut c_e = [[0.0; 6]; 6];
        c_e[0][0] = 203.0e9;
        c_e[1][1] = 190.0e9;
        c_e[0][1] = 53.0e9;
        c_e[1][0] = 53.0e9;
        c_e[0][2] = 75.0e9;
        c_e[2][0] = 75.0e9;
        c_e[1][2] = 68.0e9;
        c_e[2][1] = 68.0e9;
        c_e[2][2] = 245.0e9;
        c_e[3][3] = 60.0e9;
        c_e[4][4] = 60.0e9;
        c_e[5][5] = 75.0e9;

        let mut e_piezo = [[0.0; 6]; 3];
        e_piezo[2][0] = 0.38;
        e_piezo[2][1] = 1.72;
        e_piezo[2][2] = 1.45;
        e_piezo[0][4] = 3.76;
        e_piezo[1][3] = 2.43;

        let mut epsilon_s = [[0.0; 3]; 3];
        epsilon_s[0][0] = 44.0 * EPSILON_0;
        epsilon_s[1][1] = 44.0 * EPSILON_0;
        epsilon_s[2][2] = 29.0 * EPSILON_0;

        Self {
            name: "LiNbO3-128YX".to_string(),
            density,
            c_e,
            e_piezo,
            epsilon_s,
        }
    }

    /// Preset: Gallium Nitride ($\text{GaN}$) wurtzite $6mm$.
    pub fn gan() -> Self {
        let density = 6150.0;
        let mut c_e = [[0.0; 6]; 6];
        c_e[0][0] = 390.0e9;
        c_e[1][1] = 390.0e9;
        c_e[0][1] = 145.0e9;
        c_e[1][0] = 145.0e9;
        c_e[0][2] = 106.0e9;
        c_e[2][0] = 106.0e9;
        c_e[1][2] = 106.0e9;
        c_e[2][1] = 106.0e9;
        c_e[2][2] = 398.0e9;
        c_e[3][3] = 105.0e9;
        c_e[4][4] = 105.0e9;
        c_e[5][5] = 122.5e9;

        let mut e_piezo = [[0.0; 6]; 3];
        e_piezo[2][0] = -0.49;
        e_piezo[2][1] = -0.49;
        e_piezo[2][2] = 0.73;
        e_piezo[0][4] = -0.30;
        e_piezo[1][3] = -0.30;

        let mut epsilon_s = [[0.0; 3]; 3];
        epsilon_s[0][0] = 9.5 * EPSILON_0;
        epsilon_s[1][1] = 9.5 * EPSILON_0;
        epsilon_s[2][2] = 10.4 * EPSILON_0;

        Self {
            name: "GaN".to_string(),
            density,
            c_e,
            e_piezo,
            epsilon_s,
        }
    }

    /// Preset: Diamond (CVD single crystal). Non-piezoelectric ultra-high-velocity acoustic substrate ($v_L \approx 17,500 \text{ m/s}$).
    pub fn diamond() -> Self {
        let density = 3515.0;
        let mut c_e = [[0.0; 6]; 6];
        c_e[0][0] = 1076.0e9;
        c_e[1][1] = 1076.0e9;
        c_e[2][2] = 1076.0e9;
        c_e[0][1] = 125.0e9;
        c_e[1][0] = 125.0e9;
        c_e[0][2] = 125.0e9;
        c_e[2][0] = 125.0e9;
        c_e[1][2] = 125.0e9;
        c_e[2][1] = 125.0e9;
        c_e[3][3] = 577.0e9;
        c_e[4][4] = 577.0e9;
        c_e[5][5] = 577.0e9;

        let e_piezo = [[0.0; 6]; 3]; // Zero piezoelectric coupling
        let mut epsilon_s = [[0.0; 3]; 3];
        epsilon_s[0][0] = 5.7 * EPSILON_0;
        epsilon_s[1][1] = 5.7 * EPSILON_0;
        epsilon_s[2][2] = 5.7 * EPSILON_0;

        Self {
            name: "Diamond".to_string(),
            density,
            c_e,
            e_piezo,
            epsilon_s,
        }
    }

    /// Preset: Silicon ($\text{Si}$ [100]). Standard semiconductor substrate for composite acoustic layers.
    pub fn silicon() -> Self {
        let density = 2330.0;
        let mut c_e = [[0.0; 6]; 6];
        c_e[0][0] = 165.7e9;
        c_e[1][1] = 165.7e9;
        c_e[2][2] = 165.7e9;
        c_e[0][1] = 63.9e9;
        c_e[1][0] = 63.9e9;
        c_e[0][2] = 63.9e9;
        c_e[2][0] = 63.9e9;
        c_e[1][2] = 63.9e9;
        c_e[2][1] = 63.9e9;
        c_e[3][3] = 79.6e9;
        c_e[4][4] = 79.6e9;
        c_e[5][5] = 79.6e9;

        let e_piezo = [[0.0; 6]; 3];
        let mut epsilon_s = [[0.0; 3]; 3];
        epsilon_s[0][0] = 11.7 * EPSILON_0;
        epsilon_s[1][1] = 11.7 * EPSILON_0;
        epsilon_s[2][2] = 11.7 * EPSILON_0;

        Self {
            name: "Silicon".to_string(),
            density,
            c_e,
            e_piezo,
            epsilon_s,
        }
    }
}

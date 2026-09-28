//! Topological Valley Acoustic Metamaterials & Chiral Edge States.
//!
//! Formulates honeycomb acoustic metamaterials with broken spatial inversion
//! symmetry, deterministic valley bandgap opening, valley Chern numbers,
//! and topologically protected backscattering-immune chiral phononic edge modes.

use std::f64::consts::PI;

/// Honeycomb acoustic metamaterial lattice configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct HoneycombAcousticLattice {
    /// Lattice constant $a$ in meters (e.g. 20 mm).
    pub lattice_constant_m: f64,
    /// Sublattice A pillar diameter $d_A$ in meters.
    pub pillar_diameter_a_m: f64,
    /// Sublattice B pillar diameter $d_B$ in meters.
    pub pillar_diameter_b_m: f64,
    /// Background medium speed of sound $c_0$ in m/s (default 343.0 m/s for air).
    pub sound_speed_m_per_s: f64,
    /// Background medium mass density $\rho_0$ in kg/m^3 (default 1.225 kg/m^3 for air).
    pub density_kg_per_m3: f64,
}

impl Default for HoneycombAcousticLattice {
    fn default() -> Self {
        Self {
            lattice_constant_m: 0.020,
            pillar_diameter_a_m: 0.009,
            pillar_diameter_b_m: 0.007,
            sound_speed_m_per_s: 343.0,
            density_kg_per_m3: 1.225,
        }
    }
}

impl HoneycombAcousticLattice {
    /// Creates a new honeycomb acoustic metamaterial lattice.
    pub fn new(
        lattice_constant_m: f64,
        pillar_diameter_a_m: f64,
        pillar_diameter_b_m: f64,
        sound_speed_m_per_s: f64,
        density_kg_per_m3: f64,
    ) -> Self {
        Self {
            lattice_constant_m,
            pillar_diameter_a_m,
            pillar_diameter_b_m,
            sound_speed_m_per_s,
            density_kg_per_m3,
        }
    }

    /// Creates an inversion-symmetric lattice where $d_A = d_B = d_0$.
    pub fn symmetric(lattice_constant_m: f64, pillar_diameter_m: f64) -> Self {
        Self {
            lattice_constant_m,
            pillar_diameter_a_m: pillar_diameter_m,
            pillar_diameter_b_m: pillar_diameter_m,
            sound_speed_m_per_s: 343.0,
            density_kg_per_m3: 1.225,
        }
    }

    /// Background bulk modulus $K_0 = \rho_0 c_0^2$ in Pascals.
    #[inline]
    pub fn background_bulk_modulus_pa(&self) -> f64 {
        self.density_kg_per_m3 * self.sound_speed_m_per_s.powi(2)
    }

    /// Sublattice diameter asymmetry parameter:
    /// $\delta_A = \frac{d_A - d_B}{a}$.
    #[inline]
    pub fn asymmetry_parameter(&self) -> f64 {
        (self.pillar_diameter_a_m - self.pillar_diameter_b_m) / self.lattice_constant_m
    }

    /// Dirac degeneracy frequency $\omega_0 = \frac{4\pi c_0}{3\sqrt{3} a}$ in rad/s.
    #[inline]
    pub fn dirac_frequency_rad_per_s(&self) -> f64 {
        (4.0 * PI * self.sound_speed_m_per_s) / (3.0 * 3.0_f64.sqrt() * self.lattice_constant_m)
    }

    /// Dirac degeneracy frequency $f_0 = \frac{\omega_0}{2\pi}$ in Hertz.
    #[inline]
    pub fn dirac_frequency_hz(&self) -> f64 {
        self.dirac_frequency_rad_per_s() / (2.0 * PI)
    }

    /// Acoustic Dirac velocity $v_D = \frac{\sqrt{3}}{2} c_0$ in m/s.
    #[inline]
    pub fn dirac_velocity_m_per_s(&self) -> f64 {
        (3.0_f64.sqrt() / 2.0) * self.sound_speed_m_per_s
    }

    /// Valley bandgap opening $\Delta\omega_v$ in rad/s:
    /// $\Delta\omega_v = \sqrt{3} \omega_0 |\delta_A|$.
    #[inline]
    pub fn valley_gap_rad_per_s(&self) -> f64 {
        3.0_f64.sqrt() * self.dirac_frequency_rad_per_s() * self.asymmetry_parameter().abs()
    }

    /// Valley bandgap opening in Hertz.
    #[inline]
    pub fn valley_gap_hz(&self) -> f64 {
        self.valley_gap_rad_per_s() / (2.0 * PI)
    }

    /// Normalized valley gap ratio $\Delta\omega_v / \omega_0$.
    #[inline]
    pub fn valley_gap_ratio(&self) -> f64 {
        if self.dirac_frequency_rad_per_s() > 0.0 {
            self.valley_gap_rad_per_s() / self.dirac_frequency_rad_per_s()
        } else {
            0.0
        }
    }

    /// Lower valley edge frequency $\omega_{lower} = \omega_0 - \frac{\Delta\omega_v}{2}$ in rad/s.
    #[inline]
    pub fn lower_valley_edge_rad_per_s(&self) -> f64 {
        self.dirac_frequency_rad_per_s() - 0.5 * self.valley_gap_rad_per_s()
    }

    /// Upper valley edge frequency $\omega_{upper} = \omega_0 + \frac{\Delta\omega_v}{2}$ in rad/s.
    #[inline]
    pub fn upper_valley_edge_rad_per_s(&self) -> f64 {
        self.dirac_frequency_rad_per_s() + 0.5 * self.valley_gap_rad_per_s()
    }

    /// Valley Chern number $\mathcal{C}_v = \text{sgn}(\delta_A) \in \{-1, 0, +1\}$.
    #[inline]
    pub fn valley_chern_number(&self) -> i32 {
        let delta = self.asymmetry_parameter();
        if delta > 1e-6 {
            1
        } else if delta < -1e-6 {
            -1
        } else {
            0
        }
    }

    /// Berry curvature near valley $\tau \in \{+1, -1\}$ at momentum $\mathbf{q} = (q_x, q_y)$ from the valley point:
    /// $\Omega_\tau(\mathbf{q}) = -\frac{1}{2} \tau \frac{v_D^2 m_v}{(v_D^2 q^2 + m_v^2)^{3/2}}$,
    /// where $m_v = \frac{1}{2} \Delta\omega_v \text{sgn}(\delta_A)$.
    pub fn berry_curvature(&self, valley_tau: i32, qx: f64, qy: f64) -> f64 {
        let delta = self.asymmetry_parameter();
        if delta.abs() < 1e-9 {
            return 0.0;
        }
        let v_d = self.dirac_velocity_m_per_s();
        let m_v = 0.5 * self.valley_gap_rad_per_s() * delta.signum();
        let q_sq = qx * qx + qy * qy;
        let denom = (v_d.powi(2) * q_sq + m_v.powi(2)).powf(1.5);
        if denom < 1e-30 {
            0.0
        } else {
            -0.5 * (valley_tau as f64) * (v_d.powi(2) * m_v) / denom
        }
    }

    /// Characteristic transverse decay length $\xi = \frac{2 v_D}{\Delta\omega_v}$ in meters
    /// for domain wall topological edge modes.
    pub fn edge_state_decay_length_m(&self) -> f64 {
        let gap = self.valley_gap_rad_per_s();
        if gap > 1e-6 {
            (2.0 * self.dirac_velocity_m_per_s()) / gap
        } else {
            f64::INFINITY
        }
    }

    /// Power transmission through a sharp waveguide bend of angle $\theta$ (in radians).
    /// For topological edge states protected against backscattering,
    /// $T_{bend} \ge 0.90$ across sharp $60^\circ$ and $120^\circ$ turns.
    pub fn corner_transmission(&self, bend_angle_rad: f64) -> f64 {
        let delta = self.asymmetry_parameter().abs();
        if delta < 1e-6 {
            // Trivial or non-existent gap has no topological protection
            return (bend_angle_rad.cos().abs()).max(0.1);
        }
        // Topologically protected suppression of backscattering:
        // Reflection scales down inversely with the valley bandgap robustness
        let normalized_angle = (bend_angle_rad / PI).abs();
        let reflection = 0.08 * normalized_angle.powi(2) / (1.0 + 10.0 * delta);
        (1.0 - reflection).clamp(0.0, 1.0)
    }

    /// Insertion loss in decibels: $\mathcal{L}_{dB} = -10 \log_{10}(T)$.
    pub fn insertion_loss_db(&self, transmission: f64) -> f64 {
        let t = transmission.clamp(1e-12, 1.0);
        -10.0 * t.log10()
    }
}

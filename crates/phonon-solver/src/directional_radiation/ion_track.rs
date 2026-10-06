#![deny(unsafe_code)]

//! Directional Heavy Ion Species, Incident Trajectory & Bragg Peak LET Ionization Model.
//!
//! Models cosmic ray heavy ions (Protons to Iron Fe-56) with trajectory vector
//! k_rad = (sin theta cos phi, sin theta sin phi, cos theta), calculating depth-dependent
//! Bragg peak Linear Energy Transfer (LET(z)), radial electron-hole plasma column
//! deposition n_eh(r, z), and sensitive volume deposited charge Q_dep.

use std::f64::consts::PI;

/// Cosmic ray heavy ion species.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeavyIonSpecies {
    /// Galactic/solar proton (Z=1, A=1).
    Proton,
    /// Alpha particle / Helium nucleus (Z=2, A=4).
    Alpha,
    /// Carbon ion (Z=6, A=12).
    Carbon,
    /// Silicon ion (Z=14, A=28).
    Silicon,
    /// Galactic cosmic ray Iron ion (Z=26, A=56).
    IronFe56,
}

impl HeavyIonSpecies {
    /// Atomic number Z (nuclear charge).
    pub fn atomic_number(&self) -> u32 {
        match self {
            Self::Proton => 1,
            Self::Alpha => 2,
            Self::Carbon => 6,
            Self::Silicon => 14,
            Self::IronFe56 => 26,
        }
    }

    /// Mass number A in atomic mass units (amu).
    pub fn mass_number(&self) -> u32 {
        match self {
            Self::Proton => 1,
            Self::Alpha => 4,
            Self::Carbon => 12,
            Self::Silicon => 28,
            Self::IronFe56 => 56,
        }
    }

    /// Name identifier.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Proton => "Proton (1H)",
            Self::Alpha => "Alpha (4He)",
            Self::Carbon => "Carbon (12C)",
            Self::Silicon => "Silicon (28Si)",
            Self::IronFe56 => "Iron (56Fe)",
        }
    }

    /// Peak stopping power LET in silicon in MeV*cm^2/mg.
    pub fn peak_stopping_power_let(&self) -> f64 {
        let _z = self.atomic_number() as f64;
        // Bethe-Bloch Bragg peak scales approximately with Z^2
        match self {
            Self::Proton => 1.45,
            Self::Alpha => 5.80,
            Self::Carbon => 18.2,
            Self::Silicon => 42.5,
            Self::IronFe56 => 78.4,
        }
    }

    /// Core plasma ionization radius r_0 in nanometers.
    pub fn core_ionization_radius_nm(&self) -> f64 {
        match self {
            Self::Proton => 15.0,
            Self::Alpha => 25.0,
            Self::Carbon => 45.0,
            Self::Silicon => 70.0,
            Self::IronFe56 => 110.0,
        }
    }
}

/// Incident trajectory angles and energy for the cosmic heavy ion.
#[derive(Debug, Clone)]
pub struct IncidentTrajectory {
    /// Polar zenith angle theta in radians in [0, pi/2] (0 = normal incident, pi/2 = grazing).
    pub theta_rad: f64,
    /// Azimuthal angle phi in radians in [0, 2*pi].
    pub phi_rad: f64,
    /// Kinetic energy per nucleon in MeV/nucleon (e.g. 10.0 to 1000.0 MeV/nuc).
    pub energy_mev_per_nuc: f64,
    /// Ion species.
    pub species: HeavyIonSpecies,
}

impl Default for IncidentTrajectory {
    fn default() -> Self {
        Self {
            theta_rad: 30.0 * PI / 180.0, // 30 degrees oblique
            phi_rad: 45.0 * PI / 180.0,   // 45 degrees azimuth
            energy_mev_per_nuc: 150.0,    // 150 MeV/nuc GCR baseline
            species: HeavyIonSpecies::IronFe56,
        }
    }
}

impl IncidentTrajectory {
    /// Unit direction vector k_rad = (kx, ky, kz) pointing along trajectory into target die.
    pub fn direction_vector(&self) -> [f64; 3] {
        let st = self.theta_rad.sin();
        let ct = self.theta_rad.cos();
        let sp = self.phi_rad.sin();
        let cp = self.phi_rad.cos();

        [st * cp, st * sp, ct]
    }

    /// Effective path length elongation factor sec(theta) = 1 / cos(theta).
    pub fn path_elongation_factor(&self) -> f64 {
        let ct = self.theta_rad.cos().max(0.05); // Clamp to prevent division by zero at 90 deg
        1.0 / ct
    }
}

/// Ionization track profile and charge collection metrics.
#[derive(Debug, Clone)]
pub struct IonTrackProfile {
    /// Silicon density in mg/cm^3 (2329.0 mg/cm^3 for silicon, equivalent to 2.329 g/cm^3).
    pub silicon_density_mg_cm3: f64,
    /// Electron-hole pair creation energy in silicon in eV (3.6 eV).
    pub eh_pair_energy_ev: f64,
    /// Elementary charge in Coulombs (1.602e-19 C).
    pub q_elem_coulombs: f64,
}

impl Default for IonTrackProfile {
    fn default() -> Self {
        Self {
            silicon_density_mg_cm3: 2329.0,
            eh_pair_energy_ev: 3.60,
            q_elem_coulombs: 1.602_176_634e-19,
        }
    }
}

impl IonTrackProfile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate Linear Energy Transfer LET(z) in MeV*cm^2/mg as a function of depth z in micrometers.
    ///
    /// Evaluates Bragg peak profile: LET increases towards end-of-range, then drops sharply.
    pub fn let_at_depth_um(&self, trajectory: &IncidentTrajectory, depth_um: f64) -> f64 {
        let peak_let = trajectory.species.peak_stopping_power_let();
        let elongation = trajectory.path_elongation_factor();
        let effective_depth = depth_um * elongation;

        // Projected range in silicon scales with energy and inversely with mass/Z^2
        let range_um = trajectory.energy_mev_per_nuc * 1.8;
        if effective_depth > range_um * 1.05 {
            return 0.0; // Stopped
        }

        // Modified Bohr-Bethe Bragg curve approximation
        let norm_depth = (effective_depth / range_um.max(1.0)).min(1.0);
        let bragg_factor = if norm_depth < 0.85 {
            0.45 + 0.55 * (norm_depth / 0.85).powf(0.6)
        } else {
            // Bragg peak maximum near end of range
            let peak_pos = 0.95;
            let width = 0.08;
            1.0 * (-((norm_depth - peak_pos) / width).powi(2)).exp()
        };

        (peak_let * bragg_factor * elongation).max(0.1)
    }

    /// Calculate linear charge deposition density dQ/dz in fC/um.
    ///
    /// Conversion: 1 MeV*cm^2/mg in silicon produces ~10.36 fC/um of electron-hole pairs.
    pub fn charge_density_fc_per_um(&self, trajectory: &IncidentTrajectory, depth_um: f64) -> f64 {
        let let_val = self.let_at_depth_um(trajectory, depth_um);
        // Formula: (LET in MeV*cm^2/mg) * (density 2.329 mg/cm^3) * 1e-4 cm/um / (3.6e-6 MeV/pair) * 1.602e-19 C * 1e15 fC/C
        let conversion_factor = 10.364; // fC/um per (MeV*cm^2/mg)
        let_val * conversion_factor
    }

    /// Calculate total deposited charge Q_dep in fC across depth interval [z_start_um, z_end_um].
    pub fn total_deposited_charge_fc(
        &self,
        trajectory: &IncidentTrajectory,
        z_start_um: f64,
        z_end_um: f64,
        steps: usize,
    ) -> f64 {
        let n = steps.max(4);
        let dz = (z_end_um - z_start_um) / (n as f64);
        let mut total_q = 0.0;

        for i in 0..n {
            let z = z_start_um + (i as f64 + 0.5) * dz;
            let dq_dz = self.charge_density_fc_per_um(trajectory, z);
            total_q += dq_dz * dz;
        }

        total_q
    }

    /// Calculate radial carrier concentration n_eh(r, z) in cm^-3 at radius r_nm and depth z_um.
    pub fn radial_carrier_density_cm3(
        &self,
        trajectory: &IncidentTrajectory,
        r_nm: f64,
        depth_um: f64,
    ) -> f64 {
        let dq_dz = self.charge_density_fc_per_um(trajectory, depth_um);
        let r0_nm = trajectory.species.core_ionization_radius_nm();

        // Convert dq_dz (fC/um) to pairs / cm:
        // pairs/cm = (dq_dz * 1e-15 / 1.602e-19) * 1e4 um/cm
        let pairs_per_cm = (dq_dz * 1e-15 / self.q_elem_coulombs) * 1e4;
        let r0_cm = r0_nm * 1e-7;
        let r_cm = r_nm * 1e-7;

        // Gaussian radial diffusion distribution
        let norm = 1.0 / (2.0 * PI * r0_cm * r0_cm);
        pairs_per_cm * norm * (-0.5 * (r_cm / r0_cm).powi(2)).exp()
    }
}

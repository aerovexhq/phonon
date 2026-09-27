//! Chemical dopant species, ionization energy, solid solubility, and carrier freeze-out statistics.

use super::bandstructure::Bandstructure;
use phonon_core::constants::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE};

/// Dopant polarity type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DopantType {
    Donor,
    Acceptor,
}

/// Chemical element classification of semiconductor dopants.
#[derive(Debug, Clone, PartialEq)]
pub enum DopantSpecies {
    // Group IV Silicon/Germanium donors
    Phosphorus,
    Arsenic,
    Antimony,
    // Group IV Silicon/Germanium acceptors
    Boron,
    Gallium,
    Indium,
    // Compound semiconductor dopants
    SiliconDonorInGaAs,
    CarbonAcceptorInGaAs,
    BerylliumAcceptorInGaAs,
    MagnesiumAcceptorInGaN,
    SiliconDonorInGaN,
    NitrogenDonorInSiC,
    AluminumAcceptorInSiC,
    // Custom chemical dopant
    Custom {
        name: String,
        dopant_type: DopantType,
        ionization_energy_ev: f64,
        ground_state_degeneracy: f64,
        solid_solubility_m3: f64,
    },
}

impl DopantSpecies {
    /// Returns the dopant polarity (Donor or Acceptor).
    pub fn dopant_type(&self) -> DopantType {
        match self {
            Self::Phosphorus
            | Self::Arsenic
            | Self::Antimony
            | Self::SiliconDonorInGaAs
            | Self::SiliconDonorInGaN
            | Self::NitrogenDonorInSiC => DopantType::Donor,

            Self::Boron
            | Self::Gallium
            | Self::Indium
            | Self::CarbonAcceptorInGaAs
            | Self::BerylliumAcceptorInGaAs
            | Self::MagnesiumAcceptorInGaN
            | Self::AluminumAcceptorInSiC => DopantType::Acceptor,

            Self::Custom { dopant_type, .. } => *dopant_type,
        }
    }

    /// Returns the ground-state activation / ionization energy in electron-volts ($eV$):
    /// $\Delta E_d = E_c - E_D$ for donors, $\Delta E_a = E_A - E_v$ for acceptors.
    pub fn ionization_energy_ev(&self) -> f64 {
        match self {
            // Silicon donors
            Self::Phosphorus => 0.045, // 45 meV
            Self::Arsenic => 0.054,    // 54 meV
            Self::Antimony => 0.039,   // 39 meV
            // Silicon acceptors
            Self::Boron => 0.045,   // 45 meV
            Self::Gallium => 0.072, // 72 meV
            Self::Indium => 0.160,  // 160 meV (deep acceptor)
            // GaAs dopants
            Self::SiliconDonorInGaAs => 0.0058, // 5.8 meV (shallow hydrogenic)
            Self::CarbonAcceptorInGaAs => 0.0260, // 26 meV
            Self::BerylliumAcceptorInGaAs => 0.0280, // 28 meV
            // GaN dopants
            Self::SiliconDonorInGaN => 0.020,      // 20 meV
            Self::MagnesiumAcceptorInGaN => 0.170, // 170 meV (causes strong freeze-out at 300K)
            // SiC dopants
            Self::NitrogenDonorInSiC => 0.060,    // 60 meV
            Self::AluminumAcceptorInSiC => 0.200, // 200 meV
            // Custom
            Self::Custom {
                ionization_energy_ev,
                ..
            } => *ionization_energy_ev,
        }
    }

    /// Returns the ground-state degeneracy factor:
    /// $g_D = 2$ for donors (spin-up / spin-down).
    /// $g_A = 4$ for acceptors (spin degeneracy + valence band heavy/light hole degeneracy).
    pub fn ground_state_degeneracy(&self) -> f64 {
        match self {
            Self::Custom {
                ground_state_degeneracy,
                ..
            } => *ground_state_degeneracy,
            _ => match self.dopant_type() {
                DopantType::Donor => 2.0,
                DopantType::Acceptor => 4.0,
            },
        }
    }

    /// Returns the chemical solid solubility limit in active atoms per cubic meter ($\text{m}^{-3}$).
    pub fn solid_solubility_m3(&self) -> f64 {
        match self {
            Self::Phosphorus => 1.3e27, // 1.3e21 cm^-3
            Self::Arsenic => 1.8e27,    // 1.8e21 cm^-3
            Self::Antimony => 7.0e25,   // 7.0e19 cm^-3
            Self::Boron => 6.0e26,      // 6.0e20 cm^-3
            Self::Gallium => 4.0e25,    // 4.0e19 cm^-3
            Self::Indium => 2.0e24,     // 2.0e18 cm^-3
            Self::SiliconDonorInGaAs => 1.0e25,
            Self::CarbonAcceptorInGaAs => 1.0e26,
            Self::BerylliumAcceptorInGaAs => 5.0e25,
            Self::SiliconDonorInGaN => 5.0e25,
            Self::MagnesiumAcceptorInGaN => 2.0e26,
            Self::NitrogenDonorInSiC => 2.0e25,
            Self::AluminumAcceptorInSiC => 1.0e26,
            Self::Custom {
                solid_solubility_m3,
                ..
            } => *solid_solubility_m3,
        }
    }

    /// Computes the electrically active dopant concentration $N_{active}$ in $\text{m}^{-3}$,
    /// accounting for precipitation and clustering beyond the chemical solid solubility limit:
    /// $$N_{active} = \min(N_{total}, N_{solubility})$$
    pub fn active_concentration(&self, total_doping_m3: f64) -> f64 {
        total_doping_m3.min(self.solid_solubility_m3()).max(0.0)
    }

    /// Computes the ionized carrier concentration $N_{ionized}(T)$ in $\text{m}^{-3}$
    /// accounting for incomplete ionization / carrier freeze-out:
    ///
    /// For Donors:
    /// $$\left( \frac{g_D}{N_c} e^{\Delta E_d / k_B T} \right) n^2 + n - N_D = 0$$
    ///
    /// For Acceptors:
    /// $$\left( \frac{g_A}{N_v} e^{\Delta E_a / k_B T} \right) p^2 + p - N_A = 0$$
    pub fn ionized_concentration(
        &self,
        bandstructure: &Bandstructure,
        total_doping_m3: f64,
        temp_k: f64,
    ) -> f64 {
        let n_active = self.active_concentration(total_doping_m3);
        if n_active <= 0.0 {
            return 0.0;
        }

        let t = temp_k.max(1.0);
        let g = self.ground_state_degeneracy();
        let delta_e_joules = self.ionization_energy_ev() * ELEMENTARY_CHARGE;
        let exp_arg = (delta_e_joules / (BOLTZMANN_CONSTANT * t)).clamp(0.0, 70.0);
        let exp_term = exp_arg.exp();

        let n_dos = match self.dopant_type() {
            DopantType::Donor => bandstructure.effective_nc(t),
            DopantType::Acceptor => bandstructure.effective_nv(t),
        };

        let coeff_a = (g / n_dos.max(1e10)) * exp_term;
        let coeff_b = 1.0;
        let coeff_c = -n_active;

        // Solve quadratic equation: a*x^2 + b*x + c = 0 -> x = (-b + sqrt(b^2 - 4ac)) / (2a)
        let discr = coeff_b * coeff_b - 4.0 * coeff_a * coeff_c;
        if discr < 0.0 {
            return n_active;
        }

        if coeff_a < 1e-30 {
            // High temperature or very shallow dopant: 100% complete ionization
            n_active
        } else {
            let root = (-coeff_b + discr.sqrt()) / (2.0 * coeff_a);
            root.clamp(0.0, n_active)
        }
    }

    /// Computes the ionization fraction $\eta_{ion}(T) = N_{ionized} / N_{active} \in [0, 1]$.
    pub fn ionization_fraction(
        &self,
        bandstructure: &Bandstructure,
        total_doping_m3: f64,
        temp_k: f64,
    ) -> f64 {
        let n_active = self.active_concentration(total_doping_m3);
        if n_active <= 0.0 {
            return 1.0;
        }
        let n_ion = self.ionized_concentration(bandstructure, total_doping_m3, temp_k);
        (n_ion / n_active).clamp(0.0, 1.0)
    }
}

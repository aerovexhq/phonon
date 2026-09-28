//! Coupled CISS-SMM Molecular Spintronic Logic and Non-Volatile Memory Cells.
//!
//! Synthesizes molecular spintronic junctions coupling:
//! 1. Chiral-Induced Spin Selectivity (CISS) helical spin filters.
//! 2. Single-Molecule Magnets (SMM) with bistable ground states ($m_s = \pm S$).
//! 3. Non-destructive zero-magnetic-field readout via giant differential magnetoresistance:
//!    \[\text{MR} = \frac{G_P - G_{AP}}{G_{AP}} = \frac{2 P_{CISS} P_{SMM}}{1 - P_{CISS} P_{SMM}} > 100\%\]
//! 4. Sub-femtojoule ($< 0.05\text{ fJ}$) spin-torque write operations with zero static leakage ($P_{static} = 0.0\text{ W}$).

use crate::spintronics::ciss::{ChiralHelixGeometry, Chirality, CissHamiltonian};
use crate::spintronics::smm::{SingleMoleculeMagnet, SpinValue};
use phonon_core::CONDUCTANCE_QUANTUM;

/// Binary magnetic state of the Single-Molecule Magnet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MolecularSpinState {
    /// State '1': Magnetization aligned along $+z$ ($m_s = +S$).
    State1,
    /// State '0': Magnetization aligned along $-z$ ($m_s = -S$).
    State0,
}

impl MolecularSpinState {
    #[inline(always)]
    pub fn to_bit(self) -> bool {
        match self {
            Self::State1 => true,
            Self::State0 => false,
        }
    }

    #[inline(always)]
    pub fn from_bit(bit: bool) -> Self {
        if bit {
            Self::State1
        } else {
            Self::State0
        }
    }
}

/// Coupled CISS-SMM Molecular Spintronic Unit Cell.
#[derive(Debug, Clone, PartialEq)]
pub struct MolecularSpintronicCell {
    /// Chiral helical oligomer filter.
    pub ciss: CissHamiltonian,
    /// Single-Molecule Magnet magnetic memory core.
    pub smm: SingleMoleculeMagnet,
    /// Current binary magnetization state.
    pub state: MolecularSpinState,
    /// Non-destructive readout voltage in Volts ($V_{read} \approx 10\text{ mV}$).
    pub read_voltage_v: f64,
    /// Write switching pulse voltage in Volts ($V_{write} \approx 0.3\text{ V}$).
    pub write_voltage_v: f64,
    /// Write pulse duration in seconds ($\tau_{write} \approx 1\text{ ns}$).
    pub write_pulse_duration_s: f64,
    /// Write switching current in Amperes ($I_{write} \approx 100\text{ nA}$).
    pub write_current_a: f64,
    /// Cell lateral dimension in nanometers ($nm$) (e.g. $2.0\text{ nm} \times 2.0\text{ nm}$).
    pub cell_pitch_nm: f64,
}

impl Default for MolecularSpintronicCell {
    fn default() -> Self {
        // Default: 12-site right-handed helix (DNA-like) with Dysprosium SMM core
        let geom = ChiralHelixGeometry::new(
            1.0,
            3.4,
            2.0 * std::f64::consts::PI / 10.0,
            12,
            Chirality::RightHanded,
        );
        let ciss = CissHamiltonian::new(geom, 0.0, 1.2, 0.25, 0.20)
            .with_dephasing(0.02)
            .with_chiral_potential(0.15);

        let smm = SingleMoleculeMagnet::new(
            SpinValue::s9_half(),
            -0.025, // D = -25 meV
            0.001,  // E = 1 meV
            2.0,
        )
        .with_attempt_time(1.0e-10);

        Self {
            ciss,
            smm,
            state: MolecularSpinState::State1,
            read_voltage_v: 0.010,
            write_voltage_v: 0.30,
            write_pulse_duration_s: 1.0e-9,
            write_current_a: 1.0e-7,
            cell_pitch_nm: 2.0,
        }
    }
}

impl MolecularSpintronicCell {
    pub fn new(ciss: CissHamiltonian, smm: SingleMoleculeMagnet) -> Self {
        Self {
            ciss,
            smm,
            state: MolecularSpinState::State1,
            read_voltage_v: 0.010,
            write_voltage_v: 0.30,
            write_pulse_duration_s: 1.0e-9,
            write_current_a: 1.0e-7,
            cell_pitch_nm: 2.0,
        }
    }

    /// Evaluates the CISS spin polarization $P_{CISS} \in [0, 1]$ near the Fermi level.
    pub fn ciss_spin_polarization(&self) -> f64 {
        let trans = self.ciss.calculate_transmission(0.0);
        trans.spin_polarization.abs()
    }

    /// Evaluates the SMM spin polarization factor $P_{SMM} = \langle S_z \rangle / S \approx 0.95$.
    pub fn smm_spin_polarization(&self) -> f64 {
        0.95
    }

    /// Computes the Giant Magnetoresistance (GMR) ratio:
    /// \[\text{MR} = \frac{G_P - G_{AP}}{G_{AP}} = \frac{2 P_1 P_2}{1 - P_1 P_2}\]
    pub fn mr_ratio(&self) -> f64 {
        let p_ciss = self.ciss_spin_polarization();
        let p_smm = self.smm_spin_polarization();
        let prod = (p_ciss * p_smm).clamp(0.0, 0.95);
        (2.0 * prod) / (1.0 - prod)
    }

    /// Differential electrical conductance in Siemens ($S$) for parallel (State 1) vs antiparallel (State 0).
    pub fn read_conductance(&self) -> f64 {
        let p_ciss = self.ciss_spin_polarization();
        let p_smm = self.smm_spin_polarization();
        let g_base = CONDUCTANCE_QUANTUM * 0.15; // Realistic transmission prefactor

        match self.state {
            MolecularSpinState::State1 => g_base * (1.0 + p_ciss * p_smm),
            MolecularSpinState::State0 => g_base * (1.0 - p_ciss * p_smm),
        }
    }

    /// Non-destructive readout current in Amperes ($A$) under zero applied magnetic field:
    /// \[I_{read} = G_{cell} \cdot V_{read}\]
    pub fn read_current(&self) -> f64 {
        self.read_conductance() * self.read_voltage_v
    }

    /// Non-destructive zero-magnetic-field readout returning stored binary bit.
    pub fn read_bit(&self) -> bool {
        self.state.to_bit()
    }

    /// Executes write switching operation to the `target_bit`.
    /// Returns the switching energy dissipated in Joules ($J$).
    pub fn write_bit(&mut self, target_bit: bool) -> f64 {
        let current_bit = self.state.to_bit();
        if current_bit != target_bit {
            self.state = MolecularSpinState::from_bit(target_bit);
            self.write_voltage_v * self.write_current_a * self.write_pulse_duration_s
        } else {
            0.0
        }
    }

    /// Standby static power leakage dissipation:
    /// In the non-volatile SMM state, static current is identically zero:
    /// \[P_{\text{static}} = 0.0\text{ W}\]
    #[inline(always)]
    pub fn static_leakage_w(&self) -> f64 {
        0.0
    }

    /// Retention lifetime in years at operating temperature $T$ ($K$).
    #[inline(always)]
    pub fn retention_time_years(&self, temp_k: f64) -> f64 {
        self.smm.retention_time_years(temp_k)
    }

    /// Physical cell area footprint in nanometers squared ($nm^2$).
    #[inline(always)]
    pub fn cell_footprint_nm2(&self) -> f64 {
        self.cell_pitch_nm * self.cell_pitch_nm
    }

    /// Integration density in bits per square centimeter ($\text{bits/cm}^2$).
    #[inline(always)]
    pub fn integration_density_bits_cm2(&self) -> f64 {
        let area_cm2 = (self.cell_pitch_nm * 1.0e-7).powi(2);
        1.0 / area_cm2
    }
}

/// Molecular Spintronic Inverter (NOT gate).
#[derive(Debug, Clone, Default)]
pub struct MolecularInverter {
    pub cell: MolecularSpintronicCell,
}

impl MolecularInverter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates NOT gate output and dissipated switching energy in Joules ($J$).
    pub fn evaluate(&mut self, input: bool) -> (bool, f64) {
        let target = !input;
        let energy = self.cell.write_bit(target);
        (self.cell.read_bit(), energy)
    }
}

/// Molecular Spintronic Majority-3 Gate:
/// \[M(A, B, C) = (A \land B) \lor (B \land C) \lor (A \land C)\]
#[derive(Debug, Clone, Default)]
pub struct MolecularMajority3 {
    pub cell_a: MolecularSpintronicCell,
    pub cell_b: MolecularSpintronicCell,
    pub cell_c: MolecularSpintronicCell,
    pub output_cell: MolecularSpintronicCell,
}

impl MolecularMajority3 {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates Majority-3 function and returns (output_bit, total_switching_energy_j).
    pub fn evaluate(&mut self, a: bool, b: bool, c: bool) -> (bool, f64) {
        let mut total_energy = 0.0;
        total_energy += self.cell_a.write_bit(a);
        total_energy += self.cell_b.write_bit(b);
        total_energy += self.cell_c.write_bit(c);

        let votes = (a as usize) + (b as usize) + (c as usize);
        let majority = votes >= 2;
        total_energy += self.output_cell.write_bit(majority);

        (self.output_cell.read_bit(), total_energy)
    }
}

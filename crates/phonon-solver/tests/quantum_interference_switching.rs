//! Integration Test: Quantum Interference Switching & Conformation Mechanics.
//!
//! Validates:
//! 1. Destructive Quantum Interference (QI) anti-resonance in meta-benzene vs constructive in para-benzene (> 1000x ratio).
//! 2. Molecular conformational switching via dihedral twist angle: \(t(\theta) = t_0 \cos(\theta)\).
//! 3. Sub-100 meV energy dissipation per logic state transition.

use phonon_models::molecular::{
    MolecularInverter, MolecularJunction, MolecularXor2, NegfTransportSolver,
};
use std::f64::consts::PI;

#[test]
fn test_para_vs_meta_quantum_interference_ratio() {
    let solver = NegfTransportSolver::new(300.0, 0.0);
    let para = MolecularJunction::para_benzene(0.6);
    let meta = MolecularJunction::meta_benzene(0.6);

    let t_para = solver.evaluate_transmission(&para, 0.0, 0.0);
    let t_meta = solver.evaluate_transmission(&meta, 0.0, 0.0);

    assert!(
        t_para > 0.01,
        "Para transmission should be > 0.01, got {:.4}",
        t_para
    );
    assert!(
        t_meta < 0.0001,
        "Meta transmission should be < 0.0001 due to QI anti-resonance, got {:.6}",
        t_meta
    );

    let qi_ratio = t_para / t_meta.max(1.0e-12);
    assert!(
        qi_ratio > 1000.0,
        "Quantum interference on/off ratio must exceed 1,000x, got {:.1}x",
        qi_ratio
    );
}

#[test]
fn test_conformational_dihedral_twist_switching() {
    let solver = NegfTransportSolver::new(300.0, 0.0);
    let mut junction = MolecularJunction::linear_chain(4, 0.6);

    // Planar conformation: theta = 0 rad (cos(0) = 1) -> maximum pi orbital overlap
    junction.set_dihedral_angle(0.0);
    let t_planar = solver.evaluate_transmission(&junction, -1.0, 0.0);

    // Twisted conformation: theta = 85 degrees (broken pi conjugation)
    junction.set_dihedral_angle(85.0 * PI / 180.0);
    let t_twisted = solver.evaluate_transmission(&junction, -1.0, 0.0);

    assert!(
        t_planar > t_twisted * 5.0,
        "Planar transmission ({:.4}) should be significantly higher than twisted ({:.4})",
        t_planar,
        t_twisted
    );

    // Orthogonal conformation: theta = 90 degrees (cos(pi/2) = 0) -> broken pathway
    junction.set_dihedral_angle(90.0 * PI / 180.0);
    let t_orthogonal = solver.evaluate_transmission(&junction, -1.0, 0.0);
    assert!(
        t_orthogonal < 1.0e-4,
        "Orthogonal twist should pinch off transmission, got {:.6}",
        t_orthogonal
    );
}

#[test]
fn test_sub_100_mev_switching_energy_primitives() {
    let inverter = MolecularInverter::new(0.35, 2.5e6);
    let m_inv = inverter.compute_metrics();
    assert!(
        m_inv.switching_energy_mev < 100.0,
        "Inverter switching energy must be sub-100 meV, got {:.2} meV",
        m_inv.switching_energy_mev
    );
    assert!(
        m_inv.switching_energy_joules < 1.6e-19,
        "Inverter switching energy in Joules must be < 1.6e-19 J, got {:.2e} J",
        m_inv.switching_energy_joules
    );

    let xor = MolecularXor2::new(0.35);
    let m_xor = xor.compute_metrics();
    assert!(
        m_xor.switching_energy_mev < 100.0,
        "XOR2 switching energy must be sub-100 meV, got {:.2} meV",
        m_xor.switching_energy_mev
    );
}

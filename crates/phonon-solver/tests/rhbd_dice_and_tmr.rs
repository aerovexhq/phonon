//! Integration test: Radiation-Hardened-by-Design (RHBD) Dual-Interlocked Storage Cell (DICE),
//! Triple-Modular Redundancy (TMR) majority voting, and Displacement Damage Dose (DDD) degradation.

use phonon_models::{DiceCell, DisplacementDamageModel, StandardSramCell, TmrVoter};

#[test]
fn test_dice_vs_standard_sram_single_node_strike_resilience() {
    let mut sram = StandardSramCell::new(true, 1.0);
    let mut dice = DiceCell::new(true, 1.0);

    let q_crit_sram = sram.critical_charge_fc();
    assert!(q_crit_sram > 1.0 && q_crit_sram < 5.0);

    // Apply 20 fC strike (10x SRAM Qcrit) to standard SRAM
    let sram_outcome = sram.strike(20.0);
    assert!(
        sram_outcome.upset_occurred,
        "Standard 6T SRAM must flip under 20 fC strike"
    );
    assert!(!sram.state_q); // Bit corrupted from 1 to 0

    // Apply the exact same 20 fC strike to internal node 0 of DICE cell
    let dice_outcome = dice.strike_single_node(0, 20.0);
    assert!(
        !dice_outcome.upset_occurred,
        "DICE cell must be immune to single-node 20 fC strike"
    );
    assert!(dice_outcome.self_restored);
    assert!(dice.read_bit()); // Bit preserved as 1!

    // Even an extreme 100 fC single-node strike cannot corrupt DICE
    let dice_extreme = dice.strike_single_node(2, 100.0);
    assert!(!dice_extreme.upset_occurred);
    assert!(dice.read_bit());
}

#[test]
fn test_dice_two_node_charge_sharing_vulnerability() {
    let mut dice = DiceCell::new(true, 1.0);
    let q_crit = dice.standard_q_crit_fc();

    // Two adjacent nodes (node 0 and node 1) both struck simultaneously with supercritical charge
    let outcome = dice.strike_adjacent_nodes(0, q_crit * 3.0, 1, q_crit * 3.0);
    assert!(
        outcome.upset_occurred,
        "Simultaneous adjacent two-node charge sharing must overcome DICE cross-coupling"
    );
    assert!(!dice.read_bit()); // State flipped
}

#[test]
fn test_tmr_majority_voting_glitch_masking_stream() {
    let voter = TmrVoter::new(1.2);
    let vdd = 1.2;

    // Simulate 50 clock cycles of digital logic
    let mut cycle_inputs = Vec::new();
    for cycle in 0..50 {
        let nominal_high = (cycle / 5) % 2 == 0;
        let v_nominal = if nominal_high { vdd } else { 0.0 };

        let (mut va, mut vb, mut vc) = (v_nominal, v_nominal, v_nominal);

        // Inject SET transient glitches on individual channels
        if cycle == 7 {
            va = if nominal_high { 0.1 } else { 1.1 }; // SET on channel 0
        } else if cycle == 18 {
            vb = if nominal_high { 0.2 } else { 1.0 }; // SET on channel 1
        } else if cycle == 33 {
            vc = if nominal_high { 0.05 } else { 1.15 }; // SET on channel 2
        }

        cycle_inputs.push((v_nominal, va, vb, vc, cycle));
    }

    let mut glitches_masked = 0;
    for (v_expected, va, vb, vc, cycle) in cycle_inputs {
        let output = voter.vote(va, vb, vc);

        // The majority voter output MUST always match the true expected signal
        assert!(
            (output.voted_output - v_expected).abs() < 1e-6,
            "Cycle {}: TMR output failed to mask fault! got {} V, expected {} V",
            cycle,
            output.voted_output,
            v_expected
        );

        if output.fault_detected() {
            glitches_masked += 1;
            match cycle {
                7 => assert_eq!(output.disagreeing_channel, Some(0)),
                18 => assert_eq!(output.disagreeing_channel, Some(1)),
                33 => assert_eq!(output.disagreeing_channel, Some(2)),
                _ => {}
            }
        }
    }

    assert_eq!(
        glitches_masked, 3,
        "Exactly 3 injected glitches must be detected and masked"
    );
}

#[test]
fn test_displacement_damage_dose_physics() {
    let ddd = DisplacementDamageModel::space_proton_environment(120.0);

    // Dose sweep from 1e10 to 1e13 protons/cm^2
    let fluences = [1.0e10, 1.0e11, 1.0e12, 1.0e13];

    let mut lifetimes = Vec::new();
    let mut betas = Vec::new();
    let mut dark_spikes = Vec::new();
    let mut efficiencies = Vec::new();

    for &phi in &fluences {
        lifetimes.push(ddd.carrier_lifetime_s(phi));
        betas.push(ddd.degraded_beta(phi));
        dark_spikes.push(ddd.dark_current_spike_a(phi));
        efficiencies.push(ddd.relative_collection_efficiency(phi));
    }

    // 1. Lifetime must monotonically decrease
    for i in 1..lifetimes.len() {
        assert!(lifetimes[i] < lifetimes[i - 1]);
    }

    // 2. BJT gain beta must monotonically decrease
    for i in 1..betas.len() {
        assert!(betas[i] < betas[i - 1]);
    }
    assert!(betas.last().unwrap() < &20.0); // Significant gain loss at 1e13

    // 3. Dark current must monotonically increase
    for i in 1..dark_spikes.len() {
        assert!(dark_spikes[i] > dark_spikes[i - 1]);
    }

    // 4. Photovoltaic collection efficiency must decrease
    for i in 1..efficiencies.len() {
        assert!(efficiencies[i] < efficiencies[i - 1]);
    }
}

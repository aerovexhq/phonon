//! Integration test: Single-Event Latchup (SEL) in parasitic CMOS p-n-p-n thyristors,
//! regenerative feedback, holding state, and coupled electro-thermal positive feedback.

use phonon_models::ParasiticThyristorModel;

#[test]
fn test_sel_untriggered_and_subthreshold_immunity() {
    let mut thyristor = ParasiticThyristorModel::bulk_cmos_inverter();
    assert!(!thyristor.latched_state);

    let vdd = 3.3;
    let eval_off = thyristor.evaluate(vdd, 0.0, 300.15);
    assert!(!eval_off.is_latched);
    assert!(eval_off.current_a < 1e-6); // Off leakage < 1 uA
    assert!(eval_off.power_dissipation_w < 1e-5);
    assert!((eval_off.junction_temperature_k - 300.15).abs() < 1e-3); // No self-heating

    // Inject 50% of critical trigger current: should NOT latch
    let i_crit = eval_off.critical_trigger_current_a;
    let eval_sub = thyristor.evaluate(vdd, i_crit * 0.5, 300.15);
    assert!(!eval_sub.is_latched);
    assert!(!thyristor.latched_state);
}

#[test]
fn test_sel_supercritical_trigger_and_holding_state() {
    let mut thyristor = ParasiticThyristorModel::bulk_cmos_inverter();
    let vdd = 3.3;
    let eval_init = thyristor.evaluate(vdd, 0.0, 300.15);
    let i_crit = eval_init.critical_trigger_current_a;

    // 1. Inject supercritical ion strike pulse (1.5x i_crit)
    let eval_latched = thyristor.evaluate(vdd, i_crit * 1.5, 300.15);
    assert!(eval_latched.is_latched);
    assert!(thyristor.latched_state);
    assert!(
        eval_latched.current_a > 0.1,
        "Latched state current must exceed 100 mA: got {} A",
        eval_latched.current_a
    );
    assert!(eval_latched.power_dissipation_w > 0.3); // > 300 mW
    assert!(
        eval_latched.junction_temperature_k > 300.15 + 10.0,
        "Electro-thermal dissipation must heat junction: got {} K",
        eval_latched.junction_temperature_k
    );

    // 2. Remove trigger current completely: thyristor must REMAIN latched in holding state!
    let eval_held = thyristor.evaluate(vdd, 0.0, 300.15);
    assert!(eval_held.is_latched);
    assert!(eval_held.current_a > 0.1);
}

#[test]
fn test_sel_power_cycle_quenching() {
    let mut thyristor = ParasiticThyristorModel::bulk_cmos_inverter();
    let vdd = 3.3;
    let eval_init = thyristor.evaluate(vdd, 0.0, 300.15);
    let i_crit = eval_init.critical_trigger_current_a;

    // Trigger latchup
    thyristor.evaluate(vdd, i_crit * 2.0, 300.15);
    assert!(thyristor.latched_state);

    // Power collapse below holding voltage (e.g. 0.5V < 1.1V V_hold)
    let eval_collapsed = thyristor.evaluate(0.5, 0.0, 300.15);
    assert!(!eval_collapsed.is_latched);
    assert!(!thyristor.latched_state);

    // Re-apply Vdd = 3.3V: must remain unlatched after power recovery
    let eval_recovered = thyristor.evaluate(vdd, 0.0, 300.15);
    assert!(!eval_recovered.is_latched);
    assert!(eval_recovered.current_a < 1e-6);
}

#[test]
fn test_rhbd_guard_ring_sel_immunity() {
    let mut rhbd_thyristor = ParasiticThyristorModel::rad_hard_guard_ring_cmos();

    // Guard rings suppress bipolar gain: beta_pnp * beta_npn < 1.0
    let loop_gain = rhbd_thyristor.loop_gain(300.15);
    assert!(
        loop_gain < 1.0,
        "Guard ring CMOS must have loop gain < 1.0 to guarantee SEL immunity: got {}",
        loop_gain
    );

    // Massive ion strike current (100 mA) cannot sustain regenerative latch
    let eval = rhbd_thyristor.evaluate(3.3, 0.10, 300.15);
    assert!(!eval.is_latched);
    assert!(!rhbd_thyristor.latched_state);
}

#[test]
fn test_sel_electrothermal_positive_feedback() {
    let thyristor = ParasiticThyristorModel::bulk_cmos_inverter();

    let i_crit_300 = thyristor.critical_trigger_current(300.15);
    let i_crit_350 = thyristor.critical_trigger_current(350.15);
    let i_crit_400 = thyristor.critical_trigger_current(400.15);

    // Temperature elevation reduces barrier height, decreasing critical trigger current
    assert!(i_crit_350 < i_crit_300);
    assert!(i_crit_400 < i_crit_350);

    // Temperature elevation boosts loop gain
    let gain_300 = thyristor.loop_gain(300.15);
    let gain_400 = thyristor.loop_gain(400.15);
    assert!(gain_400 > gain_300);
}

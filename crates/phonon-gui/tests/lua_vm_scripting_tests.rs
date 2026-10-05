#![deny(unsafe_code)]

//! Verification test suite for Phase 355: Pure Safe Rust Lua Scripting VM.
//!
//! Tests lexer, parser, variables, tables, loops, conditional branching,
//! closures, standard math/string/table libraries, and Phonon CAD testbench APIs.

use phonon_gui::scripting::lua_engine::{LuaEngine, LuaValue};

#[test]
fn test_lua_lexer_and_parser_basic_syntax() {
    let mut engine = LuaEngine::new();
    let script = r#"
        -- Single line comment
        --[[ Multi-line
             comment block ]]
        local a = 42
        local b = 3.14159
        local c = 1e-3
        local d = 2.5e3
        local s1 = "hello world"
        local s2 = 'phonon cad'
        local flag_t = true
        local flag_f = false
        local n = nil

        local sum = a + b + c
        return sum
    "#;

    let res = engine.run_script(script).expect("Script execution failed");
    match res {
        LuaValue::Number(val) => {
            assert!((val - (42.0 + 3.14159 + 0.001)).abs() < 1e-6);
        }
        other => panic!("Expected number return, got {:?}", other),
    }
}

#[test]
fn test_lua_control_flow_branching_and_loops() {
    let mut engine = LuaEngine::new();
    let script = r#"
        local x = 15
        local category = ""
        if x < 10 then
            category = "low"
        elseif x == 15 then
            category = "medium"
        else
            category = "high"
        end

        local sum_for = 0
        for i = 1, 10 do
            sum_for = sum_for + i
        end

        local sum_step = 0
        for i = 10, 2, -2 do
            sum_step = sum_step + i
        end

        local count = 0
        while count < 5 do
            count = count + 1
            if count == 3 then
                -- continue counting
            end
        end

        return sum_for + sum_step + count
    "#;

    let res = engine.run_script(script).expect("Control flow script failed");
    // sum_for = 55 (1..10)
    // sum_step = 10 + 8 + 6 + 4 + 2 = 30
    // count = 5
    // total = 90
    match res {
        LuaValue::Number(n) => assert_eq!(n, 90.0),
        other => panic!("Expected number return, got {:?}", other),
    }
}

#[test]
fn test_lua_functions_and_closures() {
    let mut engine = LuaEngine::new();
    let script = r#"
        function add(a, b)
            return a + b
        end

        local function make_counter(start)
            local current = start
            return function(step)
                current = current + step
                return current
            end
        end

        local c = make_counter(10)
        local val1 = c(5)  -- 15
        local val2 = c(10) -- 25

        return add(val1, val2) -- 40
    "#;

    let res = engine.run_script(script).expect("Closure script failed");
    match res {
        LuaValue::Number(n) => assert_eq!(n, 40.0),
        other => panic!("Expected number return, got {:?}", other),
    }
}

#[test]
fn test_lua_tables_and_table_library() {
    let mut engine = LuaEngine::new();
    let script = r#"
        local t = { 10, 20, 30 }
        table.insert(t, 40)
        local len1 = #t -- 4

        local removed = table.remove(t) -- 40
        local len2 = #t -- 3

        local joined = table.concat(t, "-") -- "10-20-30"

        local map = { mode = "fast", ["frequency"] = 1000 }
        map.gain = 2.5

        return map.frequency + map.gain + len1 + len2
    "#;

    let res = engine.run_script(script).expect("Table script failed");
    // 1000 + 2.5 + 4 + 3 = 1009.5
    match res {
        LuaValue::Number(n) => assert_eq!(n, 1009.5),
        other => panic!("Expected number return, got {:?}", other),
    }
}

#[test]
fn test_lua_standard_library_math() {
    let mut engine = LuaEngine::new();
    let script = r#"
        local s = math.sin(0.0)
        local c = math.cos(0.0)
        local sq = math.sqrt(16.0)
        local ab = math.abs(-42.0)
        local ex = math.exp(0.0)
        local lg = math.log(math.exp(1.0))
        local mn = math.min(10, 20, 5, 30)
        local mx = math.max(10, 20, 5, 30)
        local fl = math.floor(3.9)
        local cl = math.ceil(3.1)
        local p = math.pi

        return s + c + sq + ab + ex + lg + mn + mx + fl + cl
    "#;

    let res = engine.run_script(script).expect("Math script failed");
    // s = 0, c = 1, sq = 4, ab = 42, ex = 1, lg = 1, mn = 5, mx = 30, fl = 3, cl = 4
    // total = 0 + 1 + 4 + 42 + 1 + 1 + 5 + 30 + 3 + 4 = 91
    match res {
        LuaValue::Number(n) => assert_eq!(n, 91.0),
        other => panic!("Expected number return, got {:?}", other),
    }
}

#[test]
fn test_lua_standard_library_string_and_core() {
    let mut engine = LuaEngine::new();
    let script = r#"
        local str = "PhononCAD"
        local l = string.len(str) -- 9
        local sub = string.sub(str, 1, 6) -- "Phonon"
        local up = string.upper(sub) -- "PHONON"
        local lo = string.lower(sub) -- "phonon"

        local t_str = type("hello") -- "string"
        local t_num = type(123) -- "number"
        local num = tonumber("456.5") -- 456.5
        local s_conv = tostring(789) -- "789"

        print("Test log output: " .. up .. " " .. s_conv)
        return num + l
    "#;

    let res = engine.run_script(script).expect("String script failed");
    match res {
        LuaValue::Number(n) => assert_eq!(n, 456.5 + 9.0),
        other => panic!("Expected number return, got {:?}", other),
    }

    assert!(!engine.output_log.is_empty());
    assert!(engine.output_log[0].contains("PHONON 789"));
}

#[test]
fn test_phonon_cad_simulation_api_and_telemetry() {
    let mut engine = LuaEngine::new();
    engine.set_voltage("VIN", 5.0);
    engine.set_voltage("VOUT", 2.5);
    engine.set_current("I_R1", 0.0025);

    let script = r#"
        phonon.log("Starting testbench verification")
        local vin = phonon.get_voltage("VIN")
        local vout = phonon.get_voltage("VOUT")
        local ir1 = phonon.get_current("I_R1")

        -- Passing assertions
        phonon.assert_eq(vin, 5.0, 1e-4, "VIN supply rail check")
        phonon.assert_range(vout, 2.4, 2.6, "VOUT operating point check")
        phonon.assert_eq(ir1, 0.0025, 1e-5, "Branch current check")

        -- Failing assertion to verify failure tracking
        phonon.assert_eq(vout, 10.0, 1e-4, "Intentional fail check")

        local dc = phonon.run_dc()
        local tran = phonon.run_transient(0.003, 1e-5)
    "#;

    let _ = engine.run_script(script).expect("Script execution failed");

    assert_eq!(engine.total_assertions(), 4);
    assert_eq!(engine.passed_assertions(), 3);
    assert_eq!(engine.failed_assertions(), 1);

    // Verify output log contains pass and fail markers
    assert!(engine.output_log.iter().any(|l| l.contains("[PASS] VIN supply rail check")));
    assert!(engine.output_log.iter().any(|l| l.contains("[PASS] VOUT operating point check")));
    assert!(engine.output_log.iter().any(|l| l.contains("[FAIL] Intentional fail check")));
    assert!(engine.output_log.iter().any(|l| l.contains("[PHONON] Starting testbench verification")));
}

#[test]
fn test_phonon_plot_expression_in_lua() {
    let mut engine = LuaEngine::new();
    let script = r#"
        phonon.plot_expression("Sine Wave", "2.5 + 2.5 * sin(2 * pi * 1000 * t)")
        phonon.plot_expression("Cosine Wave", "1.0 * cos(2 * pi * 2000 * t)")
    "#;

    let _ = engine.run_script(script).expect("Expression plot script failed");

    assert_eq!(engine.generated_traces.len(), 2);
    let trace1 = &engine.generated_traces[0];
    assert_eq!(trace1.name, "Sine Wave");
    assert_eq!(trace1.samples.len(), 1000);

    let trace2 = &engine.generated_traces[1];
    assert_eq!(trace2.name, "Cosine Wave");
    assert_eq!(trace2.samples.len(), 1000);
}

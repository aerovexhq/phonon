#![deny(unsafe_code)]

//! Structural and behavioral Verilog/VHDL HDL parser and digital graph compiler.
//!
//! Parses modules, combinational assignments (`assign`), sequential registers
//! (`always @(posedge clk)`), and structural primitive gates into an interconnected `DigitalEngine`.

use std::collections::HashMap;

use crate::mixed_signal::digital_engine::{DigitalEngine, DigitalLogicGate, GateKind};

/// HDL parsing and compilation errors.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HdlParseError {
    #[error("Missing module declaration")]
    MissingModule,
    #[error("Unexpected end of file while parsing HDL")]
    UnexpectedEof,
    #[error("Syntax error in HDL: {0}")]
    SyntaxError(String),
    #[error("Unknown or unsupported gate/expression: {0}")]
    UnsupportedExpression(String),
    #[error("Undefined signal identifier: {0}")]
    UndefinedSignal(String),
}

/// Representation of a compiled Verilog HDL module.
#[derive(Debug, Clone)]
pub struct ParsedHdlModule {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub wires: Vec<String>,
    pub net_to_id: HashMap<String, usize>,
    pub id_to_net: HashMap<usize, String>,
    pub engine: DigitalEngine,
}

impl ParsedHdlModule {
    /// Look up the digital engine node ID for a given net name.
    pub fn get_net_id(&self, name: &str) -> Option<usize> {
        self.net_to_id.get(name).copied()
    }

    /// Look up the signal name for a given node ID.
    pub fn get_net_name(&self, id: usize) -> Option<&str> {
        self.id_to_net.get(&id).map(|s| s.as_str())
    }
}

/// Strips single-line (`//`) and block (`/* ... */`) comments from HDL source.
fn strip_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            i += 2;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            if i + 1 < chars.len() {
                i += 2;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

/// Tokenizes sanitized HDL source text into discrete tokens.
fn tokenize(source: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let clean = strip_comments(source);
    let chars: Vec<char> = clean.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Two-character operators: <=, ~^, ^~
        if i + 1 < chars.len() {
            let next_c = chars[i + 1];
            if (c == '<' && next_c == '=')
                || (c == '~' && next_c == '^')
                || (c == '^' && next_c == '~')
            {
                tokens.push(format!("{}{}", c, next_c));
                i += 2;
                continue;
            }
        }

        // Single-character symbols
        if matches!(c, ';' | ',' | '(' | ')' | '=' | '&' | '|' | '^' | '~' | '!' | '@' | '[' | ']' | ':') {
            tokens.push(c.to_string());
            i += 1;
            continue;
        }

        // Alphanumeric words / identifiers
        if c.is_alphanumeric() || c == '_' || c == '$' {
            let mut word = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '$') {
                word.push(chars[i]);
                i += 1;
            }
            tokens.push(word);
            continue;
        }

        // Other single characters
        tokens.push(c.to_string());
        i += 1;
    }

    tokens
}

/// Intermediate parsed gate definition before node ID allocation.
struct RawGate {
    kind: GateKind,
    inputs: Vec<String>,
    output: String,
    delay_s: f64,
}

/// Parses a Verilog module from source code into a `ParsedHdlModule`.
pub fn parse_verilog_module(source: &str) -> Result<ParsedHdlModule, HdlParseError> {
    let tokens = tokenize(source);
    let mut idx = 0;

    // Find `module` keyword
    while idx < tokens.len() && tokens[idx] != "module" {
        idx += 1;
    }
    if idx >= tokens.len() {
        return Err(HdlParseError::MissingModule);
    }
    idx += 1;

    // Module name
    if idx >= tokens.len() {
        return Err(HdlParseError::UnexpectedEof);
    }
    let module_name = tokens[idx].clone();
    idx += 1;

    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    let mut wires = Vec::new();
    let mut raw_gates = Vec::new();

    let default_delay_s = 1.0e-11; // 10 ps default propagation delay

    // Parse port list in header: `( ... )`
    if idx < tokens.len() && tokens[idx] == "(" {
        idx += 1;
        let mut cur_dir = String::new();
        while idx < tokens.len() && tokens[idx] != ")" {
            let tok = &tokens[idx];
            if tok == "input" || tok == "output" || tok == "inout" {
                cur_dir = tok.clone();
                idx += 1;
                // Skip optional 'wire' or 'reg'
                if idx < tokens.len() && (tokens[idx] == "wire" || tokens[idx] == "reg") {
                    idx += 1;
                }
                // Skip optional bit width: [N:M]
                if idx < tokens.len() && tokens[idx] == "[" {
                    while idx < tokens.len() && tokens[idx] != "]" {
                        idx += 1;
                    }
                    if idx < tokens.len() && tokens[idx] == "]" {
                        idx += 1;
                    }
                }
                continue;
            }

            if tok == "," {
                idx += 1;
                continue;
            }

            // Signal identifier
            if cur_dir == "input" {
                inputs.push(tok.clone());
            } else if cur_dir == "output" {
                outputs.push(tok.clone());
            } else {
                wires.push(tok.clone());
            }
            idx += 1;
        }
        if idx < tokens.len() && tokens[idx] == ")" {
            idx += 1;
        }
    }

    // Expect semicolon after module header
    if idx < tokens.len() && tokens[idx] == ";" {
        idx += 1;
    }

    // Parse module body until `endmodule`
    while idx < tokens.len() {
        let tok = &tokens[idx];

        if tok == "endmodule" {
            break;
        }

        // Port or wire declarations in body
        if tok == "input" || tok == "output" || tok == "wire" || tok == "reg" {
            let decl_type = tok.clone();
            idx += 1;
            // Skip optional bit width [N:M]
            if idx < tokens.len() && tokens[idx] == "[" {
                while idx < tokens.len() && tokens[idx] != "]" {
                    idx += 1;
                }
                if idx < tokens.len() && tokens[idx] == "]" {
                    idx += 1;
                }
            }
            while idx < tokens.len() && tokens[idx] != ";" {
                if tokens[idx] != "," {
                    let sig = tokens[idx].clone();
                    if decl_type == "input" && !inputs.contains(&sig) {
                        inputs.push(sig);
                    } else if decl_type == "output" && !outputs.contains(&sig) {
                        outputs.push(sig);
                    } else if (decl_type == "wire" || decl_type == "reg")
                        && !wires.contains(&sig)
                        && !inputs.contains(&sig)
                        && !outputs.contains(&sig)
                    {
                        wires.push(sig);
                    }
                }
                idx += 1;
            }
            if idx < tokens.len() && tokens[idx] == ";" {
                idx += 1;
            }
            continue;
        }

        // Combinational assignment: `assign <lhs> = <rhs>;`
        if tok == "assign" {
            idx += 1;
            if idx >= tokens.len() {
                return Err(HdlParseError::UnexpectedEof);
            }
            let lhs = tokens[idx].clone();
            idx += 1;

            if idx >= tokens.len() || tokens[idx] != "=" {
                return Err(HdlParseError::SyntaxError(format!(
                    "Expected '=' after assign target '{}'",
                    lhs
                )));
            }
            idx += 1;

            // Collect RHS expression tokens until semicolon
            let mut expr_tokens = Vec::new();
            while idx < tokens.len() && tokens[idx] != ";" {
                expr_tokens.push(tokens[idx].clone());
                idx += 1;
            }
            if idx < tokens.len() && tokens[idx] == ";" {
                idx += 1;
            }

            let gate = parse_rhs_expression(lhs, &expr_tokens, default_delay_s)?;
            raw_gates.push(gate);
            continue;
        }

        // Sequential register: `always @(posedge clk ...)`
        if tok == "always" {
            idx += 1;
            // Expect `@`
            if idx < tokens.len() && tokens[idx] == "@" {
                idx += 1;
            }
            // Parse sensitivity list `( ... )`
            let mut clk_sig = String::new();
            let mut rst_sig: Option<String> = None;
            if idx < tokens.len() && tokens[idx] == "(" {
                idx += 1;
                while idx < tokens.len() && tokens[idx] != ")" {
                    if tokens[idx] == "posedge" {
                        idx += 1;
                        if idx < tokens.len() {
                            if clk_sig.is_empty() {
                                clk_sig = tokens[idx].clone();
                            } else {
                                rst_sig = Some(tokens[idx].clone());
                            }
                            idx += 1;
                        }
                    } else {
                        idx += 1;
                    }
                }
                if idx < tokens.len() && tokens[idx] == ")" {
                    idx += 1;
                }
            }

            // Parse body inside `always`
            let mut target_q = String::new();
            let mut source_d = String::new();

            while idx < tokens.len() && tokens[idx] != "endmodule" {
                // Check if we hit next statement at module level
                if tokens[idx] == "assign" || tokens[idx] == "always" || tokens[idx] == "endmodule" {
                    break;
                }

                // Look for non-blocking assignment: `q <= d;`
                if tokens[idx] == "<=" {
                    if idx > 0 {
                        target_q = tokens[idx - 1].clone();
                    }
                    idx += 1;
                    if idx < tokens.len() {
                        let rhs = tokens[idx].clone();
                        // Ignore constant zero reset assignments like `1'b0` or `0`
                        if rhs != "0" && rhs != "1'b0" && rhs != "1'b1" {
                            source_d = rhs;
                        }
                        idx += 1;
                    }
                    continue;
                }

                if tokens[idx] == "end" {
                    idx += 1;
                    break;
                }

                idx += 1;
            }

            if !clk_sig.is_empty() && !target_q.is_empty() && !source_d.is_empty() {
                let mut dff_inputs = vec![clk_sig, source_d];
                if let Some(rst) = rst_sig {
                    dff_inputs.push(rst);
                }
                raw_gates.push(RawGate {
                    kind: GateKind::Dff,
                    inputs: dff_inputs,
                    output: target_q,
                    delay_s: default_delay_s,
                });
            }
            continue;
        }

        // Structural primitive gates: and, or, nand, nor, xor, xnor, not, buf
        let prim_kind = match tok.as_str() {
            "and" => Some(GateKind::And),
            "or" => Some(GateKind::Or),
            "nand" => Some(GateKind::Nand),
            "nor" => Some(GateKind::Nor),
            "xor" => Some(GateKind::Xor),
            "xnor" => Some(GateKind::Xnor),
            "not" => Some(GateKind::Not),
            "buf" => Some(GateKind::Buffer),
            _ => None,
        };

        if let Some(kind) = prim_kind {
            idx += 1;
            // Optional instance name before '('
            if idx < tokens.len() && tokens[idx] != "(" {
                idx += 1; // skip instance identifier
            }
            // Parse arguments `(out, in1, in2, ...)`
            if idx < tokens.len() && tokens[idx] == "(" {
                idx += 1;
                let mut args = Vec::new();
                while idx < tokens.len() && tokens[idx] != ")" {
                    if tokens[idx] != "," {
                        args.push(tokens[idx].clone());
                    }
                    idx += 1;
                }
                if idx < tokens.len() && tokens[idx] == ")" {
                    idx += 1;
                }
                if idx < tokens.len() && tokens[idx] == ";" {
                    idx += 1;
                }

                if !args.is_empty() {
                    let out_sig = args[0].clone();
                    let in_sigs = args[1..].to_vec();
                    raw_gates.push(RawGate {
                        kind,
                        inputs: in_sigs,
                        output: out_sig,
                        delay_s: default_delay_s,
                    });
                }
            }
            continue;
        }

        idx += 1;
    }

    // Allocate unified node IDs for all nets
    let mut net_to_id = HashMap::new();
    let mut id_to_net = HashMap::new();

    let mut allocate_net = |name: &str| -> usize {
        let next_id = net_to_id.len();
        *net_to_id.entry(name.to_string()).or_insert_with(|| {
            id_to_net.insert(next_id, name.to_string());
            next_id
        })
    };

    for sig in &inputs {
        allocate_net(sig);
    }
    for sig in &outputs {
        allocate_net(sig);
    }
    for sig in &wires {
        allocate_net(sig);
    }
    for gate in &raw_gates {
        allocate_net(&gate.output);
        for inp in &gate.inputs {
            allocate_net(inp);
        }
    }

    // Build the DigitalEngine
    let mut engine = DigitalEngine::new(net_to_id.len());
    for (gate_id, gate) in raw_gates.into_iter().enumerate() {
        let in_ids: Vec<usize> = gate
            .inputs
            .iter()
            .map(|name| net_to_id[name])
            .collect();
        let out_id = net_to_id[&gate.output];
        engine.add_gate(DigitalLogicGate::new(
            gate_id,
            gate.kind,
            in_ids,
            out_id,
            gate.delay_s,
        ));
    }

    Ok(ParsedHdlModule {
        name: module_name,
        inputs,
        outputs,
        wires,
        net_to_id,
        id_to_net,
        engine,
    })
}

/// Parses a combinational RHS expression into a RawGate.
fn parse_rhs_expression(
    output: String,
    tokens: &[String],
    delay_s: f64,
) -> Result<RawGate, HdlParseError> {
    if tokens.is_empty() {
        return Err(HdlParseError::SyntaxError("Empty assign expression".into()));
    }

    // Inverted compound expression: `~(a & b)`, `~(a | b)`, `~(a ^ b)`
    if tokens.len() >= 4 && tokens[0] == "~" && tokens[1] == "(" && tokens.last().map(|s| s.as_str()) == Some(")") {
        let inner = &tokens[2..tokens.len() - 1];
        if inner.contains(&"&".to_string()) {
            let operands: Vec<String> = inner.iter().filter(|s| s.as_str() != "&").cloned().collect();
            return Ok(RawGate {
                kind: GateKind::Nand,
                inputs: operands,
                output,
                delay_s,
            });
        }
        if inner.contains(&"|".to_string()) {
            let operands: Vec<String> = inner.iter().filter(|s| s.as_str() != "|").cloned().collect();
            return Ok(RawGate {
                kind: GateKind::Nor,
                inputs: operands,
                output,
                delay_s,
            });
        }
        if inner.contains(&"^".to_string()) {
            let operands: Vec<String> = inner.iter().filter(|s| s.as_str() != "^").cloned().collect();
            return Ok(RawGate {
                kind: GateKind::Xnor,
                inputs: operands,
                output,
                delay_s,
            });
        }
    }

    // Unary inversion: `~in` or `!in`
    if (tokens[0] == "~" || tokens[0] == "!") && tokens.len() == 2 {
        return Ok(RawGate {
            kind: GateKind::Not,
            inputs: vec![tokens[1].clone()],
            output,
            delay_s,
        });
    }

    // AND expression: `a & b & ...`
    if tokens.contains(&"&".to_string()) {
        let operands: Vec<String> = tokens.iter().filter(|s| s.as_str() != "&").cloned().collect();
        return Ok(RawGate {
            kind: GateKind::And,
            inputs: operands,
            output,
            delay_s,
        });
    }

    // OR expression: `a | b | ...`
    if tokens.contains(&"|".to_string()) {
        let operands: Vec<String> = tokens.iter().filter(|s| s.as_str() != "|").cloned().collect();
        return Ok(RawGate {
            kind: GateKind::Or,
            inputs: operands,
            output,
            delay_s,
        });
    }

    // XNOR expression: `a ~^ b` or `a ^~ b`
    if tokens.contains(&"~^".to_string()) || tokens.contains(&"^~".to_string()) {
        let operands: Vec<String> = tokens
            .iter()
            .filter(|s| s.as_str() != "~^" && s.as_str() != "^~")
            .cloned()
            .collect();
        return Ok(RawGate {
            kind: GateKind::Xnor,
            inputs: operands,
            output,
            delay_s,
        });
    }

    // XOR expression: `a ^ b`
    if tokens.contains(&"^".to_string()) {
        let operands: Vec<String> = tokens.iter().filter(|s| s.as_str() != "^").cloned().collect();
        return Ok(RawGate {
            kind: GateKind::Xor,
            inputs: operands,
            output,
            delay_s,
        });
    }

    // Simple buffer assignment: `assign out = in;`
    if tokens.len() == 1 {
        return Ok(RawGate {
            kind: GateKind::Buffer,
            inputs: vec![tokens[0].clone()],
            output,
            delay_s,
        });
    }

    Err(HdlParseError::UnsupportedExpression(tokens.join(" ")))
}

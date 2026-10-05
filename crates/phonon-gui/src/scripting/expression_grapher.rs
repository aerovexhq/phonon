#![deny(unsafe_code)]

//! Mathematical Expression Waveform Graphing Engine for Phonon Studio Oscilloscope.
//!
//! Evaluates arbitrary mathematical equations over time intervals and node voltages,
//! compiling dynamic formulas into high-resolution virtual oscilloscope waveform traces.

use egui::Color32;
use std::collections::HashMap;
use std::f64::consts::{E, PI};
use crate::oscilloscope::WaveformTrace;

/// Mathematical binary operators for expression evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

/// Abstract Syntax Tree node for mathematical expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum MathAst {
    Number(f64),
    Var(String),
    NodeVoltage(String),
    UnaryNeg(Box<MathAst>),
    Binary {
        op: MathOp,
        left: Box<MathAst>,
        right: Box<MathAst>,
    },
    Call {
        func: String,
        args: Vec<MathAst>,
    },
}

impl MathAst {
    /// Evaluates the AST node given instantaneous time t and node voltages.
    pub fn eval(&self, t: f64, voltages: &HashMap<String, f64>) -> Result<f64, String> {
        match self {
            Self::Number(val) => Ok(*val),
            Self::Var(name) => {
                let lower = name.to_lowercase();
                if lower == "t" {
                    Ok(t)
                } else if lower == "pi" {
                    Ok(PI)
                } else if lower == "e" {
                    Ok(E)
                } else if let Some(&v) = voltages.get(name) {
                    Ok(v)
                } else if let Some(&v) = voltages.get(&name.to_uppercase()) {
                    Ok(v)
                } else {
                    Ok(0.0)
                }
            }
            Self::NodeVoltage(net) => {
                if let Some(&v) = voltages.get(net) {
                    Ok(v)
                } else if let Some(&v) = voltages.get(&net.to_uppercase()) {
                    Ok(v)
                } else {
                    Ok(0.0)
                }
            }
            Self::UnaryNeg(inner) => Ok(-inner.eval(t, voltages)?),
            Self::Binary { op, left, right } => {
                let l = left.eval(t, voltages)?;
                let r = right.eval(t, voltages)?;
                match op {
                    MathOp::Add => Ok(l + r),
                    MathOp::Sub => Ok(l - r),
                    MathOp::Mul => Ok(l * r),
                    MathOp::Div => {
                        if r.abs() < 1e-15 {
                            Ok(0.0)
                        } else {
                            Ok(l / r)
                        }
                    }
                    MathOp::Mod => {
                        if r.abs() < 1e-15 {
                            Ok(0.0)
                        } else {
                            Ok(l % r)
                        }
                    }
                    MathOp::Pow => Ok(l.powf(r)),
                }
            }
            Self::Call { func, args } => {
                let lower = func.to_lowercase();
                if lower == "v" {
                    if let Some(arg) = args.first() {
                        match arg {
                            MathAst::Var(name) | MathAst::NodeVoltage(name) => {
                                return if let Some(&val) = voltages.get(name) {
                                    Ok(val)
                                } else if let Some(&val) = voltages.get(&name.to_uppercase()) {
                                    Ok(val)
                                } else {
                                    Ok(0.0)
                                };
                            }
                            _ => {
                                let evaluated_val = arg.eval(t, voltages)?;
                                return Ok(evaluated_val);
                            }
                        }
                    }
                    return Ok(0.0);
                }

                let evaluated_args: Result<Vec<f64>, String> =
                    args.iter().map(|a| a.eval(t, voltages)).collect();
                let evaluated = evaluated_args?;

                match lower.as_str() {
                    "sin" => Ok(evaluated.first().copied().unwrap_or(0.0).sin()),
                    "cos" => Ok(evaluated.first().copied().unwrap_or(0.0).cos()),
                    "tan" => Ok(evaluated.first().copied().unwrap_or(0.0).tan()),
                    "asin" => Ok(evaluated.first().copied().unwrap_or(0.0).asin()),
                    "acos" => Ok(evaluated.first().copied().unwrap_or(0.0).acos()),
                    "atan" => Ok(evaluated.first().copied().unwrap_or(0.0).atan()),
                    "sinh" => Ok(evaluated.first().copied().unwrap_or(0.0).sinh()),
                    "cosh" => Ok(evaluated.first().copied().unwrap_or(0.0).cosh()),
                    "tanh" => Ok(evaluated.first().copied().unwrap_or(0.0).tanh()),
                    "sqrt" => Ok(evaluated.first().copied().unwrap_or(0.0).max(0.0).sqrt()),
                    "abs" => Ok(evaluated.first().copied().unwrap_or(0.0).abs()),
                    "exp" => Ok(evaluated.first().copied().unwrap_or(0.0).exp()),
                    "log" | "ln" => Ok(evaluated.first().copied().unwrap_or(0.0).max(1e-15).ln()),
                    "log10" => Ok(evaluated.first().copied().unwrap_or(0.0).max(1e-15).log10()),
                    "floor" => Ok(evaluated.first().copied().unwrap_or(0.0).floor()),
                    "ceil" => Ok(evaluated.first().copied().unwrap_or(0.0).ceil()),
                    "min" => {
                        let a = evaluated.first().copied().unwrap_or(0.0);
                        let b = evaluated.get(1).copied().unwrap_or(a);
                        Ok(a.min(b))
                    }
                    "max" => {
                        let a = evaluated.first().copied().unwrap_or(0.0);
                        let b = evaluated.get(1).copied().unwrap_or(a);
                        Ok(a.max(b))
                    }
                    unknown => Err(format!("Unknown math function: {}", unknown)),
                }
            }
        }
    }
}

/// Tokenizer token for mathematical expressions.
#[derive(Debug, Clone, PartialEq)]
enum MathToken {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Comma,
    Eof,
}

/// Tokenizes mathematical expression string.
fn tokenize(expr: &str) -> Result<Vec<MathToken>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        if ch.is_ascii_digit() || (ch == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                i += 1;
                if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                    i += 1;
                }
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let s: String = chars[start..i].iter().collect();
            let val = s.parse::<f64>().map_err(|e| format!("Invalid number '{}': {}", s, e))?;
            tokens.push(MathToken::Number(val));
            continue;
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            tokens.push(MathToken::Ident(s));
            continue;
        }

        match ch {
            '+' => tokens.push(MathToken::Plus),
            '-' => tokens.push(MathToken::Minus),
            '*' => tokens.push(MathToken::Star),
            '/' => tokens.push(MathToken::Slash),
            '%' => tokens.push(MathToken::Percent),
            '^' => tokens.push(MathToken::Caret),
            '(' => tokens.push(MathToken::LParen),
            ')' => tokens.push(MathToken::RParen),
            ',' => tokens.push(MathToken::Comma),
            other => return Err(format!("Unexpected character in expression: '{}'", other)),
        }
        i += 1;
    }

    tokens.push(MathToken::Eof);
    Ok(tokens)
}

/// Recursive descent parser for mathematical expressions.
struct Parser {
    tokens: Vec<MathToken>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<MathToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &MathToken {
        self.tokens.get(self.pos).unwrap_or(&MathToken::Eof)
    }

    fn advance(&mut self) -> MathToken {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            tok
        } else {
            MathToken::Eof
        }
    }

    fn parse_expression(&mut self) -> Result<MathAst, String> {
        self.parse_add_sub()
    }

    fn parse_add_sub(&mut self) -> Result<MathAst, String> {
        let mut left = self.parse_mul_div()?;
        loop {
            match self.peek() {
                MathToken::Plus => {
                    self.advance();
                    let right = self.parse_mul_div()?;
                    left = MathAst::Binary {
                        op: MathOp::Add,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                MathToken::Minus => {
                    self.advance();
                    let right = self.parse_mul_div()?;
                    left = MathAst::Binary {
                        op: MathOp::Sub,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_mul_div(&mut self) -> Result<MathAst, String> {
        let mut left = self.parse_power()?;
        loop {
            match self.peek() {
                MathToken::Star => {
                    self.advance();
                    let right = self.parse_power()?;
                    left = MathAst::Binary {
                        op: MathOp::Mul,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                MathToken::Slash => {
                    self.advance();
                    let right = self.parse_power()?;
                    left = MathAst::Binary {
                        op: MathOp::Div,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                MathToken::Percent => {
                    self.advance();
                    let right = self.parse_power()?;
                    left = MathAst::Binary {
                        op: MathOp::Mod,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<MathAst, String> {
        let left = self.parse_unary()?;
        if let MathToken::Caret = self.peek() {
            self.advance();
            let right = self.parse_power()?; // right-associative
            Ok(MathAst::Binary {
                op: MathOp::Pow,
                left: Box::new(left),
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_unary(&mut self) -> Result<MathAst, String> {
        match self.peek() {
            MathToken::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(MathAst::UnaryNeg(Box::new(operand)))
            }
            MathToken::Plus => {
                self.advance();
                self.parse_unary()
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<MathAst, String> {
        match self.advance() {
            MathToken::Number(val) => Ok(MathAst::Number(val)),
            MathToken::Ident(name) => {
                if let MathToken::LParen = self.peek() {
                    self.advance(); // consume '('
                    if name.eq_ignore_ascii_case("v") {
                        // SPICE node voltage syntax: V(VIN)
                        let net_token = self.advance();
                        let net_name = match net_token {
                            MathToken::Ident(n) => n,
                            MathToken::Number(num) => format!("{}", num as i64),
                            other => return Err(format!("Expected net name in V(...), got {:?}", other)),
                        };
                        if self.advance() != MathToken::RParen {
                            return Err(format!("Expected ')' closing V({})", net_name));
                        }
                        Ok(MathAst::NodeVoltage(net_name))
                    } else {
                        // Math function call: e.g. sin(...)
                        let mut args = Vec::new();
                        if let MathToken::RParen = self.peek() {
                            self.advance();
                        } else {
                            loop {
                                args.push(self.parse_expression()?);
                                match self.peek() {
                                    MathToken::Comma => {
                                        self.advance();
                                    }
                                    MathToken::RParen => {
                                        self.advance();
                                        break;
                                    }
                                    other => return Err(format!("Expected ',' or ')' in function call, got {:?}", other)),
                                }
                            }
                        }
                        Ok(MathAst::Call { func: name, args })
                    }
                } else {
                    Ok(MathAst::Var(name))
                }
            }
            MathToken::LParen => {
                let expr = self.parse_expression()?;
                if self.advance() != MathToken::RParen {
                    return Err("Expected ')' matching '('".to_string());
                }
                Ok(expr)
            }
            other => Err(format!("Unexpected token in expression: {:?}", other)),
        }
    }
}

/// Parses a mathematical expression string into an AST.
pub fn parse_expression(expr: &str) -> Result<MathAst, String> {
    let tokens = tokenize(expr)?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_expression()?;
    if *parser.peek() != MathToken::Eof {
        return Err(format!("Unexpected trailing token: {:?}", parser.peek()));
    }
    Ok(ast)
}

/// Evaluates a mathematical expression across time samples [t_start, t_stop].
pub fn evaluate_expression(
    expr: &str,
    t_start: f64,
    t_stop: f64,
    num_samples: usize,
    voltages: &HashMap<String, f64>,
) -> Result<Vec<[f64; 2]>, String> {
    if num_samples < 2 {
        return Err("num_samples must be at least 2".to_string());
    }
    let ast = parse_expression(expr)?;
    let dt = (t_stop - t_start) / (num_samples - 1) as f64;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = t_start + i as f64 * dt;
        let y = ast.eval(t, voltages)?;
        samples.push([t, y]);
    }

    Ok(samples)
}

/// Generates a `WaveformTrace` for the virtual oscilloscope from an expression.
pub fn generate_trace(
    name: &str,
    color: Color32,
    expr: &str,
    t_start: f64,
    t_stop: f64,
    num_samples: usize,
    voltages: &HashMap<String, f64>,
) -> Result<WaveformTrace, String> {
    let samples = evaluate_expression(expr, t_start, t_stop, num_samples, voltages)?;
    let mut trace = WaveformTrace::new(name, color);
    trace.samples = samples;
    Ok(trace)
}

/// Mathematical Expression Waveform Grapher configuration and execution manager.
#[derive(Debug, Clone)]
pub struct ExpressionGrapher {
    pub t_start: f64,
    pub t_stop: f64,
    pub num_samples: usize,
    pub node_voltages: HashMap<String, f64>,
}

impl Default for ExpressionGrapher {
    fn default() -> Self {
        Self {
            t_start: 0.0,
            t_stop: 0.003, // 3 ms default window
            num_samples: 1000,
            node_voltages: HashMap::new(),
        }
    }
}

impl ExpressionGrapher {
    /// Creates a new ExpressionGrapher with standard time window.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets circuit node voltages for SPICE-style expression referencing.
    pub fn with_voltages(mut self, voltages: HashMap<String, f64>) -> Self {
        self.node_voltages = voltages;
        self
    }

    /// Sets time window [t_start, t_stop] and sample resolution.
    pub fn with_time_range(mut self, t_start: f64, t_stop: f64, num_samples: usize) -> Self {
        self.t_start = t_start;
        self.t_stop = t_stop;
        self.num_samples = num_samples;
        self
    }

    /// Generates a WaveformTrace from the provided expression.
    pub fn plot(&self, name: &str, color: Color32, expr: &str) -> Result<WaveformTrace, String> {
        generate_trace(
            name,
            color,
            expr,
            self.t_start,
            self.t_stop,
            self.num_samples,
            &self.node_voltages,
        )
    }
}

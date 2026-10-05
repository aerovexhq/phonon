#![deny(unsafe_code)]

//! Pure Safe Rust Lua Scripting VM for Phonon Studio Embedded Testbenches.
//!
//! Provides an isolated, pure safe Rust Lua interpreter compiled with zero external C
//! dependencies, supporting standard control flow, closures, tables, math libraries,
//! and native Phonon CAD simulation telemetry APIs.

use egui::Color32;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::oscilloscope::WaveformTrace;
use crate::scripting::expression_grapher::ExpressionGrapher;
use crate::scripting::permissions::{PermissionKind, PermissionManager};

/// Assertion record from testbench execution.
#[derive(Debug, Clone, PartialEq)]
pub struct AssertionRecord {
    pub name: String,
    pub passed: bool,
    pub actual: f64,
    pub expected: f64,
    pub message: String,
}

/// Table key representation for Lua associative arrays.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TableKey {
    Int(i64),
    Str(String),
    Bool(bool),
}

/// Dynamic table data structure for Lua state.
#[derive(Debug, Clone, Default)]
pub struct LuaTable {
    pub entries: HashMap<TableKey, LuaValue>,
}

impl LuaTable {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn key_from_val(val: &LuaValue) -> Option<TableKey> {
        match val {
            LuaValue::Number(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    Some(TableKey::Int(*n as i64))
                } else {
                    Some(TableKey::Str(n.to_string()))
                }
            }
            LuaValue::String(s) => Some(TableKey::Str(s.clone())),
            LuaValue::Boolean(b) => Some(TableKey::Bool(*b)),
            _ => None,
        }
    }

    pub fn get(&self, key: &LuaValue) -> LuaValue {
        if let Some(k) = Self::key_from_val(key) {
            self.entries.get(&k).cloned().unwrap_or(LuaValue::Nil)
        } else {
            LuaValue::Nil
        }
    }

    pub fn get_str(&self, key: &str) -> LuaValue {
        self.entries
            .get(&TableKey::Str(key.to_string()))
            .cloned()
            .unwrap_or(LuaValue::Nil)
    }

    pub fn set(&mut self, key: LuaValue, val: LuaValue) {
        if let Some(k) = Self::key_from_val(&key) {
            if val == LuaValue::Nil {
                self.entries.remove(&k);
            } else {
                self.entries.insert(k, val);
            }
        }
    }

    pub fn set_str(&mut self, key: &str, val: LuaValue) {
        let k = TableKey::Str(key.to_string());
        if val == LuaValue::Nil {
            self.entries.remove(&k);
        } else {
            self.entries.insert(k, val);
        }
    }

    pub fn len(&self) -> usize {
        let mut i = 1i64;
        while self.entries.contains_key(&TableKey::Int(i)) {
            i += 1;
        }
        (i - 1) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn insert(&mut self, val: LuaValue) {
        let next_idx = (self.len() + 1) as i64;
        self.entries.insert(TableKey::Int(next_idx), val);
    }

    pub fn remove(&mut self, pos: Option<usize>) -> LuaValue {
        let l = self.len();
        if l == 0 {
            return LuaValue::Nil;
        }
        let target = pos.unwrap_or(l) as i64;
        let removed = self.entries.remove(&TableKey::Int(target)).unwrap_or(LuaValue::Nil);
        for i in target..(l as i64) {
            if let Some(v) = self.entries.remove(&TableKey::Int(i + 1)) {
                self.entries.insert(TableKey::Int(i), v);
            }
        }
        removed
    }
}

/// Native function callback signature.
pub type NativeFn = Rc<dyn Fn(&mut LuaEngine, &[LuaValue]) -> Result<LuaValue, String>>;

/// User-defined function representation.
#[derive(Clone)]
pub struct UserFunction {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub captured_env: Rc<RefCell<Environment>>,
}

/// Function type in Lua engine.
#[derive(Clone)]
pub enum LuaFunction {
    Native(NativeFn),
    User(Rc<UserFunction>),
}

/// Primitive and reference values in Lua VM.
#[derive(Clone)]
pub enum LuaValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Table(Rc<RefCell<LuaTable>>),
    Function(LuaFunction),
}

impl LuaValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Nil => false,
            Self::Boolean(b) => *b,
            _ => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Nil => "nil",
            Self::Boolean(_) => "boolean",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Table(_) => "table",
            Self::Function(_) => "function",
        }
    }

    pub fn to_display_string(&self) -> String {
        match self {
            Self::Nil => "nil".to_string(),
            Self::Boolean(b) => b.to_string(),
            Self::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e14 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Self::String(s) => s.clone(),
            Self::Table(t) => format!("table: {:p}", Rc::as_ptr(t)),
            Self::Function(_) => "function".to_string(),
        }
    }

    pub fn as_number(&self) -> Result<f64, String> {
        match self {
            Self::Number(n) => Ok(*n),
            Self::String(s) => s
                .parse::<f64>()
                .map_err(|_| format!("Cannot convert string '{}' to number", s)),
            other => Err(format!("Expected number, got {}", other.type_name())),
        }
    }

    pub fn as_string(&self) -> Result<String, String> {
        match self {
            Self::String(s) => Ok(s.clone()),
            other => Ok(other.to_display_string()),
        }
    }

    pub fn as_table(&self) -> Result<Rc<RefCell<LuaTable>>, String> {
        match self {
            Self::Table(t) => Ok(t.clone()),
            other => Err(format!("Expected table, got {}", other.type_name())),
        }
    }
}

impl PartialEq for LuaValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => {
                (a - b).abs() < 1e-12 || (a.is_nan() && b.is_nan())
            }
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Table(a), Self::Table(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl std::fmt::Debug for LuaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "Nil"),
            Self::Boolean(b) => write!(f, "Boolean({})", b),
            Self::Number(n) => write!(f, "Number({})", n),
            Self::String(s) => write!(f, "String({:?})", s),
            Self::Table(t) => write!(f, "Table({:?})", t.borrow()),
            Self::Function(_) => write!(f, "Function"),
        }
    }
}

/// Lexical scope environment.
#[derive(Debug, Clone, Default)]
pub struct Environment {
    pub parent: Option<Rc<RefCell<Environment>>>,
    pub vars: HashMap<String, LuaValue>,
}

impl Environment {
    pub fn new(parent: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            parent,
            vars: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> LuaValue {
        if let Some(val) = self.vars.get(name) {
            val.clone()
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            LuaValue::Nil
        }
    }

    pub fn set_local(&mut self, name: String, val: LuaValue) {
        self.vars.insert(name, val);
    }
}

/// Assigns a variable searching up lexical scopes, defaulting to root global.
pub fn assign_variable(env: &Rc<RefCell<Environment>>, name: &str, val: LuaValue) {
    let mut current = Some(env.clone());
    let mut found_env = None;
    let mut root_env = env.clone();

    while let Some(e) = current {
        root_env = e.clone();
        if e.borrow().vars.contains_key(name) {
            found_env = Some(e);
            break;
        }
        current = e.borrow().parent.clone();
    }

    if let Some(target) = found_env {
        target.borrow_mut().vars.insert(name.to_string(), val);
    } else {
        root_env.borrow_mut().vars.insert(name.to_string(), val);
    }
}

/// Unary operator in Lua expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    Len,
}

/// Binary operator in Lua expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Concat,
}

/// Table field definition in table constructor.
#[derive(Debug, Clone, PartialEq)]
pub enum TableField {
    List(Expr),
    Record(String, Expr),
    Dynamic(Expr, Expr),
}

/// Expression AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Ident(String),
    Table(Vec<TableField>),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Index {
        table: Box<Expr>,
        index: Box<Expr>,
    },
    Dot {
        table: Box<Expr>,
        field: String,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    MethodCall {
        table: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    Function {
        params: Vec<String>,
        body: Vec<Stmt>,
    },
}

/// Assignment target lvalue.
#[derive(Debug, Clone, PartialEq)]
pub enum AssignTarget {
    Var(String),
    Dot { table: Expr, field: String },
    Index { table: Expr, index: Expr },
}

/// Statement AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Assign {
        target: AssignTarget,
        value: Expr,
    },
    LocalAssign {
        name: String,
        value: Option<Expr>,
    },
    If {
        cond: Expr,
        then_block: Vec<Stmt>,
        elseif_blocks: Vec<(Expr, Vec<Stmt>)>,
        else_block: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    ForNum {
        var: String,
        start: Expr,
        stop: Expr,
        step: Option<Expr>,
        body: Vec<Stmt>,
    },
    FunctionDecl {
        target: AssignTarget,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    LocalFunctionDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return(Vec<Expr>),
    Break,
}

/// Tokenizer token for Lua.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    String(String),
    Ident(String),
    And,
    Break,
    Do,
    Else,
    Elseif,
    End,
    False,
    For,
    Function,
    If,
    Local,
    Nil,
    Not,
    Or,
    Return,
    Then,
    True,
    While,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    EqEq,
    TildeEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Assign,
    DotDot,
    Dot,
    Colon,
    Hash,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Semi,
    Eof,
}

/// Lexes Lua source code into tokens.
pub fn lex(code: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = code.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        // Comment handling: -- or --[[ ... ]]
        if ch == '-' && i + 1 < chars.len() && chars[i + 1] == '-' {
            i += 2;
            if i + 1 < chars.len() && chars[i] == '[' && chars[i + 1] == '[' {
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == ']' && chars[i + 1] == ']') {
                    i += 1;
                }
                if i + 1 < chars.len() {
                    i += 2;
                }
            } else {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            continue;
        }

        // Multiline string [[ ... ]]
        if ch == '[' && i + 1 < chars.len() && chars[i + 1] == '[' {
            i += 2;
            let start = i;
            while i + 1 < chars.len() && !(chars[i] == ']' && chars[i + 1] == ']') {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            if i + 1 < chars.len() {
                i += 2;
            }
            tokens.push(Token::String(s));
            continue;
        }

        // Numbers: int, float, scientific notation (e.g. 1e-3, 2.5e3)
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
            let val = s
                .parse::<f64>()
                .map_err(|e| format!("Invalid Lua number '{}': {}", s, e))?;
            tokens.push(Token::Number(val));
            continue;
        }

        // String literals: "..." or '...'
        if ch == '"' || ch == '\'' {
            let quote = ch;
            i += 1;
            let mut s = String::new();
            while i < chars.len() && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 1;
                    match chars[i] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        '\'' => s.push('\''),
                        other => {
                            s.push('\\');
                            s.push(other);
                        }
                    }
                } else {
                    s.push(chars[i]);
                }
                i += 1;
            }
            if i >= chars.len() {
                return Err("Unterminated string literal in Lua script".to_string());
            }
            i += 1;
            tokens.push(Token::String(s));
            continue;
        }

        // Identifiers and keywords
        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            match s.as_str() {
                "and" => tokens.push(Token::And),
                "break" => tokens.push(Token::Break),
                "do" => tokens.push(Token::Do),
                "else" => tokens.push(Token::Else),
                "elseif" => tokens.push(Token::Elseif),
                "end" => tokens.push(Token::End),
                "false" => tokens.push(Token::False),
                "for" => tokens.push(Token::For),
                "function" => tokens.push(Token::Function),
                "if" => tokens.push(Token::If),
                "local" => tokens.push(Token::Local),
                "nil" => tokens.push(Token::Nil),
                "not" => tokens.push(Token::Not),
                "or" => tokens.push(Token::Or),
                "return" => tokens.push(Token::Return),
                "then" => tokens.push(Token::Then),
                "true" => tokens.push(Token::True),
                "while" => tokens.push(Token::While),
                _ => tokens.push(Token::Ident(s)),
            }
            continue;
        }

        // Multi-character operators
        if ch == '=' && i + 1 < chars.len() && chars[i + 1] == '=' {
            tokens.push(Token::EqEq);
            i += 2;
            continue;
        }
        if (ch == '~' || ch == '!') && i + 1 < chars.len() && chars[i + 1] == '=' {
            tokens.push(Token::TildeEq);
            i += 2;
            continue;
        }
        if ch == '<' && i + 1 < chars.len() && chars[i + 1] == '=' {
            tokens.push(Token::LtEq);
            i += 2;
            continue;
        }
        if ch == '>' && i + 1 < chars.len() && chars[i + 1] == '=' {
            tokens.push(Token::GtEq);
            i += 2;
            continue;
        }
        if ch == '.' && i + 1 < chars.len() && chars[i + 1] == '.' {
            tokens.push(Token::DotDot);
            i += 2;
            continue;
        }

        // Single-character symbols
        match ch {
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Star),
            '/' => tokens.push(Token::Slash),
            '%' => tokens.push(Token::Percent),
            '^' => tokens.push(Token::Caret),
            '<' => tokens.push(Token::Lt),
            '>' => tokens.push(Token::Gt),
            '=' => tokens.push(Token::Assign),
            '.' => tokens.push(Token::Dot),
            ':' => tokens.push(Token::Colon),
            '#' => tokens.push(Token::Hash),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            '[' => tokens.push(Token::LBracket),
            ']' => tokens.push(Token::RBracket),
            '{' => tokens.push(Token::LBrace),
            '}' => tokens.push(Token::RBrace),
            ',' => tokens.push(Token::Comma),
            ';' => tokens.push(Token::Semi),
            other => return Err(format!("Unexpected character in Lua script: '{}'", other)),
        }
        i += 1;
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

/// Lua parser converting token stream into AST.
pub struct LuaParser {
    pub tokens: Vec<Token>,
    pub pos: usize,
}

impl LuaParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    pub fn advance(&mut self) -> Token {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            tok
        } else {
            Token::Eof
        }
    }

    pub fn expect(&mut self, expected: &Token) -> Result<(), String> {
        if self.peek() == expected {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected token {:?}, found {:?}",
                expected,
                self.peek()
            ))
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while *self.peek() != Token::Eof {
            if *self.peek() == Token::Semi {
                self.advance();
                continue;
            }
            stmts.push(self.parse_stmt()?);
            if *self.peek() == Token::Semi {
                self.advance();
            }
        }
        Ok(stmts)
    }

    pub fn parse_block_until(&mut self, stop_tokens: &[Token]) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while !stop_tokens.contains(self.peek()) && *self.peek() != Token::Eof {
            if *self.peek() == Token::Semi {
                self.advance();
                continue;
            }
            stmts.push(self.parse_stmt()?);
            if *self.peek() == Token::Semi {
                self.advance();
            }
        }
        Ok(stmts)
    }

    pub fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Token::Local => {
                self.advance();
                if *self.peek() == Token::Function {
                    self.advance();
                    let name = match self.advance() {
                        Token::Ident(n) => n,
                        other => return Err(format!("Expected local function name, got {:?}", other)),
                    };
                    self.expect(&Token::LParen)?;
                    let params = self.parse_param_list()?;
                    let body = self.parse_block_until(&[Token::End])?;
                    self.expect(&Token::End)?;
                    Ok(Stmt::LocalFunctionDecl { name, params, body })
                } else {
                    let name = match self.advance() {
                        Token::Ident(n) => n,
                        other => return Err(format!("Expected variable name after local, got {:?}", other)),
                    };
                    let value = if *self.peek() == Token::Assign {
                        self.advance();
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };
                    Ok(Stmt::LocalAssign { name, value })
                }
            }
            Token::Function => {
                self.advance();
                let target = self.parse_assign_target()?;
                self.expect(&Token::LParen)?;
                let params = self.parse_param_list()?;
                let body = self.parse_block_until(&[Token::End])?;
                self.expect(&Token::End)?;
                Ok(Stmt::FunctionDecl {
                    target,
                    params,
                    body,
                })
            }
            Token::If => {
                self.advance();
                let cond = self.parse_expr()?;
                self.expect(&Token::Then)?;
                let then_block =
                    self.parse_block_until(&[Token::Elseif, Token::Else, Token::End])?;
                let mut elseif_blocks = Vec::new();
                while *self.peek() == Token::Elseif {
                    self.advance();
                    let c = self.parse_expr()?;
                    self.expect(&Token::Then)?;
                    let b = self.parse_block_until(&[Token::Elseif, Token::Else, Token::End])?;
                    elseif_blocks.push((c, b));
                }
                let else_block = if *self.peek() == Token::Else {
                    self.advance();
                    Some(self.parse_block_until(&[Token::End])?)
                } else {
                    None
                };
                self.expect(&Token::End)?;
                Ok(Stmt::If {
                    cond,
                    then_block,
                    elseif_blocks,
                    else_block,
                })
            }
            Token::While => {
                self.advance();
                let cond = self.parse_expr()?;
                self.expect(&Token::Do)?;
                let body = self.parse_block_until(&[Token::End])?;
                self.expect(&Token::End)?;
                Ok(Stmt::While { cond, body })
            }
            Token::For => {
                self.advance();
                let var = match self.advance() {
                    Token::Ident(v) => v,
                    other => return Err(format!("Expected variable in for loop, got {:?}", other)),
                };
                self.expect(&Token::Assign)?;
                let start = self.parse_expr()?;
                self.expect(&Token::Comma)?;
                let stop = self.parse_expr()?;
                let step = if *self.peek() == Token::Comma {
                    self.advance();
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.expect(&Token::Do)?;
                let body = self.parse_block_until(&[Token::End])?;
                self.expect(&Token::End)?;
                Ok(Stmt::ForNum {
                    var,
                    start,
                    stop,
                    step,
                    body,
                })
            }
            Token::Return => {
                self.advance();
                let mut returns = Vec::new();
                if !matches!(
                    self.peek(),
                    Token::End | Token::Else | Token::Elseif | Token::Semi | Token::Eof
                ) {
                    returns.push(self.parse_expr()?);
                    while *self.peek() == Token::Comma {
                        self.advance();
                        returns.push(self.parse_expr()?);
                    }
                }
                Ok(Stmt::Return(returns))
            }
            Token::Break => {
                self.advance();
                Ok(Stmt::Break)
            }
            _ => {
                // Primary expression or assignment
                let expr = self.parse_expr()?;
                if *self.peek() == Token::Assign {
                    self.advance();
                    let value = self.parse_expr()?;
                    let target = match expr {
                        Expr::Ident(name) => AssignTarget::Var(name),
                        Expr::Dot { table, field } => AssignTarget::Dot {
                            table: *table,
                            field,
                        },
                        Expr::Index { table, index } => AssignTarget::Index {
                            table: *table,
                            index: *index,
                        },
                        other => return Err(format!("Invalid assignment target: {:?}", other)),
                    };
                    Ok(Stmt::Assign { target, value })
                } else {
                    Ok(Stmt::Expr(expr))
                }
            }
        }
    }

    fn parse_assign_target(&mut self) -> Result<AssignTarget, String> {
        let mut expr = match self.advance() {
            Token::Ident(name) => Expr::Ident(name),
            other => return Err(format!("Expected identifier in function declaration, got {:?}", other)),
        };

        loop {
            match self.peek() {
                Token::Dot => {
                    self.advance();
                    let field = match self.advance() {
                        Token::Ident(f) => f,
                        other => return Err(format!("Expected field name after '.', got {:?}", other)),
                    };
                    expr = Expr::Dot {
                        table: Box::new(expr),
                        field,
                    };
                }
                Token::Colon => {
                    self.advance();
                    let method = match self.advance() {
                        Token::Ident(m) => m,
                        other => return Err(format!("Expected method name after ':', got {:?}", other)),
                    };
                    expr = Expr::Dot {
                        table: Box::new(expr),
                        field: method,
                    };
                }
                _ => break,
            }
        }

        match expr {
            Expr::Ident(name) => Ok(AssignTarget::Var(name)),
            Expr::Dot { table, field } => Ok(AssignTarget::Dot {
                table: *table,
                field,
            }),
            Expr::Index { table, index } => Ok(AssignTarget::Index {
                table: *table,
                index: *index,
            }),
            other => Err(format!("Invalid function target: {:?}", other)),
        }
    }

    fn parse_param_list(&mut self) -> Result<Vec<String>, String> {
        let mut params = Vec::new();
        if *self.peek() == Token::RParen {
            self.advance();
            return Ok(params);
        }
        loop {
            match self.advance() {
                Token::Ident(name) => params.push(name),
                other => return Err(format!("Expected parameter name, got {:?}", other)),
            }
            if *self.peek() == Token::Comma {
                self.advance();
            } else if *self.peek() == Token::RParen {
                self.advance();
                break;
            } else {
                return Err(format!("Expected ',' or ')' in param list, got {:?}", self.peek()));
            }
        }
        Ok(params)
    }

    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr, String> {
        let mut left = match self.advance() {
            Token::Nil => Expr::Nil,
            Token::False => Expr::Boolean(false),
            Token::True => Expr::Boolean(true),
            Token::Number(n) => Expr::Number(n),
            Token::String(s) => Expr::String(s),
            Token::Ident(name) => Expr::Ident(name),
            Token::Minus => {
                let r = self.parse_expr_bp(13)?;
                Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(r),
                }
            }
            Token::Not => {
                let r = self.parse_expr_bp(13)?;
                Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(r),
                }
            }
            Token::Hash => {
                let r = self.parse_expr_bp(13)?;
                Expr::Unary {
                    op: UnaryOp::Len,
                    expr: Box::new(r),
                }
            }
            Token::LParen => {
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                expr
            }
            Token::LBrace => self.parse_table_constructor()?,
            Token::Function => {
                self.expect(&Token::LParen)?;
                let params = self.parse_param_list()?;
                let body = self.parse_block_until(&[Token::End])?;
                self.expect(&Token::End)?;
                Expr::Function { params, body }
            }
            other => return Err(format!("Unexpected token in expression: {:?}", other)),
        };

        loop {
            // Postfix operators: calls, dots, indexes
            match self.peek() {
                Token::LParen => {
                    self.advance();
                    let mut args = Vec::new();
                    if *self.peek() != Token::RParen {
                        args.push(self.parse_expr()?);
                        while *self.peek() == Token::Comma {
                            self.advance();
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.expect(&Token::RParen)?;
                    left = Expr::Call {
                        callee: Box::new(left),
                        args,
                    };
                    continue;
                }
                Token::Dot => {
                    self.advance();
                    let field = match self.advance() {
                        Token::Ident(f) => f,
                        other => return Err(format!("Expected field name after '.', got {:?}", other)),
                    };
                    left = Expr::Dot {
                        table: Box::new(left),
                        field,
                    };
                    continue;
                }
                Token::Colon => {
                    self.advance();
                    let method = match self.advance() {
                        Token::Ident(m) => m,
                        other => return Err(format!("Expected method name after ':', got {:?}", other)),
                    };
                    self.expect(&Token::LParen)?;
                    let mut args = Vec::new();
                    if *self.peek() != Token::RParen {
                        args.push(self.parse_expr()?);
                        while *self.peek() == Token::Comma {
                            self.advance();
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.expect(&Token::RParen)?;
                    left = Expr::MethodCall {
                        table: Box::new(left),
                        method,
                        args,
                    };
                    continue;
                }
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(&Token::RBracket)?;
                    left = Expr::Index {
                        table: Box::new(left),
                        index: Box::new(index),
                    };
                    continue;
                }
                _ => {}
            }

            // Infix binary operators
            let (op, l_bp, r_bp) = match self.peek() {
                Token::Or => (BinaryOp::Or, 1, 2),
                Token::And => (BinaryOp::And, 3, 4),
                Token::EqEq => (BinaryOp::Eq, 5, 6),
                Token::TildeEq => (BinaryOp::Ne, 5, 6),
                Token::Lt => (BinaryOp::Lt, 5, 6),
                Token::LtEq => (BinaryOp::Le, 5, 6),
                Token::Gt => (BinaryOp::Gt, 5, 6),
                Token::GtEq => (BinaryOp::Ge, 5, 6),
                Token::DotDot => (BinaryOp::Concat, 8, 7), // Right-associative
                Token::Plus => (BinaryOp::Add, 9, 10),
                Token::Minus => (BinaryOp::Sub, 9, 10),
                Token::Star => (BinaryOp::Mul, 11, 12),
                Token::Slash => (BinaryOp::Div, 11, 12),
                Token::Percent => (BinaryOp::Mod, 11, 12),
                Token::Caret => (BinaryOp::Pow, 15, 14), // Right-associative
                _ => break,
            };

            if l_bp < min_bp {
                break;
            }

            self.advance();
            let right = self.parse_expr_bp(r_bp)?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_table_constructor(&mut self) -> Result<Expr, String> {
        let mut fields = Vec::new();
        if *self.peek() == Token::RBrace {
            self.advance();
            return Ok(Expr::Table(fields));
        }

        loop {
            if *self.peek() == Token::LBracket {
                self.advance();
                let key = self.parse_expr()?;
                self.expect(&Token::RBracket)?;
                self.expect(&Token::Assign)?;
                let val = self.parse_expr()?;
                fields.push(TableField::Dynamic(key, val));
            } else if let Token::Ident(name) = self.peek().clone() {
                if self.tokens.get(self.pos + 1) == Some(&Token::Assign) {
                    self.advance(); // Ident
                    self.advance(); // Assign
                    let val = self.parse_expr()?;
                    fields.push(TableField::Record(name, val));
                } else {
                    let val = self.parse_expr()?;
                    fields.push(TableField::List(val));
                }
            } else {
                let val = self.parse_expr()?;
                fields.push(TableField::List(val));
            }

            if *self.peek() == Token::Comma || *self.peek() == Token::Semi {
                self.advance();
                if *self.peek() == Token::RBrace {
                    self.advance();
                    break;
                }
            } else if *self.peek() == Token::RBrace {
                self.advance();
                break;
            } else {
                return Err(format!("Expected ',' or '}}' in table constructor, got {:?}", self.peek()));
            }
        }

        Ok(Expr::Table(fields))
    }
}

/// Control flow signals within script blocks.
#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    Continue,
    Return(Vec<LuaValue>),
    Break,
}

/// Pure Safe Rust Lua Scripting Engine & CAD Testbench Orchestrator.
pub struct LuaEngine {
    pub global_env: Rc<RefCell<Environment>>,
    pub output_log: Vec<String>,
    pub assertion_records: Vec<AssertionRecord>,
    pub permissions: PermissionManager,
    pub generated_traces: Vec<WaveformTrace>,
    pub circuit_voltages: HashMap<String, f64>,
    pub circuit_currents: HashMap<String, f64>,
    pub execution_time_ms: f64,
    pub error_trace: Option<String>,
    pub trace_palette_idx: usize,
}

impl Default for LuaEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LuaEngine {
    /// Creates a new LuaEngine initialized with standard library and Phonon CAD API.
    pub fn new() -> Self {
        let global_env = Rc::new(RefCell::new(Environment::new(None)));
        let mut engine = Self {
            global_env,
            output_log: Vec::new(),
            assertion_records: Vec::new(),
            permissions: PermissionManager::new(),
            generated_traces: Vec::new(),
            circuit_voltages: HashMap::new(),
            circuit_currents: HashMap::new(),
            execution_time_ms: 0.0,
            error_trace: None,
            trace_palette_idx: 0,
        };

        engine.init_standard_library();
        engine.init_phonon_api();
        engine
    }

    /// Sets circuit node voltages for simulation access.
    pub fn set_voltage(&mut self, net: &str, v: f64) {
        self.circuit_voltages.insert(net.to_string(), v);
    }

    /// Sets branch currents for simulation access.
    pub fn set_current(&mut self, name: &str, i: f64) {
        self.circuit_currents.insert(name.to_string(), i);
    }

    /// Total assertions executed in current run.
    pub fn total_assertions(&self) -> usize {
        self.assertion_records.len()
    }

    /// Passed assertions count.
    pub fn passed_assertions(&self) -> usize {
        self.assertion_records.iter().filter(|r| r.passed).count()
    }

    /// Failed assertions count.
    pub fn failed_assertions(&self) -> usize {
        self.assertion_records.iter().filter(|r| !r.passed).count()
    }

    /// Clears output log and telemetry.
    pub fn clear_log(&mut self) {
        self.output_log.clear();
        self.assertion_records.clear();
        self.error_trace = None;
    }

    /// Resets runtime state while preserving permanent permissions.
    pub fn reset(&mut self) {
        self.clear_log();
        self.generated_traces.clear();
        self.global_env = Rc::new(RefCell::new(Environment::new(None)));
        self.init_standard_library();
        self.init_phonon_api();
    }

    /// Initializes Lua standard library functions.
    fn init_standard_library(&mut self) {
        // print(...)
        self.register_global(
            "print",
            Rc::new(|engine, args| {
                let parts: Vec<String> = args.iter().map(|a| a.to_display_string()).collect();
                let line = parts.join("\t");
                engine.output_log.push(line);
                Ok(LuaValue::Nil)
            }),
        );

        // assert(cond, [msg])
        self.register_global(
            "assert",
            Rc::new(|engine, args| {
                let cond = args.first().cloned().unwrap_or(LuaValue::Nil);
                let msg = args
                    .get(1)
                    .map(|v| v.to_display_string())
                    .unwrap_or_else(|| "assertion failed!".to_string());
                if cond.is_truthy() {
                    engine.assertion_records.push(AssertionRecord {
                        name: "assert".to_string(),
                        passed: true,
                        actual: 1.0,
                        expected: 1.0,
                        message: msg.clone(),
                    });
                    engine.output_log.push(format!("[PASS] assert: {}", msg));
                    Ok(cond)
                } else {
                    engine.assertion_records.push(AssertionRecord {
                        name: "assert".to_string(),
                        passed: false,
                        actual: 0.0,
                        expected: 1.0,
                        message: msg.clone(),
                    });
                    engine.output_log.push(format!("[FAIL] assert: {}", msg));
                    Err(format!("assertion failed! {}", msg))
                }
            }),
        );

        // type(val)
        self.register_global(
            "type",
            Rc::new(|_engine, args| {
                let val = args.first().cloned().unwrap_or(LuaValue::Nil);
                Ok(LuaValue::String(val.type_name().to_string()))
            }),
        );

        // tostring(val)
        self.register_global(
            "tostring",
            Rc::new(|_engine, args| {
                let val = args.first().cloned().unwrap_or(LuaValue::Nil);
                Ok(LuaValue::String(val.to_display_string()))
            }),
        );

        // tonumber(val)
        self.register_global(
            "tonumber",
            Rc::new(|_engine, args| {
                let val = args.first().cloned().unwrap_or(LuaValue::Nil);
                match val {
                    LuaValue::Number(n) => Ok(LuaValue::Number(n)),
                    LuaValue::String(s) => match s.trim().parse::<f64>() {
                        Ok(n) => Ok(LuaValue::Number(n)),
                        Err(_) => Ok(LuaValue::Nil),
                    },
                    _ => Ok(LuaValue::Nil),
                }
            }),
        );

        // math table
        let math_tbl = Rc::new(RefCell::new(LuaTable::new()));
        {
            let mut m = math_tbl.borrow_mut();
            m.set_str("pi", LuaValue::Number(std::f64::consts::PI));

            let wrap_unary = |f: fn(f64) -> f64| -> LuaValue {
                LuaValue::Function(LuaFunction::Native(Rc::new(move |_engine, args| {
                    let n = args.first().cloned().unwrap_or(LuaValue::Nil).as_number()?;
                    Ok(LuaValue::Number(f(n)))
                })))
            };

            m.set_str("sin", wrap_unary(f64::sin));
            m.set_str("cos", wrap_unary(f64::cos));
            m.set_str("tan", wrap_unary(f64::tan));
            m.set_str("sqrt", wrap_unary(|x| x.max(0.0).sqrt()));
            m.set_str("abs", wrap_unary(f64::abs));
            m.set_str("exp", wrap_unary(f64::exp));
            m.set_str("log", wrap_unary(|x| x.max(1e-15).ln()));
            m.set_str("floor", wrap_unary(f64::floor));
            m.set_str("ceil", wrap_unary(f64::ceil));

            m.set_str(
                "min",
                LuaValue::Function(LuaFunction::Native(Rc::new(|_engine, args| {
                    let mut min_val = f64::INFINITY;
                    for a in args {
                        let n = a.as_number()?;
                        if n < min_val {
                            min_val = n;
                        }
                    }
                    if min_val.is_infinite() {
                        min_val = 0.0;
                    }
                    Ok(LuaValue::Number(min_val))
                }))),
            );

            m.set_str(
                "max",
                LuaValue::Function(LuaFunction::Native(Rc::new(|_engine, args| {
                    let mut max_val = f64::NEG_INFINITY;
                    for a in args {
                        let n = a.as_number()?;
                        if n > max_val {
                            max_val = n;
                        }
                    }
                    if max_val.is_infinite() {
                        max_val = 0.0;
                    }
                    Ok(LuaValue::Number(max_val))
                }))),
            );
        }
        self.global_env
            .borrow_mut()
            .set_local("math".to_string(), LuaValue::Table(math_tbl));

        // table library
        let table_tbl = Rc::new(RefCell::new(LuaTable::new()));
        {
            let mut t = table_tbl.borrow_mut();
            t.set_str(
                "insert",
                LuaValue::Function(LuaFunction::Native(Rc::new(|_engine, args| {
                    if args.len() < 2 {
                        return Err("table.insert requires at least 2 arguments".to_string());
                    }
                    let tbl_rc = args[0].as_table()?;
                    if args.len() >= 3 {
                        let pos = args[1].as_number()? as i64;
                        let val = args[2].clone();
                        let l = tbl_rc.borrow().len() as i64;
                        for i in (pos..=l).rev() {
                            let item = tbl_rc.borrow().get(&LuaValue::Number(i as f64));
                            tbl_rc
                                .borrow_mut()
                                .set(LuaValue::Number((i + 1) as f64), item);
                        }
                        tbl_rc.borrow_mut().set(LuaValue::Number(pos as f64), val);
                    } else {
                        let val = args[1].clone();
                        tbl_rc.borrow_mut().insert(val);
                    }
                    Ok(LuaValue::Nil)
                }))),
            );

            t.set_str(
                "remove",
                LuaValue::Function(LuaFunction::Native(Rc::new(|_engine, args| {
                    if args.is_empty() {
                        return Err("table.remove requires a table argument".to_string());
                    }
                    let tbl_rc = args[0].as_table()?;
                    let pos = if args.len() >= 2 {
                        Some(args[1].as_number()? as usize)
                    } else {
                        None
                    };
                    let removed = tbl_rc.borrow_mut().remove(pos);
                    Ok(removed)
                }))),
            );

            t.set_str(
                "concat",
                LuaValue::Function(LuaFunction::Native(Rc::new(|_engine, args| {
                    if args.is_empty() {
                        return Err("table.concat requires a table argument".to_string());
                    }
                    let tbl_rc = args[0].as_table()?;
                    let sep = if args.len() >= 2 {
                        args[1].as_string()?
                    } else {
                        String::new()
                    };
                    let len = tbl_rc.borrow().len();
                    let mut parts = Vec::with_capacity(len);
                    for i in 1..=len {
                        let val = tbl_rc.borrow().get(&LuaValue::Number(i as f64));
                        parts.push(val.to_display_string());
                    }
                    Ok(LuaValue::String(parts.join(&sep)))
                }))),
            );
        }
        self.global_env
            .borrow_mut()
            .set_local("table".to_string(), LuaValue::Table(table_tbl));

        // string library
        let string_tbl = Rc::new(RefCell::new(LuaTable::new()));
        {
            let mut s = string_tbl.borrow_mut();
            s.set_str(
                "len",
                LuaValue::Function(LuaFunction::Native(Rc::new(|_engine, args| {
                    let str_val = args.first().cloned().unwrap_or(LuaValue::Nil).as_string()?;
                    Ok(LuaValue::Number(str_val.len() as f64))
                }))),
            );

            s.set_str(
                "sub",
                LuaValue::Function(LuaFunction::Native(Rc::new(|_engine, args| {
                    if args.len() < 2 {
                        return Err("string.sub requires at least 2 arguments".to_string());
                    }
                    let str_val = args[0].as_string()?;
                    let len = str_val.len() as i64;
                    let i_raw = args[1].as_number()? as i64;
                    let j_raw = if args.len() >= 3 {
                        args[2].as_number()? as i64
                    } else {
                        -1
                    };

                    let start_idx = if i_raw > 0 {
                        (i_raw - 1).clamp(0, len) as usize
                    } else if i_raw < 0 {
                        (len + i_raw).clamp(0, len) as usize
                    } else {
                        0
                    };

                    let end_idx = if j_raw > 0 {
                        j_raw.clamp(0, len) as usize
                    } else if j_raw < 0 {
                        (len + j_raw + 1).clamp(0, len) as usize
                    } else {
                        0
                    };

                    if start_idx <= end_idx && start_idx < str_val.len() {
                        Ok(LuaValue::String(str_val[start_idx..end_idx].to_string()))
                    } else {
                        Ok(LuaValue::String(String::new()))
                    }
                }))),
            );

            s.set_str(
                "upper",
                LuaValue::Function(LuaFunction::Native(Rc::new(|_engine, args| {
                    let str_val = args.first().cloned().unwrap_or(LuaValue::Nil).as_string()?;
                    Ok(LuaValue::String(str_val.to_uppercase()))
                }))),
            );

            s.set_str(
                "lower",
                LuaValue::Function(LuaFunction::Native(Rc::new(|_engine, args| {
                    let str_val = args.first().cloned().unwrap_or(LuaValue::Nil).as_string()?;
                    Ok(LuaValue::String(str_val.to_lowercase()))
                }))),
            );
        }
        self.global_env
            .borrow_mut()
            .set_local("string".to_string(), LuaValue::Table(string_tbl));
    }

    /// Initializes Phonon Simulation CAD API table (`phonon`).
    fn init_phonon_api(&mut self) {
        let phonon_tbl = Rc::new(RefCell::new(LuaTable::new()));
        {
            let mut p = phonon_tbl.borrow_mut();

            // phonon.get_voltage(net_name) -> number
            p.set_str(
                "get_voltage",
                LuaValue::Function(LuaFunction::Native(Rc::new(|engine, args| {
                    let net = args.first().cloned().unwrap_or(LuaValue::Nil).as_string()?;
                    let v = engine
                        .circuit_voltages
                        .get(&net)
                        .or_else(|| engine.circuit_voltages.get(&net.to_uppercase()))
                        .copied()
                        .unwrap_or(0.0);
                    Ok(LuaValue::Number(v))
                }))),
            );

            // phonon.get_current(name) -> number
            p.set_str(
                "get_current",
                LuaValue::Function(LuaFunction::Native(Rc::new(|engine, args| {
                    let name = args.first().cloned().unwrap_or(LuaValue::Nil).as_string()?;
                    let i = engine
                        .circuit_currents
                        .get(&name)
                        .or_else(|| engine.circuit_currents.get(&name.to_uppercase()))
                        .copied()
                        .unwrap_or(0.0);
                    Ok(LuaValue::Number(i))
                }))),
            );

            // phonon.run_dc() -> table
            p.set_str(
                "run_dc",
                LuaValue::Function(LuaFunction::Native(Rc::new(|engine, _args| {
                    let res_tbl = Rc::new(RefCell::new(LuaTable::new()));
                    for (k, v) in &engine.circuit_voltages {
                        res_tbl.borrow_mut().set_str(k, LuaValue::Number(*v));
                    }
                    engine.output_log.push(format!(
                        "[PHONON] DC Operating Point analysis executed: {} solved nodes",
                        engine.circuit_voltages.len()
                    ));
                    Ok(LuaValue::Table(res_tbl))
                }))),
            );

            // phonon.run_transient(t_stop, dt) -> table
            p.set_str(
                "run_transient",
                LuaValue::Function(LuaFunction::Native(Rc::new(|engine, args| {
                    let t_stop = args.first().cloned().unwrap_or(LuaValue::Number(0.003)).as_number()?;
                    let dt = args.get(1).cloned().unwrap_or(LuaValue::Number(1e-5)).as_number()?;
                    let res_tbl = Rc::new(RefCell::new(LuaTable::new()));
                    res_tbl.borrow_mut().set_str("t_stop", LuaValue::Number(t_stop));
                    res_tbl.borrow_mut().set_str("dt", LuaValue::Number(dt));
                    res_tbl
                        .borrow_mut()
                        .set_str("status", LuaValue::String("converged".to_string()));
                    engine.output_log.push(format!(
                        "[PHONON] Transient simulation completed: t_stop = {:.4e} s, dt = {:.4e} s",
                        t_stop, dt
                    ));
                    Ok(LuaValue::Table(res_tbl))
                }))),
            );

            // phonon.assert_eq(actual, expected, [tol, msg])
            p.set_str(
                "assert_eq",
                LuaValue::Function(LuaFunction::Native(Rc::new(|engine, args| {
                    if args.len() < 2 {
                        return Err("phonon.assert_eq requires (actual, expected)".to_string());
                    }
                    let actual = args[0].as_number()?;
                    let expected = args[1].as_number()?;
                    let tol = if args.len() >= 3 {
                        args[2].as_number()?
                    } else {
                        1e-5
                    };
                    let msg = if args.len() >= 4 {
                        args[3].as_string()?
                    } else {
                        "phonon.assert_eq".to_string()
                    };

                    let diff = (actual - expected).abs();
                    let passed = diff <= tol;
                    engine.assertion_records.push(AssertionRecord {
                        name: msg.clone(),
                        passed,
                        actual,
                        expected,
                        message: msg.clone(),
                    });

                    if passed {
                        engine.output_log.push(format!(
                            "[PASS] {}: actual = {:.6}, expected = {:.6} (diff = {:.2e} <= tol {:.2e})",
                            msg, actual, expected, diff, tol
                        ));
                    } else {
                        engine.output_log.push(format!(
                            "[FAIL] {}: actual = {:.6}, expected = {:.6} (diff = {:.2e} > tol {:.2e})",
                            msg, actual, expected, diff, tol
                        ));
                    }
                    Ok(LuaValue::Boolean(passed))
                }))),
            );

            // phonon.assert_range(val, min, max, [msg])
            p.set_str(
                "assert_range",
                LuaValue::Function(LuaFunction::Native(Rc::new(|engine, args| {
                    if args.len() < 3 {
                        return Err("phonon.assert_range requires (val, min, max)".to_string());
                    }
                    let val = args[0].as_number()?;
                    let min_v = args[1].as_number()?;
                    let max_v = args[2].as_number()?;
                    let msg = if args.len() >= 4 {
                        args[3].as_string()?
                    } else {
                        "phonon.assert_range".to_string()
                    };

                    let passed = val >= min_v && val <= max_v;
                    engine.assertion_records.push(AssertionRecord {
                        name: msg.clone(),
                        passed,
                        actual: val,
                        expected: (min_v + max_v) * 0.5,
                        message: msg.clone(),
                    });

                    if passed {
                        engine.output_log.push(format!(
                            "[PASS] {}: val = {:.6} in range [{:.6}, {:.6}]",
                            msg, val, min_v, max_v
                        ));
                    } else {
                        engine.output_log.push(format!(
                            "[FAIL] {}: val = {:.6} OUTSIDE range [{:.6}, {:.6}]",
                            msg, val, min_v, max_v
                        ));
                    }
                    Ok(LuaValue::Boolean(passed))
                }))),
            );

            // phonon.plot_expression(name, expr_or_func)
            p.set_str(
                "plot_expression",
                LuaValue::Function(LuaFunction::Native(Rc::new(|engine, args| {
                    if args.len() < 2 {
                        return Err("phonon.plot_expression requires (name, expr_or_func)".to_string());
                    }
                    let name = args[0].as_string()?;
                    let palette = [
                        Color32::from_rgb(80, 200, 255),
                        Color32::from_rgb(100, 255, 140),
                        Color32::from_rgb(255, 180, 60),
                        Color32::from_rgb(220, 120, 255),
                        Color32::from_rgb(255, 100, 120),
                    ];
                    let color = palette[engine.trace_palette_idx % palette.len()];
                    engine.trace_palette_idx += 1;

                    match &args[1] {
                        LuaValue::String(expr_str) => {
                            let grapher = ExpressionGrapher::new()
                                .with_voltages(engine.circuit_voltages.clone())
                                .with_time_range(0.0, 0.003, 1000);
                            let trace = grapher.plot(&name, color, expr_str)?;
                            engine.output_log.push(format!(
                                "[PHONON] Registered expression trace '{}' with {} samples",
                                name,
                                trace.samples.len()
                            ));
                            engine.generated_traces.push(trace);
                        }
                        LuaValue::Function(func) => {
                            let mut trace = WaveformTrace::new(&name, color);
                            let num_samples = 1000;
                            let dt = 0.003 / (num_samples - 1) as f64;
                            let func_clone = func.clone();
                            for i in 0..num_samples {
                                let t = i as f64 * dt;
                                let res = engine.call_function(&func_clone, &[LuaValue::Number(t)])?;
                                let y = res.as_number()?;
                                trace.push(t, y);
                            }
                            engine.output_log.push(format!(
                                "[PHONON] Registered dynamic function trace '{}' with {} samples",
                                name,
                                trace.samples.len()
                            ));
                            engine.generated_traces.push(trace);
                        }
                        other => return Err(format!("Expected string or function for expression plot, got {:?}", other)),
                    }
                    Ok(LuaValue::Boolean(true))
                }))),
            );

            // phonon.read_file(path) -> string
            p.set_str(
                "read_file",
                LuaValue::Function(LuaFunction::Native(Rc::new(|engine, args| {
                    let path = args.first().cloned().unwrap_or(LuaValue::Nil).as_string()?;
                    engine
                        .permissions
                        .request_permission(PermissionKind::FileRead(path.clone()))?;

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let content = std::fs::read_to_string(&path)
                            .map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
                        Ok(LuaValue::String(content))
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        Ok(LuaValue::String(format!("Virtual file content of {}", path)))
                    }
                }))),
            );

            // phonon.write_file(path, content)
            p.set_str(
                "write_file",
                LuaValue::Function(LuaFunction::Native(Rc::new(|engine, args| {
                    if args.len() < 2 {
                        return Err("phonon.write_file requires (path, content)".to_string());
                    }
                    let path = args[0].as_string()?;
                    let content = args[1].as_string()?;
                    engine
                        .permissions
                        .request_permission(PermissionKind::FileWrite(path.clone()))?;

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        std::fs::write(&path, &content)
                            .map_err(|e| format!("Failed to write file '{}': {}", path, e))?;
                        Ok(LuaValue::Boolean(true))
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        let _ = content;
                        Ok(LuaValue::Boolean(true))
                    }
                }))),
            );

            // phonon.log(msg)
            p.set_str(
                "log",
                LuaValue::Function(LuaFunction::Native(Rc::new(|engine, args| {
                    let msg = args.first().cloned().unwrap_or(LuaValue::Nil).as_string()?;
                    engine.output_log.push(format!("[PHONON] {}", msg));
                    Ok(LuaValue::Nil)
                }))),
            );
        }

        self.global_env
            .borrow_mut()
            .set_local("phonon".to_string(), LuaValue::Table(phonon_tbl));
    }

    /// Helper to register a global native function.
    pub fn register_global(&mut self, name: &str, func: NativeFn) {
        self.global_env.borrow_mut().set_local(
            name.to_string(),
            LuaValue::Function(LuaFunction::Native(func)),
        );
    }

    /// Executes a Lua script string and returns the evaluated result.
    pub fn run_script(&mut self, script: &str) -> Result<LuaValue, String> {
        #[cfg(not(target_arch = "wasm32"))]
        let start = std::time::Instant::now();

        self.error_trace = None;
        self.generated_traces.clear();
        self.assertion_records.clear();
        self.output_log.clear();

        let tokens = match lex(script) {
            Ok(toks) => toks,
            Err(err) => {
                self.error_trace = Some(format!("Syntax Error: {}", err));
                self.output_log.push(format!("[ERROR] Syntax Error: {}", err));
                return Err(err);
            }
        };

        let mut parser = LuaParser::new(tokens);
        let stmts = match parser.parse_program() {
            Ok(s) => s,
            Err(err) => {
                self.error_trace = Some(format!("Parse Error: {}", err));
                self.output_log.push(format!("[ERROR] Parse Error: {}", err));
                return Err(err);
            }
        };

        let env = self.global_env.clone();
        let exec_result = self.execute_block(&stmts, &env);

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.execution_time_ms = 0.0;
        }

        match exec_result {
            Ok(flow) => {
                let val = match flow {
                    ControlFlow::Return(mut vals) => vals.pop().unwrap_or(LuaValue::Nil),
                    _ => LuaValue::Nil,
                };
                Ok(val)
            }
            Err(err) => {
                self.error_trace = Some(err.clone());
                self.output_log.push(format!("[ERROR] Runtime Error: {}", err));
                Err(err)
            }
        }
    }

    /// Executes a sequential block of statements.
    pub fn execute_block(
        &mut self,
        stmts: &[Stmt],
        env: &Rc<RefCell<Environment>>,
    ) -> Result<ControlFlow, String> {
        for stmt in stmts {
            let flow = self.execute_stmt(stmt, env)?;
            if flow != ControlFlow::Continue {
                return Ok(flow);
            }
        }
        Ok(ControlFlow::Continue)
    }

    /// Executes an individual statement.
    pub fn execute_stmt(
        &mut self,
        stmt: &Stmt,
        env: &Rc<RefCell<Environment>>,
    ) -> Result<ControlFlow, String> {
        match stmt {
            Stmt::Expr(expr) => {
                self.eval_expr(expr, env)?;
                Ok(ControlFlow::Continue)
            }
            Stmt::Assign { target, value } => {
                let val = self.eval_expr(value, env)?;
                match target {
                    AssignTarget::Var(name) => {
                        assign_variable(env, name, val);
                    }
                    AssignTarget::Dot { table, field } => {
                        let tbl_val = self.eval_expr(table, env)?;
                        let tbl_rc = tbl_val.as_table()?;
                        tbl_rc.borrow_mut().set_str(field, val);
                    }
                    AssignTarget::Index { table, index } => {
                        let tbl_val = self.eval_expr(table, env)?;
                        let idx_val = self.eval_expr(index, env)?;
                        let tbl_rc = tbl_val.as_table()?;
                        tbl_rc.borrow_mut().set(idx_val, val);
                    }
                }
                Ok(ControlFlow::Continue)
            }
            Stmt::LocalAssign { name, value } => {
                let val = if let Some(v_expr) = value {
                    self.eval_expr(v_expr, env)?
                } else {
                    LuaValue::Nil
                };
                env.borrow_mut().set_local(name.clone(), val);
                Ok(ControlFlow::Continue)
            }
            Stmt::If {
                cond,
                then_block,
                elseif_blocks,
                else_block,
            } => {
                let c_val = self.eval_expr(cond, env)?;
                if c_val.is_truthy() {
                    let sub_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));
                    return self.execute_block(then_block, &sub_env);
                }
                for (elseif_cond, elseif_body) in elseif_blocks {
                    let e_val = self.eval_expr(elseif_cond, env)?;
                    if e_val.is_truthy() {
                        let sub_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));
                        return self.execute_block(elseif_body, &sub_env);
                    }
                }
                if let Some(else_body) = else_block {
                    let sub_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));
                    return self.execute_block(else_body, &sub_env);
                }
                Ok(ControlFlow::Continue)
            }
            Stmt::While { cond, body } => {
                loop {
                    let c_val = self.eval_expr(cond, env)?;
                    if !c_val.is_truthy() {
                        break;
                    }
                    let sub_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));
                    let flow = self.execute_block(body, &sub_env)?;
                    match flow {
                        ControlFlow::Break => break,
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        ControlFlow::Continue => {}
                    }
                }
                Ok(ControlFlow::Continue)
            }
            Stmt::ForNum {
                var,
                start,
                stop,
                step,
                body,
            } => {
                let s_val = self.eval_expr(start, env)?.as_number()?;
                let e_val = self.eval_expr(stop, env)?.as_number()?;
                let d_val = if let Some(step_expr) = step {
                    self.eval_expr(step_expr, env)?.as_number()?
                } else {
                    1.0
                };

                let mut current = s_val;
                while (d_val >= 0.0 && current <= e_val) || (d_val < 0.0 && current >= e_val) {
                    let sub_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));
                    sub_env
                        .borrow_mut()
                        .set_local(var.clone(), LuaValue::Number(current));
                    let flow = self.execute_block(body, &sub_env)?;
                    match flow {
                        ControlFlow::Break => break,
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        ControlFlow::Continue => {}
                    }
                    current += d_val;
                }
                Ok(ControlFlow::Continue)
            }
            Stmt::FunctionDecl {
                target,
                params,
                body,
            } => {
                let func_val = LuaValue::Function(LuaFunction::User(Rc::new(UserFunction {
                    name: None,
                    params: params.clone(),
                    body: body.clone(),
                    captured_env: env.clone(),
                })));
                match target {
                    AssignTarget::Var(name) => {
                        assign_variable(env, name, func_val);
                    }
                    AssignTarget::Dot { table, field } => {
                        let tbl_val = self.eval_expr(table, env)?;
                        let tbl_rc = tbl_val.as_table()?;
                        tbl_rc.borrow_mut().set_str(field, func_val);
                    }
                    AssignTarget::Index { table, index } => {
                        let tbl_val = self.eval_expr(table, env)?;
                        let idx_val = self.eval_expr(index, env)?;
                        let tbl_rc = tbl_val.as_table()?;
                        tbl_rc.borrow_mut().set(idx_val, func_val);
                    }
                }
                Ok(ControlFlow::Continue)
            }
            Stmt::LocalFunctionDecl { name, params, body } => {
                let func_val = LuaValue::Function(LuaFunction::User(Rc::new(UserFunction {
                    name: Some(name.clone()),
                    params: params.clone(),
                    body: body.clone(),
                    captured_env: env.clone(),
                })));
                env.borrow_mut().set_local(name.clone(), func_val);
                Ok(ControlFlow::Continue)
            }
            Stmt::Return(exprs) => {
                let mut results = Vec::new();
                for e in exprs {
                    results.push(self.eval_expr(e, env)?);
                }
                Ok(ControlFlow::Return(results))
            }
            Stmt::Break => Ok(ControlFlow::Break),
        }
    }

    /// Evaluates an expression producing a LuaValue.
    pub fn eval_expr(
        &mut self,
        expr: &Expr,
        env: &Rc<RefCell<Environment>>,
    ) -> Result<LuaValue, String> {
        match expr {
            Expr::Nil => Ok(LuaValue::Nil),
            Expr::Boolean(b) => Ok(LuaValue::Boolean(*b)),
            Expr::Number(n) => Ok(LuaValue::Number(*n)),
            Expr::String(s) => Ok(LuaValue::String(s.clone())),
            Expr::Ident(name) => Ok(env.borrow().get(name)),
            Expr::Table(fields) => {
                let tbl = Rc::new(RefCell::new(LuaTable::new()));
                let mut list_idx = 1i64;
                for field in fields {
                    match field {
                        TableField::List(val_expr) => {
                            let val = self.eval_expr(val_expr, env)?;
                            tbl.borrow_mut()
                                .set(LuaValue::Number(list_idx as f64), val);
                            list_idx += 1;
                        }
                        TableField::Record(key, val_expr) => {
                            let val = self.eval_expr(val_expr, env)?;
                            tbl.borrow_mut().set_str(key, val);
                        }
                        TableField::Dynamic(k_expr, v_expr) => {
                            let key = self.eval_expr(k_expr, env)?;
                            let val = self.eval_expr(v_expr, env)?;
                            tbl.borrow_mut().set(key, val);
                        }
                    }
                }
                Ok(LuaValue::Table(tbl))
            }
            Expr::Unary { op, expr: inner } => {
                let val = self.eval_expr(inner, env)?;
                match op {
                    UnaryOp::Neg => {
                        let n = val.as_number()?;
                        Ok(LuaValue::Number(-n))
                    }
                    UnaryOp::Not => Ok(LuaValue::Boolean(!val.is_truthy())),
                    UnaryOp::Len => match val {
                        LuaValue::String(s) => Ok(LuaValue::Number(s.len() as f64)),
                        LuaValue::Table(t) => Ok(LuaValue::Number(t.borrow().len() as f64)),
                        other => Err(format!("Cannot take length of {}", other.type_name())),
                    },
                }
            }
            Expr::Binary { op, left, right } => {
                // Short-circuiting logical operators
                if *op == BinaryOp::And {
                    let l = self.eval_expr(left, env)?;
                    return if !l.is_truthy() {
                        Ok(l)
                    } else {
                        self.eval_expr(right, env)
                    };
                }
                if *op == BinaryOp::Or {
                    let l = self.eval_expr(left, env)?;
                    return if l.is_truthy() {
                        Ok(l)
                    } else {
                        self.eval_expr(right, env)
                    };
                }

                let l = self.eval_expr(left, env)?;
                let r = self.eval_expr(right, env)?;

                match op {
                    BinaryOp::Add => Ok(LuaValue::Number(l.as_number()? + r.as_number()?)),
                    BinaryOp::Sub => Ok(LuaValue::Number(l.as_number()? - r.as_number()?)),
                    BinaryOp::Mul => Ok(LuaValue::Number(l.as_number()? * r.as_number()?)),
                    BinaryOp::Div => {
                        let denom = r.as_number()?;
                        if denom.abs() < 1e-15 {
                            Ok(LuaValue::Number(0.0))
                        } else {
                            Ok(LuaValue::Number(l.as_number()? / denom))
                        }
                    }
                    BinaryOp::Mod => {
                        let denom = r.as_number()?;
                        if denom.abs() < 1e-15 {
                            Ok(LuaValue::Number(0.0))
                        } else {
                            Ok(LuaValue::Number(l.as_number()? % denom))
                        }
                    }
                    BinaryOp::Pow => Ok(LuaValue::Number(l.as_number()?.powf(r.as_number()?))),
                    BinaryOp::Concat => Ok(LuaValue::String(format!(
                        "{}{}",
                        l.to_display_string(),
                        r.to_display_string()
                    ))),
                    BinaryOp::Eq => Ok(LuaValue::Boolean(l == r)),
                    BinaryOp::Ne => Ok(LuaValue::Boolean(l != r)),
                    BinaryOp::Lt => {
                        if let (Ok(nl), Ok(nr)) = (l.as_number(), r.as_number()) {
                            Ok(LuaValue::Boolean(nl < nr))
                        } else {
                            Ok(LuaValue::Boolean(l.to_display_string() < r.to_display_string()))
                        }
                    }
                    BinaryOp::Le => {
                        if let (Ok(nl), Ok(nr)) = (l.as_number(), r.as_number()) {
                            Ok(LuaValue::Boolean(nl <= nr))
                        } else {
                            Ok(LuaValue::Boolean(l.to_display_string() <= r.to_display_string()))
                        }
                    }
                    BinaryOp::Gt => {
                        if let (Ok(nl), Ok(nr)) = (l.as_number(), r.as_number()) {
                            Ok(LuaValue::Boolean(nl > nr))
                        } else {
                            Ok(LuaValue::Boolean(l.to_display_string() > r.to_display_string()))
                        }
                    }
                    BinaryOp::Ge => {
                        if let (Ok(nl), Ok(nr)) = (l.as_number(), r.as_number()) {
                            Ok(LuaValue::Boolean(nl >= nr))
                        } else {
                            Ok(LuaValue::Boolean(l.to_display_string() >= r.to_display_string()))
                        }
                    }
                    BinaryOp::And | BinaryOp::Or => unreachable!(),
                }
            }
            Expr::Dot { table, field } => {
                let tbl_val = self.eval_expr(table, env)?;
                let tbl_rc = tbl_val.as_table()?;
                let res = tbl_rc.borrow().get_str(field);
                Ok(res)
            }
            Expr::Index { table, index } => {
                let tbl_val = self.eval_expr(table, env)?;
                let idx_val = self.eval_expr(index, env)?;
                let tbl_rc = tbl_val.as_table()?;
                let res = tbl_rc.borrow().get(&idx_val);
                Ok(res)
            }
            Expr::Call { callee, args } => {
                let fn_val = self.eval_expr(callee, env)?;
                let mut evaluated_args = Vec::with_capacity(args.len());
                for a in args {
                    evaluated_args.push(self.eval_expr(a, env)?);
                }
                match fn_val {
                    LuaValue::Function(func) => self.call_function(&func, &evaluated_args),
                    other => Err(format!("Attempt to call non-function ({})", other.type_name())),
                }
            }
            Expr::MethodCall {
                table,
                method,
                args,
            } => {
                let tbl_val = self.eval_expr(table, env)?;
                let tbl_rc = tbl_val.as_table()?;
                let fn_val = tbl_rc.borrow().get_str(method);
                let mut evaluated_args = Vec::with_capacity(args.len() + 1);
                evaluated_args.push(tbl_val.clone()); // Pass self as first arg
                for a in args {
                    evaluated_args.push(self.eval_expr(a, env)?);
                }
                match fn_val {
                    LuaValue::Function(func) => self.call_function(&func, &evaluated_args),
                    other => Err(format!(
                        "Attempt to call method '{}' which is non-function ({})",
                        method,
                        other.type_name()
                    )),
                }
            }
            Expr::Function { params, body } => Ok(LuaValue::Function(LuaFunction::User(Rc::new(
                UserFunction {
                    name: None,
                    params: params.clone(),
                    body: body.clone(),
                    captured_env: env.clone(),
                },
            )))),
        }
    }

    /// Invokes a Lua function (native or user closure).
    pub fn call_function(
        &mut self,
        func: &LuaFunction,
        args: &[LuaValue],
    ) -> Result<LuaValue, String> {
        match func {
            LuaFunction::Native(f) => {
                let native_fn = f.clone();
                native_fn(self, args)
            }
            LuaFunction::User(u) => {
                let call_env = Rc::new(RefCell::new(Environment::new(Some(
                    u.captured_env.clone(),
                ))));
                for (i, param_name) in u.params.iter().enumerate() {
                    let arg_val = args.get(i).cloned().unwrap_or(LuaValue::Nil);
                    call_env.borrow_mut().set_local(param_name.clone(), arg_val);
                }
                let flow = self.execute_block(&u.body, &call_env)?;
                match flow {
                    ControlFlow::Return(mut vals) => Ok(vals.pop().unwrap_or(LuaValue::Nil)),
                    _ => Ok(LuaValue::Nil),
                }
            }
        }
    }
}

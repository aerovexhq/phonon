#![deny(unsafe_code)]

//! Scripting subsystem for Phonon Studio embedded testbenches and expression graphing.

pub mod expression_grapher;
pub mod lua_engine;
pub mod permissions;

pub use expression_grapher::{
    evaluate_expression, generate_trace, parse_expression, ExpressionGrapher, MathAst, MathOp,
};
pub use lua_engine::{AssertionRecord, LuaEngine, LuaFunction, LuaTable, LuaValue, TableKey};
pub use permissions::{PermissionKind, PermissionManager, PermissionState};

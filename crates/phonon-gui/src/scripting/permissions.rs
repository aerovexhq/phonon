#![deny(unsafe_code)]

//! Interactive Runtime Permission Security System for Phonon Studio Lua Scripts.
//!
//! Provides granular sandbox controls for external file operations, ensuring
//! testbench scripts cannot access filesystem resources without explicit user approval.

use std::collections::HashMap;

/// Granular permission access type requested by a Lua script.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PermissionKind {
    /// Read access to local file path.
    FileRead(String),
    /// Write access to local file path.
    FileWrite(String),
}

impl PermissionKind {
    /// Returns target path for the permission.
    pub fn path(&self) -> &str {
        match self {
            Self::FileRead(p) | Self::FileWrite(p) => p.as_str(),
        }
    }

    /// Returns human-readable operation action name.
    pub fn action_name(&self) -> &'static str {
        match self {
            Self::FileRead(_) => "Read File",
            Self::FileWrite(_) => "Write File",
        }
    }

    /// Serializes permission kind to string key.
    pub fn to_key_string(&self) -> String {
        match self {
            Self::FileRead(p) => format!("read:{}", p),
            Self::FileWrite(p) => format!("write:{}", p),
        }
    }

    /// Deserializes permission kind from string key.
    pub fn from_key_string(s: &str) -> Option<Self> {
        if let Some(path) = s.strip_prefix("read:") {
            Some(Self::FileRead(path.to_string()))
        } else if let Some(path) = s.strip_prefix("write:") {
            Some(Self::FileWrite(path.to_string()))
        } else {
            None
        }
    }
}

/// Authorization state of a requested permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    /// Permanently allowed across application sessions.
    AllowedAlways,
    /// Temporarily allowed for current testbench session only.
    AllowedSession,
    /// Explicitly denied by user.
    Denied,
    /// Pending user decision prompt.
    PendingPrompt,
}

impl PermissionState {
    /// Returns whether this state permits execution.
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::AllowedAlways | Self::AllowedSession)
    }

    /// Converts state to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AllowedAlways => "AllowedAlways",
            Self::AllowedSession => "AllowedSession",
            Self::Denied => "Denied",
            Self::PendingPrompt => "PendingPrompt",
        }
    }

    /// Parses state from string.
    pub fn from_str_val(s: &str) -> Option<Self> {
        match s {
            "AllowedAlways" => Some(Self::AllowedAlways),
            "AllowedSession" => Some(Self::AllowedSession),
            "Denied" => Some(Self::Denied),
            "PendingPrompt" => Some(Self::PendingPrompt),
            _ => None,
        }
    }
}

/// Interactive Runtime Permission Manager for Lua scripting environment.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PermissionManager {
    /// Persistent permissions across application restarts.
    pub permanent_rules: HashMap<PermissionKind, PermissionState>,
    /// Ephemeral permissions for the active session.
    pub session_rules: HashMap<PermissionKind, PermissionState>,
    /// Active pending request awaiting user confirmation.
    pub pending_request: Option<PermissionKind>,
    /// Whether untrusted scripts default to prompting.
    pub prompt_on_unknown: bool,
}

impl PermissionManager {
    /// Creates a new PermissionManager with default secure sandbox settings.
    pub fn new() -> Self {
        Self {
            permanent_rules: HashMap::new(),
            session_rules: HashMap::new(),
            pending_request: None,
            prompt_on_unknown: true,
        }
    }

    /// Evaluates current authorization status for a permission kind.
    pub fn query_permission(&self, kind: &PermissionKind) -> PermissionState {
        if let Some(&state) = self.session_rules.get(kind) {
            return state;
        }
        if let Some(&state) = self.permanent_rules.get(kind) {
            return state;
        }
        PermissionState::PendingPrompt
    }

    /// Checks if a requested operation is permitted.
    ///
    /// If permitted, returns Ok(()).
    /// If denied, returns Err("Permission denied: ...").
    /// If pending prompt, records pending request and returns Err("Permission pending: ...").
    pub fn request_permission(&mut self, kind: PermissionKind) -> Result<(), String> {
        match self.query_permission(&kind) {
            PermissionState::AllowedAlways | PermissionState::AllowedSession => Ok(()),
            PermissionState::Denied => {
                Err(format!("Permission denied: {} on '{}'", kind.action_name(), kind.path()))
            }
            PermissionState::PendingPrompt => {
                self.pending_request = Some(kind.clone());
                Err(format!(
                    "Permission pending user approval: {} on '{}'",
                    kind.action_name(),
                    kind.path()
                ))
            }
        }
    }

    /// Grants permission for the current session only.
    pub fn allow_session(&mut self, kind: PermissionKind) {
        if self.pending_request.as_ref() == Some(&kind) {
            self.pending_request = None;
        }
        self.session_rules.insert(kind, PermissionState::AllowedSession);
    }

    /// Grants permanent permission persisted across sessions.
    pub fn allow_permanent(&mut self, kind: PermissionKind) {
        if self.pending_request.as_ref() == Some(&kind) {
            self.pending_request = None;
        }
        self.permanent_rules.insert(kind, PermissionState::AllowedAlways);
    }

    /// Denies permission either permanently or for the current session.
    pub fn deny(&mut self, kind: PermissionKind, permanent: bool) {
        if self.pending_request.as_ref() == Some(&kind) {
            self.pending_request = None;
        }
        if permanent {
            self.permanent_rules.insert(kind, PermissionState::Denied);
        } else {
            self.session_rules.insert(kind, PermissionState::Denied);
        }
    }

    /// Clears any pending permission request.
    pub fn clear_pending(&mut self) {
        self.pending_request = None;
    }

    /// Clears all session rules.
    pub fn clear_session_rules(&mut self) {
        self.session_rules.clear();
    }

    /// Clears all permanent rules.
    pub fn clear_permanent_rules(&mut self) {
        self.permanent_rules.clear();
    }

    /// Serializes permanent rules for preferences persistence.
    pub fn export_settings(&self) -> String {
        let mut lines = Vec::new();
        for (kind, state) in &self.permanent_rules {
            lines.push(format!("{}={}", kind.to_key_string(), state.as_str()));
        }
        lines.sort();
        lines.join("\n")
    }

    /// Deserializes permanent rules from settings string.
    pub fn import_settings(&mut self, content: &str) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                if let (Some(kind), Some(state)) = (
                    PermissionKind::from_key_string(key),
                    PermissionState::from_str_val(val),
                ) {
                    self.permanent_rules.insert(kind, state);
                }
            }
        }
    }
}

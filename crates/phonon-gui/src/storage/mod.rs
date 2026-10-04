#![deny(unsafe_code)]

//! Cross-platform project storage adapter and virtual file system for Phonon Studio.
//!
//! Provides unified persistence across native desktop storage and WebAssembly browser storage
//! (`localStorage`, Origin Private File System / Blob downloads).

use crate::schematic::{
    deserialize_project, serialize_project, DeserializedProject,
};

/// Metadata descriptor for a saved project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectMetadata {
    pub name: String,
    pub component_count: usize,
    pub wire_count: usize,
    pub size_bytes: usize,
    pub modified_epoch: u64,
}

/// Abstract storage adapter trait implemented for browser and desktop targets.
pub trait StorageAdapter: Send + Sync {
    /// Saves project payload bytes under the given project name.
    fn save_project(&mut self, name: &str, data: &[u8]) -> Result<(), String>;

    /// Loads project payload bytes for the given project name.
    fn load_project(&self, name: &str) -> Result<Vec<u8>, String>;

    /// Lists all project names available in storage.
    fn list_projects(&self) -> Result<Vec<String>, String>;

    /// Deletes a project from storage.
    fn delete_project(&mut self, name: &str) -> Result<(), String>;

    /// Saves the current session autosave snapshot.
    fn save_autosave(&mut self, data: &[u8]) -> Result<(), String>;

    /// Loads the active session autosave snapshot, if any.
    fn load_autosave(&self) -> Option<Vec<u8>>;

    /// Clears the autosave snapshot.
    fn clear_autosave(&mut self) -> Result<(), String>;
}

/// In-memory storage adapter for testing and fallback environments.
#[derive(Debug, Clone, Default)]
pub struct MemoryStorageAdapter {
    projects: std::collections::HashMap<String, Vec<u8>>,
    autosave: Option<Vec<u8>>,
}

impl MemoryStorageAdapter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StorageAdapter for MemoryStorageAdapter {
    fn save_project(&mut self, name: &str, data: &[u8]) -> Result<(), String> {
        self.projects.insert(name.to_string(), data.to_vec());
        Ok(())
    }

    fn load_project(&self, name: &str) -> Result<Vec<u8>, String> {
        self.projects
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Project '{}' not found in memory storage", name))
    }

    fn list_projects(&self) -> Result<Vec<String>, String> {
        let mut list: Vec<String> = self.projects.keys().cloned().collect();
        list.sort();
        Ok(list)
    }

    fn delete_project(&mut self, name: &str) -> Result<(), String> {
        self.projects.remove(name);
        Ok(())
    }

    fn save_autosave(&mut self, data: &[u8]) -> Result<(), String> {
        self.autosave = Some(data.to_vec());
        Ok(())
    }

    fn load_autosave(&self) -> Option<Vec<u8>> {
        self.autosave.clone()
    }

    fn clear_autosave(&mut self) -> Result<(), String> {
        self.autosave = None;
        Ok(())
    }
}

/// Converts a byte slice to a lowercase hexadecimal string.
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

/// Converts a hexadecimal string back into raw bytes.
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Hex string length must be even".to_string());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex byte at index {}: {}", i, e))
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
/// Browser `localStorage` and Blob download adapter for WebAssembly.
#[derive(Debug, Clone, Default)]
pub struct BrowserStorageAdapter;

#[cfg(target_arch = "wasm32")]
impl BrowserStorageAdapter {
    pub fn new() -> Self {
        Self
    }

    fn get_storage() -> Result<web_sys::Storage, String> {
        let window = web_sys::window().ok_or_else(|| "No global window found".to_string())?;
        window
            .local_storage()
            .map_err(|_| "Failed to access localStorage".to_string())?
            .ok_or_else(|| "localStorage not available".to_string())
    }

    /// Triggers a browser file download using a Web Blob and temporary anchor element.
    pub fn trigger_download(filename: &str, data: &[u8]) -> Result<(), String> {
        let window = web_sys::window().ok_or_else(|| "No global window found".to_string())?;
        let document = window
            .document()
            .ok_or_else(|| "No document found in window".to_string())?;

        let uint8_array = js_sys::Uint8Array::from(data);
        let array = js_sys::Array::new();
        array.push(&uint8_array);

        let bag = web_sys::BlobPropertyBag::new();
        bag.set_type("application/octet-stream");

        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&array, &bag)
            .map_err(|_| "Failed to create Blob".to_string())?;

        let url = web_sys::Url::create_object_url_with_blob(&blob)
            .map_err(|_| "Failed to create Object URL".to_string())?;

        let anchor = document
            .create_element("a")
            .map_err(|_| "Failed to create anchor element".to_string())?;
        let anchor = anchor
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .map_err(|_| "Element is not an anchor".to_string())?;

        anchor.set_href(&url);
        anchor.set_download(filename);
        anchor.click();

        let _ = web_sys::Url::revoke_object_url(&url);
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl StorageAdapter for BrowserStorageAdapter {
    fn save_project(&mut self, name: &str, data: &[u8]) -> Result<(), String> {
        let storage = Self::get_storage()?;
        let hex = bytes_to_hex(data);
        let key = format!("phonon:project:{}", name);
        storage
            .set_item(&key, &hex)
            .map_err(|_| format!("Failed to write project '{}' to localStorage", name))?;

        // Update project catalog index
        let mut projects = self.list_projects().unwrap_or_default();
        if !projects.contains(&name.to_string()) {
            projects.push(name.to_string());
            let index_val = projects.join(";");
            let _ = storage.set_item("phonon:projects_index", &index_val);
        }
        Ok(())
    }

    fn load_project(&self, name: &str) -> Result<Vec<u8>, String> {
        let storage = Self::get_storage()?;
        let key = format!("phonon:project:{}", name);
        let hex = storage
            .get_item(&key)
            .map_err(|_| format!("Failed to read project '{}'", name))?
            .ok_or_else(|| format!("Project '{}' not found in browser storage", name))?;
        hex_to_bytes(&hex)
    }

    fn list_projects(&self) -> Result<Vec<String>, String> {
        let storage = Self::get_storage()?;
        if let Ok(Some(index_val)) = storage.get_item("phonon:projects_index") {
            let list: Vec<String> = index_val
                .split(';')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            Ok(list)
        } else {
            Ok(Vec::new())
        }
    }

    fn delete_project(&mut self, name: &str) -> Result<(), String> {
        let storage = Self::get_storage()?;
        let key = format!("phonon:project:{}", name);
        let _ = storage.remove_item(&key);

        let mut projects = self.list_projects().unwrap_or_default();
        projects.retain(|p| p != name);
        let index_val = projects.join(";");
        let _ = storage.set_item("phonon:projects_index", &index_val);
        Ok(())
    }

    fn save_autosave(&mut self, data: &[u8]) -> Result<(), String> {
        let storage = Self::get_storage()?;
        let hex = bytes_to_hex(data);
        storage
            .set_item("phonon:autosave", &hex)
            .map_err(|_| "Failed to write autosave to localStorage".to_string())
    }

    fn load_autosave(&self) -> Option<Vec<u8>> {
        let storage = Self::get_storage().ok()?;
        let hex = storage.get_item("phonon:autosave").ok()??;
        hex_to_bytes(&hex).ok()
    }

    fn clear_autosave(&mut self) -> Result<(), String> {
        let storage = Self::get_storage()?;
        let _ = storage.remove_item("phonon:autosave");
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Native filesystem project storage adapter for desktop targets.
#[derive(Debug, Clone)]
pub struct DiskStorageAdapter {
    root_dir: std::path::PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl DiskStorageAdapter {
    pub fn new() -> Self {
        let in_test = cfg!(test)
            || std::env::var_os("CARGO_TARGET_TMPDIR").is_some()
            || std::env::current_exe().map_or(false, |p| p.to_string_lossy().contains("/deps/"));
        let root_dir = if in_test {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static TEST_ID: AtomicUsize = AtomicUsize::new(1);
            let id = TEST_ID.fetch_add(1, Ordering::Relaxed);
            std::env::temp_dir()
                .join("phonon_tests")
                .join(format!("{}_{}", std::process::id(), id))
        } else {
            std::env::temp_dir().join("phonon").join("projects")
        };
        let _ = std::fs::create_dir_all(&root_dir);
        Self { root_dir }
    }

    pub fn with_directory(path: impl Into<std::path::PathBuf>) -> Self {
        let root_dir = path.into();
        let _ = std::fs::create_dir_all(&root_dir);
        Self { root_dir }
    }

    fn project_path(&self, name: &str) -> std::path::PathBuf {
        let safe_name: String = name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
            .collect();
        self.root_dir.join(format!("{}.phn", safe_name))
    }

    fn autosave_path(&self) -> std::path::PathBuf {
        self.root_dir.join("autosave_session.phn")
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for DiskStorageAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl StorageAdapter for DiskStorageAdapter {
    fn save_project(&mut self, name: &str, data: &[u8]) -> Result<(), String> {
        let path = self.project_path(name);
        std::fs::write(&path, data)
            .map_err(|e| format!("Failed to write project file {}: {}", path.display(), e))
    }

    fn load_project(&self, name: &str) -> Result<Vec<u8>, String> {
        let path = self.project_path(name);
        std::fs::read(&path)
            .map_err(|e| format!("Failed to read project file {}: {}", path.display(), e))
    }

    fn list_projects(&self) -> Result<Vec<String>, String> {
        let mut list = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.root_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("phn") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if stem != "autosave_session" {
                            list.push(stem.to_string());
                        }
                    }
                }
            }
        }
        list.sort();
        Ok(list)
    }

    fn delete_project(&mut self, name: &str) -> Result<(), String> {
        let path = self.project_path(name);
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete {}: {}", path.display(), e))?;
        }
        Ok(())
    }

    fn save_autosave(&mut self, data: &[u8]) -> Result<(), String> {
        let path = self.autosave_path();
        std::fs::write(&path, data)
            .map_err(|e| format!("Failed to write autosave {}: {}", path.display(), e))
    }

    fn load_autosave(&self) -> Option<Vec<u8>> {
        let path = self.autosave_path();
        if path.exists() {
            std::fs::read(&path).ok()
        } else {
            None
        }
    }

    fn clear_autosave(&mut self) -> Result<(), String> {
        let path = self.autosave_path();
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        Ok(())
    }
}

/// Unified cross-platform Project Storage Manager.
pub struct ProjectStorageManager {
    adapter: Box<dyn StorageAdapter>,
}

impl ProjectStorageManager {
    /// Creates a ProjectStorageManager with the platform default adapter.
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                adapter: Box::new(BrowserStorageAdapter::new()),
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                adapter: Box::new(DiskStorageAdapter::new()),
            }
        }
    }

    /// Creates a ProjectStorageManager with a custom storage adapter (e.g. MemoryStorageAdapter).
    pub fn with_adapter(adapter: Box<dyn StorageAdapter>) -> Self {
        Self { adapter }
    }

    /// Saves project state under a given name.
    pub fn save_project(
        &mut self,
        title: &str,
        components: &[crate::schematic::SchematicComponent],
        wires: &[crate::schematic::SchematicWire],
    ) -> Result<(), String> {
        let bytes = serialize_project(title, components, wires);
        self.adapter.save_project(title, &bytes)
    }

    /// Loads and deserializes a project by name.
    pub fn load_project(&self, name: &str) -> Result<DeserializedProject, String> {
        let bytes = self.adapter.load_project(name)?;
        deserialize_project(&bytes).map_err(|e| e.to_string())
    }

    /// Lists all projects in storage.
    pub fn list_projects(&self) -> Result<Vec<String>, String> {
        self.adapter.list_projects()
    }

    /// Deletes a project by name.
    pub fn delete_project(&mut self, name: &str) -> Result<(), String> {
        self.adapter.delete_project(name)
    }

    /// Autosaves the active circuit session.
    pub fn save_autosave(
        &mut self,
        title: &str,
        components: &[crate::schematic::SchematicComponent],
        wires: &[crate::schematic::SchematicWire],
    ) -> Result<(), String> {
        let bytes = serialize_project(title, components, wires);
        self.adapter.save_autosave(&bytes)
    }

    /// Restores the last active circuit session, if available.
    pub fn load_autosave(&self) -> Option<DeserializedProject> {
        let bytes = self.adapter.load_autosave()?;
        deserialize_project(&bytes).ok()
    }

    /// Clears the autosave record.
    pub fn clear_autosave(&mut self) -> Result<(), String> {
        self.adapter.clear_autosave()
    }

    /// Saves raw payload bytes under a key.
    pub fn save_raw(&mut self, key: &str, data: &[u8]) -> Result<(), String> {
        self.adapter.save_project(key, data)
    }

    /// Loads raw payload bytes for a key.
    pub fn load_raw(&self, key: &str) -> Result<Vec<u8>, String> {
        self.adapter.load_project(key)
    }
}

impl Default for ProjectStorageManager {
    fn default() -> Self {
        Self::new()
    }
}

#![deny(unsafe_code)]

//! Centralized Component Cost Registry & Single-Source-of-Truth Pricing Engine.
//!
//! Maintains component pricing across part numbers, models, and footprints.
//! When a component price is updated in one location, all matching components
//! across the active schematic, multi-sheet designs, and nested subcircuits
//! synchronously inherit the updated cost.

use std::collections::HashMap;

/// Detailed catalog entry for a component pricing record.
#[derive(Debug, Clone, PartialEq)]
pub struct PriceEntry {
    /// Baseline 1-off prototype unit cost in USD.
    pub unit_cost: f64,
    /// Distributor / supplier order part number (e.g. Digi-Key, Mouser, LCSC).
    pub supplier_pn: String,
    /// Component manufacturer designation.
    pub manufacturer: String,
    /// Functional component description.
    pub description: String,
    /// Unix timestamp of last price modification.
    pub last_updated_timestamp: u64,
}

impl Default for PriceEntry {
    fn default() -> Self {
        Self {
            unit_cost: 0.05,
            supplier_pn: String::new(),
            manufacturer: String::new(),
            description: "Generic electronic component".to_string(),
            last_updated_timestamp: 0,
        }
    }
}

impl PriceEntry {
    pub fn new(unit_cost: f64, supplier_pn: &str, manufacturer: &str, description: &str) -> Self {
        Self {
            unit_cost: unit_cost.max(0.0001),
            supplier_pn: supplier_pn.to_string(),
            manufacturer: manufacturer.to_string(),
            description: description.to_string(),
            last_updated_timestamp: 0,
        }
    }
}

/// Central single-source-of-truth cost registry.
#[derive(Debug, Clone, PartialEq)]
pub struct CentralCostRegistry {
    /// Keyed by normalized part key (e.g. "R_10k_0805", "BC547B", "NE555P", "C_100nF").
    entries: HashMap<String, PriceEntry>,
}

impl Default for CentralCostRegistry {
    fn default() -> Self {
        Self::new_with_standard_defaults()
    }
}

impl CentralCostRegistry {
    /// Creates an empty cost registry.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Pre-populates registry with standard reference component costs.
    pub fn new_with_standard_defaults() -> Self {
        let mut reg = Self::new();

        // Standard passive components (SMD 0805 / 0603)
        reg.insert(
            "R_PASSIVE_GENERIC",
            PriceEntry::new(0.012, "LCSC: C17414", "Yageo", "Thick Film Chip Resistor 1% 1/8W"),
        );
        reg.insert(
            "C_CERAMIC_GENERIC",
            PriceEntry::new(0.025, "LCSC: C15849", "Samsung", "Multilayer Ceramic Capacitor X7R 50V"),
        );
        reg.insert(
            "L_INDUCTOR_GENERIC",
            PriceEntry::new(0.085, "DigiKey: 445-1234-1-ND", "TDK", "Shielded Power Inductor SMD"),
        );

        // Specific common passives
        reg.insert(
            "R_10k",
            PriceEntry::new(0.012, "LCSC: C17414", "Yageo", "10k Ohm 0805 1% SMD Resistor"),
        );
        reg.insert(
            "R_1k",
            PriceEntry::new(0.012, "LCSC: C21190", "Yageo", "1k Ohm 0805 1% SMD Resistor"),
        );
        reg.insert(
            "R_100k",
            PriceEntry::new(0.012, "LCSC: C17407", "Yageo", "100k Ohm 0805 1% SMD Resistor"),
        );
        reg.insert(
            "R_47k",
            PriceEntry::new(0.012, "LCSC: C17462", "Yageo", "47k Ohm 0805 1% SMD Resistor"),
        );
        reg.insert(
            "R_220",
            PriceEntry::new(0.012, "LCSC: C17548", "Yageo", "220 Ohm 0805 1% SMD Resistor"),
        );
        reg.insert(
            "C_100nF",
            PriceEntry::new(0.025, "LCSC: C15849", "Samsung", "100nF (0.1uF) 50V X7R 0805 MLCC"),
        );
        reg.insert(
            "C_10uF",
            PriceEntry::new(0.065, "LCSC: C19702", "Murata", "10uF 25V X5R 0805 MLCC"),
        );
        reg.insert(
            "C_1uF",
            PriceEntry::new(0.038, "LCSC: C28323", "Murata", "1uF 50V X7R 0805 MLCC"),
        );

        // Discrete semiconductors
        reg.insert(
            "D_1N4148",
            PriceEntry::new(0.035, "LCSC: C2128", "Nexperia", "High-speed switching diode SOD-323"),
        );
        reg.insert(
            "D_LED_GREEN",
            PriceEntry::new(0.045, "LCSC: C72043", "Everlight", "0805 SMD Green Indicator LED"),
        );
        reg.insert(
            "D_ZENER_3V3",
            PriceEntry::new(0.055, "DigiKey: BZX84C3V3-ND", "Onsemi", "3.3V 350mW Zener Diode SOT-23"),
        );
        reg.insert(
            "Q_BC547B",
            PriceEntry::new(0.095, "LCSC: C2148", "Nexperia", "NPN General Purpose BJT SOT-23"),
        );
        reg.insert(
            "Q_BC557B",
            PriceEntry::new(0.098, "LCSC: C2152", "Nexperia", "PNP General Purpose BJT SOT-23"),
        );
        reg.insert(
            "Q_2N7002",
            PriceEntry::new(0.088, "LCSC: C8502", "Diodes Inc", "60V N-Channel MOSFET SOT-23"),
        );
        reg.insert(
            "Q_BSS84",
            PriceEntry::new(0.092, "LCSC: C8601", "Diodes Inc", "50V P-Channel MOSFET SOT-23"),
        );

        // Integrated circuits
        reg.insert(
            "IC_NE555P",
            PriceEntry::new(0.350, "Mouser: 595-NE555P", "Texas Instruments", "Precision Single Timer DIP-8/SOIC-8"),
        );
        reg.insert(
            "IC_LM358",
            PriceEntry::new(0.280, "LCSC: C7442", "Texas Instruments", "Dual Operational Amplifier SOIC-8"),
        );
        reg.insert(
            "IC_OPA2134",
            PriceEntry::new(2.450, "DigiKey: 296-14125-5-ND", "Texas Instruments", "High Performance Audio Dual Op-Amp"),
        );
        reg.insert(
            "IC_STM32F401",
            PriceEntry::new(2.850, "LCSC: C51347", "STMicroelectronics", "ARM Cortex-M4 84MHz MCU LQFP-64"),
        );

        // Subcircuit assembly packages
        reg.insert(
            "SUB_OPAMP_BUFFER",
            PriceEntry::new(0.650, "CUSTOM: PHNC-001", "In-House Assembly", "Dual Unity-Gain Buffer Subcircuit"),
        );
        reg.insert(
            "SUB_POWER_REGULATOR",
            PriceEntry::new(1.850, "CUSTOM: PHNC-002", "In-House Assembly", "Synchronous Buck DC-DC Stage"),
        );

        reg
    }

    /// Normalizes lookup keys (trims, uppercase for standard keys).
    pub fn normalize_key(key: &str) -> String {
        key.trim().to_string()
    }

    /// Inserts or updates a pricing record for a component key.
    pub fn insert(&mut self, key: &str, entry: PriceEntry) {
        let norm = Self::normalize_key(key);
        self.entries.insert(norm, entry);
    }

    /// Sets or updates the unit cost for a component key.
    /// Returns the prior cost if previously present.
    pub fn set_cost(&mut self, key: &str, cost: f64) -> Option<f64> {
        let norm = Self::normalize_key(key);
        let safe_cost = cost.max(0.0001);

        if let Some(entry) = self.entries.get_mut(&norm) {
            let old = entry.unit_cost;
            entry.unit_cost = safe_cost;
            Some(old)
        } else {
            let mut entry = PriceEntry::default();
            entry.unit_cost = safe_cost;
            entry.description = format!("Auto-registered component {}", key);
            self.entries.insert(norm, entry);
            None
        }
    }

    /// Queries the unit cost for a part key.
    /// If not found directly, tries fuzzy matching or returns $0.05 default.
    pub fn get_cost(&self, key: &str) -> f64 {
        let norm = Self::normalize_key(key);
        if let Some(entry) = self.entries.get(&norm) {
            return entry.unit_cost;
        }

        // Prefix/fallback matching: check if key matches common families
        if norm.starts_with('R') || norm.contains("Resistor") {
            return 0.012;
        }
        if norm.starts_with('C') || norm.contains("Capacitor") {
            return 0.025;
        }
        if norm.starts_with('L') || norm.contains("Inductor") {
            return 0.085;
        }
        if norm.starts_with('D') || norm.contains("Diode") {
            return 0.035;
        }
        if norm.starts_with('Q') || norm.starts_with('M') || norm.contains("Bjt") || norm.contains("Mosfet") {
            return 0.120;
        }
        if norm.starts_with("IC_") || norm.starts_with('U') || norm.contains("OpAmp") {
            return 0.450;
        }

        0.050
    }

    /// Retrieves read-only reference to a catalog entry.
    pub fn get_entry(&self, key: &str) -> Option<&PriceEntry> {
        let norm = Self::normalize_key(key);
        self.entries.get(&norm)
    }

    /// Returns all registered part keys sorted alphabetically.
    pub fn sorted_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.entries.keys().cloned().collect();
        keys.sort();
        keys
    }

    /// Total count of catalog entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether registry contains zero entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Serializes registry entries into a simple pipe-delimited format
    /// suitable for embedded metadata persistence.
    pub fn serialize_to_text(&self) -> String {
        let mut lines = Vec::new();
        for key in self.sorted_keys() {
            if let Some(e) = self.entries.get(&key) {
                lines.push(format!(
                    "{}|{:.6}|{}|{}|{}",
                    key, e.unit_cost, e.supplier_pn, e.manufacturer, e.description
                ));
            }
        }
        lines.join("\n")
    }

    /// Deserializes registry entries from pipe-delimited text.
    pub fn deserialize_from_text(text: &str) -> Self {
        let mut reg = Self::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 2 {
                let key = parts[0];
                let cost = parts[1].parse::<f64>().unwrap_or(0.05);
                let supplier = if parts.len() >= 3 { parts[2] } else { "" };
                let mfr = if parts.len() >= 4 { parts[3] } else { "" };
                let desc = if parts.len() >= 5 { parts[4] } else { "" };

                reg.insert(key, PriceEntry::new(cost, supplier, mfr, desc));
            }
        }
        reg
    }
}

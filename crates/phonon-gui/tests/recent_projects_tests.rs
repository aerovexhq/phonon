#![deny(unsafe_code)]

//! Test suite for the recent projects management and persistence system.
//!
//! Validates:
//! 1. Default `recent_projects` initialization in `AppPreferences`.
//! 2. Most-recently-used (MRU) ordering when adding projects.
//! 3. Deduplication of existing project entries.
//! 4. Maximum list capping at 10 entries.
//! 5. Clear recent projects functionality.
//! 6. Config string serialization and deserialization roundtrip.

use phonon_gui::preferences::AppPreferences;

#[test]
fn test_recent_projects_defaults_and_ordering() {
    let mut prefs = AppPreferences::default();
    assert!(prefs.recent_projects.is_empty());

    prefs.add_recent_project("Project Alpha");
    assert_eq!(prefs.recent_projects.len(), 1);
    assert_eq!(prefs.recent_projects[0], "Project Alpha");

    prefs.add_recent_project("Project Beta");
    assert_eq!(prefs.recent_projects.len(), 2);
    // Beta should be at the front (MRU order)
    assert_eq!(prefs.recent_projects[0], "Project Beta");
    assert_eq!(prefs.recent_projects[1], "Project Alpha");
}

#[test]
fn test_recent_projects_deduplication() {
    let mut prefs = AppPreferences::default();
    prefs.add_recent_project("Project 1");
    prefs.add_recent_project("Project 2");
    prefs.add_recent_project("Project 3");
    assert_eq!(prefs.recent_projects, vec!["Project 3", "Project 2", "Project 1"]);

    // Re-adding Project 1 must bump it to the front without duplicating
    prefs.add_recent_project("Project 1");
    assert_eq!(prefs.recent_projects, vec!["Project 1", "Project 3", "Project 2"]);
    assert_eq!(prefs.recent_projects.len(), 3);
}

#[test]
fn test_recent_projects_capping_at_ten() {
    let mut prefs = AppPreferences::default();
    for i in 1..=15 {
        prefs.add_recent_project(&format!("Project_{:02}", i));
    }

    assert_eq!(prefs.recent_projects.len(), 10);
    // Most recent is Project_15
    assert_eq!(prefs.recent_projects[0], "Project_15");
    // Tenth is Project_06
    assert_eq!(prefs.recent_projects[9], "Project_06");
}

#[test]
fn test_recent_projects_clear() {
    let mut prefs = AppPreferences::default();
    prefs.add_recent_project("A");
    prefs.add_recent_project("B");
    assert_eq!(prefs.recent_projects.len(), 2);

    prefs.clear_recent_projects();
    assert!(prefs.recent_projects.is_empty());
}

#[test]
fn test_recent_projects_serialization_roundtrip() {
    let mut original = AppPreferences::default();
    original.add_recent_project("Superconducting Qubit");
    original.add_recent_project("Topological Transceiver");
    original.add_recent_project("Floquet Time Crystal");

    let config_str = original.to_config_str();
    assert!(config_str.contains("recent_project: \"Floquet Time Crystal\""));
    assert!(config_str.contains("recent_project: \"Topological Transceiver\""));
    assert!(config_str.contains("recent_project: \"Superconducting Qubit\""));

    let parsed = AppPreferences::from_config_str(&config_str);
    assert_eq!(parsed.recent_projects.len(), 3);
    assert_eq!(parsed.recent_projects[0], "Floquet Time Crystal");
    assert_eq!(parsed.recent_projects[1], "Topological Transceiver");
    assert_eq!(parsed.recent_projects[2], "Superconducting Qubit");
}

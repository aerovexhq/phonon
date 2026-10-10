#![deny(unsafe_code)]

//! Unit and integration test suite for P2P CAD Collaboration & CRDT Synchronization.

use phonon_solver::crdt_collaboration::{
    audit_crdt_collaboration, CollaborationEngine, CrdtComponent, CrdtDelta, CrdtEngine,
    CrdtWire, CryptoAuthEngine, LamportTimestamp, MeshTopology, P2pMeshNetwork, PeerNode,
    PeerRole, PresenceManager, VectorClock,
};

#[test]
fn test_crdt_collaboration_10_point_audit_full_pass() {
    let report = audit_crdt_collaboration();
    assert_eq!(report.score, 10, "10-point collaboration audit must achieve 10/10 PASS");
    assert_eq!(report.total, 10);
    assert!(report.all_passed);
    assert!(report.execution_time_us < 2000, "Audit execution latency must be sub-2.0 ms");
}

#[test]
fn test_p2p_mesh_topology_and_signaling() {
    let mut net = P2pMeshNetwork::new("ROOM-123", "alice", "Alice", PeerRole::Host);
    assert_eq!(net.topology, MeshTopology::FullMesh);
    assert_eq!(net.connected_peer_count(), 0);

    let bob = PeerNode::new("bob", "Bob", PeerRole::Contributor, "webrtc-dc://bob:8443");
    net.add_peer(bob);
    assert_eq!(net.connected_peer_count(), 1);

    let offer = net.create_sdp_offer("bob");
    assert_eq!(offer.sender_id, "alice");
    assert_eq!(offer.target_id, "bob");

    net.receive_signaling_answer("bob", "v=0\r\nanswer");
    assert!(net.get_peer("bob").unwrap().state.is_active());

    net.broadcast(128);
    assert_eq!(net.total_broadcast_messages, 1);
    assert_eq!(net.total_broadcast_bytes, 128);
    assert_eq!(net.get_peer("bob").unwrap().bytes_sent, 4096 + 128);
}

#[test]
fn test_vector_clock_causality_and_concurrency() {
    let mut vc1 = VectorClock::new();
    vc1.increment("alice");
    vc1.increment("alice");
    vc1.increment("bob");

    let mut vc2 = VectorClock::new();
    vc2.increment("alice");
    vc2.increment("bob");

    assert!(vc1.dominates(&vc2));
    assert!(!vc2.dominates(&vc1));
    assert!(!vc1.is_concurrent(&vc2));

    let mut vc3 = VectorClock::new();
    vc3.increment("charlie");

    assert!(vc1.is_concurrent(&vc3));
    assert!(!vc1.dominates(&vc3));

    vc1.merge(&vc3);
    assert_eq!(vc1.get("alice"), 2);
    assert_eq!(vc1.get("bob"), 1);
    assert_eq!(vc1.get("charlie"), 1);
    assert!(vc1.dominates(&vc3));
}

#[test]
fn test_crdt_mathematical_properties_commutativity_associativity_idempotency() {
    let c_a = CrdtComponent::new("cmp-1", "R1", "Resistor", "10k", "0805", 10.0, 10.0, LamportTimestamp::new(1, "peer-a"));
    let c_b = CrdtComponent::new("cmp-1", "R1", "Resistor", "20k", "0805", 20.0, 20.0, LamportTimestamp::new(2, "peer-b"));
    let c_c = CrdtComponent::new("cmp-1", "R1", "Resistor", "30k", "0805", 30.0, 30.0, LamportTimestamp::new(3, "peer-c"));

    // 1. Commutativity: A then B vs B then A
    let mut engine_ab = CrdtEngine::new("test");
    engine_ab.apply_delta(&CrdtDelta::UpsertComponent(c_a.clone()));
    engine_ab.apply_delta(&CrdtDelta::UpsertComponent(c_b.clone()));

    let mut engine_ba = CrdtEngine::new("test");
    engine_ba.apply_delta(&CrdtDelta::UpsertComponent(c_b.clone()));
    engine_ba.apply_delta(&CrdtDelta::UpsertComponent(c_a.clone()));

    assert_eq!(engine_ab.components, engine_ba.components);
    assert_eq!(engine_ab.components.get("cmp-1").unwrap().value, "20k");

    // 2. Associativity: (A + B) + C == A + (B + C)
    let mut engine_left = CrdtEngine::new("test");
    engine_left.apply_delta(&CrdtDelta::UpsertComponent(c_a.clone()));
    engine_left.apply_delta(&CrdtDelta::UpsertComponent(c_b.clone()));
    engine_left.apply_delta(&CrdtDelta::UpsertComponent(c_c.clone()));

    let mut engine_right = CrdtEngine::new("test");
    engine_right.apply_delta(&CrdtDelta::UpsertComponent(c_b.clone()));
    engine_right.apply_delta(&CrdtDelta::UpsertComponent(c_c.clone()));
    engine_right.apply_delta(&CrdtDelta::UpsertComponent(c_a.clone()));

    assert_eq!(engine_left.components, engine_right.components);
    assert_eq!(engine_left.components.get("cmp-1").unwrap().value, "30k");

    // 3. Idempotency: duplicate delta application
    let mut engine_idem = CrdtEngine::new("test");
    engine_idem.apply_delta(&CrdtDelta::UpsertComponent(c_a.clone()));
    let snapshot = engine_idem.components.clone();
    engine_idem.apply_delta(&CrdtDelta::UpsertComponent(c_a.clone()));
    assert_eq!(engine_idem.components, snapshot);
}

#[test]
fn test_crdt_tombstone_deletion_and_zombie_prevention() {
    let mut engine = CrdtEngine::new("alice");

    let comp = CrdtComponent::new("cmp-r1", "R1", "Resistor", "10k", "0805", 50.0, 50.0, LamportTimestamp::new(1, "alice"));
    engine.apply_delta(&CrdtDelta::UpsertComponent(comp.clone()));
    assert_eq!(engine.active_component_count(), 1);

    // Delete at Lamport 5
    engine.apply_delta(&CrdtDelta::DeleteComponent {
        id: "cmp-r1".to_string(),
        lamport: LamportTimestamp::new(5, "alice"),
    });
    assert_eq!(engine.active_component_count(), 0);
    assert!(engine.components.get("cmp-r1").unwrap().tombstoned);

    // Delayed update from another peer at Lamport 2 - must NOT resurrect
    let delayed_comp = CrdtComponent::new("cmp-r1", "R1", "Resistor", "47k", "0805", 60.0, 60.0, LamportTimestamp::new(2, "bob"));
    let applied = engine.apply_delta(&CrdtDelta::UpsertComponent(delayed_comp));
    assert!(!applied);
    assert_eq!(engine.active_component_count(), 0);
    assert!(engine.components.get("cmp-r1").unwrap().tombstoned);

    // Strictly newer update at Lamport 10 - CAN update and un-tombstone
    let newer_comp = CrdtComponent::new("cmp-r1", "R1", "Resistor", "100k", "0805", 80.0, 80.0, LamportTimestamp::new(10, "bob"));
    let applied_newer = engine.apply_delta(&CrdtDelta::UpsertComponent(newer_comp));
    assert!(applied_newer);
    assert_eq!(engine.active_component_count(), 1);
    assert!(!engine.components.get("cmp-r1").unwrap().tombstoned);
    assert_eq!(engine.components.get("cmp-r1").unwrap().value, "100k");
}

#[test]
fn test_presence_manager_and_conflict_detection() {
    let mut pm = PresenceManager::new("alice", "Alice");
    pm.update_local_selection(vec!["cmp-r1".to_string()], vec!["wire-w1".to_string()]);

    let mut bob_pres = phonon_solver::crdt_collaboration::PeerPresence::new("bob", "Bob", [200, 100, 50]);
    bob_pres.selected_component_ids = vec!["cmp-c1".to_string()];
    pm.update_remote_presence(bob_pres.clone());
    assert_eq!(pm.detected_conflicts.len(), 0);

    // Bob now also selects cmp-r1 -> conflict detected!
    bob_pres.selected_component_ids.push("cmp-r1".to_string());
    pm.update_remote_presence(bob_pres);
    assert_eq!(pm.detected_conflicts.len(), 1);
    assert_eq!(pm.detected_conflicts[0].entity_id, "cmp-r1");
    assert_eq!(pm.detected_conflicts[0].entity_kind, "Component");

    let halo = pm.component_selection_halo("cmp-r1");
    assert!(halo.is_some());
}

#[test]
fn test_cryptographic_envelope_authentication_and_anti_replay() {
    let mut auth = CryptoAuthEngine::default_session("ROOM-SECURE");
    let env1 = auth.sign_delta("alice", 1_000_000, b"DELTA-1".to_vec());
    let env2 = auth.sign_delta("alice", 1_000_100, b"DELTA-2".to_vec());

    let mut receiver = CryptoAuthEngine::default_session("ROOM-SECURE");
    assert!(receiver.verify_envelope(&env1).is_ok());
    assert!(receiver.verify_envelope(&env2).is_ok());

    // Replay env1 -> must be rejected
    let replay_err = receiver.verify_envelope(&env1);
    assert!(replay_err.is_err());

    // Tamper payload in env3
    let mut env3 = auth.sign_delta("alice", 1_000_200, b"DELTA-3".to_vec());
    env3.payload_bytes[0] ^= 0x01;
    let tamper_err = receiver.verify_envelope(&env3);
    assert!(tamper_err.is_err());
}

#[test]
fn test_collaboration_engine_orchestration() {
    let mut engine = CollaborationEngine::new_fast();
    assert_eq!(engine.network.connected_peer_count(), 2);
    assert_eq!(engine.crdt.active_component_count(), 2);
    assert_eq!(engine.crdt.active_wire_count(), 1);

    // Broadcast component addition
    let env = engine.broadcast_component("cmp-q1", "Q1", "BJT_NPN", "2N3904", "TO-92", 150.0, 150.0);
    assert_eq!(engine.crdt.active_component_count(), 3);
    assert_eq!(env.sender_id, "peer-alice-01");

    // Broadcast component deletion
    engine.broadcast_component_deletion("cmp-q1");
    assert_eq!(engine.crdt.active_component_count(), 2);

    // Receive remote delta
    let remote_wire = CrdtWire::new("wire-w2", 10.0, 20.0, 30.0, 40.0, "NET_VCC", LamportTimestamp::new(100, "peer-bob-02"));
    let applied = engine.receive_remote_delta(CrdtDelta::UpsertWire(remote_wire));
    assert!(applied);
    assert_eq!(engine.crdt.active_wire_count(), 2);

    engine.run_audit();
    assert_eq!(engine.audit_report.score, 10);
}

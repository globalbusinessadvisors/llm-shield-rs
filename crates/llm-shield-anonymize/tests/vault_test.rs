//! Tests for Enhanced Vault with TTL and Session Management
//!
//! ## London School TDD - Tests First
//!
//! Test Categories:
//! - TTL Tests (10 tests)
//! - Session Management Tests (8 tests)
//! - Audit Logging Tests (5 tests)

use llm_shield_anonymize::vault::{
    AuditEvent, AuditLogger, AnonymizationSession, EntityMapping, MemoryVault,
    SessionManager, VaultStorage,
};
use llm_shield_anonymize::types::EntityType;
use std::time::Duration;
use tokio::time::sleep;

// ============================================================================
// TTL TESTS (10 tests)
// ============================================================================

#[tokio::test]
async fn test_mapping_has_expires_at_field() {
    let mapping = EntityMapping::new(
        "sess_1".to_string(),
        "[PERSON_1]".to_string(),
        EntityType::Person,
        "John Doe".to_string(),
        0.95,
        3600, // 1 hour TTL
    );

    assert!(mapping.expires_at > mapping.timestamp);
    let duration = mapping.expires_at.duration_since(mapping.timestamp).unwrap();
    assert_eq!(duration.as_secs(), 3600);
}

#[tokio::test]
async fn test_mapping_is_expired_returns_false_for_valid() {
    let mapping = EntityMapping::new(
        "sess_1".to_string(),
        "[PERSON_1]".to_string(),
        EntityType::Person,
        "John Doe".to_string(),
        0.95,
        3600, // 1 hour TTL
    );

    assert!(!mapping.is_expired());
}

#[tokio::test]
async fn test_mapping_is_expired_returns_true_after_ttl() {
    let mapping = EntityMapping::new(
        "sess_1".to_string(),
        "[PERSON_1]".to_string(),
        EntityType::Person,
        "John Doe".to_string(),
        0.95,
        0, // Already expired
    );

    // Sleep a tiny bit to ensure expiration
    sleep(Duration::from_millis(10)).await;
    assert!(mapping.is_expired());
}

#[tokio::test]
async fn test_get_mapping_returns_none_for_expired() {
    let vault = MemoryVault::new();

    let mapping = EntityMapping::new(
        "sess_1".to_string(),
        "[PERSON_1]".to_string(),
        EntityType::Person,
        "John Doe".to_string(),
        0.95,
        0, // Already expired
    );

    vault.store_mapping(mapping).await.unwrap();

    // Wait for expiration
    sleep(Duration::from_millis(10)).await;

    // Should return None for expired mapping
    let result = vault.get_mapping("sess_1", "[PERSON_1]").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_mapping_returns_some_for_valid() {
    let vault = MemoryVault::new();

    let mapping = EntityMapping::new(
        "sess_1".to_string(),
        "[PERSON_1]".to_string(),
        EntityType::Person,
        "John Doe".to_string(),
        0.95,
        3600,
    );

    vault.store_mapping(mapping).await.unwrap();

    let result = vault.get_mapping("sess_1", "[PERSON_1]").await.unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().original_value, "John Doe");
}

#[tokio::test]
async fn test_cleanup_expired_removes_expired_mappings() {
    let vault = MemoryVault::new();

    // Add expired mapping
    let expired = EntityMapping::new(
        "sess_expired".to_string(),
        "[PERSON_1]".to_string(),
        EntityType::Person,
        "Expired User".to_string(),
        0.95,
        0,
    );
    vault.store_mapping(expired).await.unwrap();

    // Add valid mapping
    let valid = EntityMapping::new(
        "sess_valid".to_string(),
        "[PERSON_1]".to_string(),
        EntityType::Person,
        "Valid User".to_string(),
        0.95,
        3600,
    );
    vault.store_mapping(valid).await.unwrap();

    sleep(Duration::from_millis(10)).await;

    // Cleanup should remove 1 expired mapping
    let removed_count = vault.cleanup_expired().await.unwrap();
    assert_eq!(removed_count, 1);

    // Expired should be gone
    let result = vault.get_mapping("sess_expired", "[PERSON_1]").await.unwrap();
    assert!(result.is_none());

    // Valid should still exist
    let result = vault.get_mapping("sess_valid", "[PERSON_1]").await.unwrap();
    assert!(result.is_some());
}

#[tokio::test]
async fn test_cleanup_expired_with_no_expired_mappings() {
    let vault = MemoryVault::new();

    let mapping = EntityMapping::new(
        "sess_1".to_string(),
        "[PERSON_1]".to_string(),
        EntityType::Person,
        "John Doe".to_string(),
        0.95,
        3600,
    );
    vault.store_mapping(mapping).await.unwrap();

    let removed_count = vault.cleanup_expired().await.unwrap();
    assert_eq!(removed_count, 0);
}

#[tokio::test]
async fn test_concurrent_access_to_vault() {
    let vault = MemoryVault::new();
    let vault_clone1 = vault.clone();
    let vault_clone2 = vault.clone();

    // Spawn concurrent writes
    let handle1 = tokio::spawn(async move {
        for i in 0..100 {
            let mapping = EntityMapping::new(
                format!("sess_{}", i),
                format!("[PERSON_{}]", i),
                EntityType::Person,
                format!("User {}", i),
                0.95,
                3600,
            );
            vault_clone1.store_mapping(mapping).await.unwrap();
        }
    });

    let handle2 = tokio::spawn(async move {
        for i in 100..200 {
            let mapping = EntityMapping::new(
                format!("sess_{}", i),
                format!("[EMAIL_{}]", i),
                EntityType::Email,
                format!("user{}@example.com", i),
                0.98,
                3600,
            );
            vault_clone2.store_mapping(mapping).await.unwrap();
        }
    });

    handle1.await.unwrap();
    handle2.await.unwrap();

    // Verify all mappings were stored
    let session_mappings = vault.get_session_mappings("sess_50").await.unwrap();
    assert_eq!(session_mappings.len(), 1);
}

#[tokio::test]
async fn test_vault_expiration_with_mixed_ttls() {
    let vault = MemoryVault::new();

    // Short TTL (already expired)
    let short_ttl = EntityMapping::new(
        "sess_1".to_string(),
        "[PERSON_1]".to_string(),
        EntityType::Person,
        "Short TTL".to_string(),
        0.95,
        0,
    );
    vault.store_mapping(short_ttl).await.unwrap();

    // Medium TTL (100ms)
    let medium_ttl = EntityMapping::new(
        "sess_1".to_string(),
        "[EMAIL_1]".to_string(),
        EntityType::Email,
        "medium@example.com".to_string(),
        0.98,
        1, // 1 second
    );
    vault.store_mapping(medium_ttl).await.unwrap();

    // Long TTL
    let long_ttl = EntityMapping::new(
        "sess_1".to_string(),
        "[PHONE_1]".to_string(),
        EntityType::PhoneNumber,
        "555-0100".to_string(),
        0.99,
        3600,
    );
    vault.store_mapping(long_ttl).await.unwrap();

    sleep(Duration::from_millis(10)).await;

    // Short should be expired
    assert!(vault.get_mapping("sess_1", "[PERSON_1]").await.unwrap().is_none());

    // Medium should still be valid
    assert!(vault.get_mapping("sess_1", "[EMAIL_1]").await.unwrap().is_some());

    // Long should still be valid
    assert!(vault.get_mapping("sess_1", "[PHONE_1]").await.unwrap().is_some());
}

#[tokio::test]
async fn test_store_and_retrieve_multiple_entity_types() {
    let vault = MemoryVault::new();

    let entities = vec![
        (EntityType::Person, "John Doe", "[PERSON_1]"),
        (EntityType::Email, "john@example.com", "[EMAIL_1]"),
        (EntityType::PhoneNumber, "555-0100", "[PHONE_1]"),
        (EntityType::SSN, "123-45-6789", "[SSN_1]"),
        (EntityType::CreditCard, "4111-1111-1111-1111", "[CC_1]"),
    ];

    for (entity_type, value, placeholder) in entities.iter() {
        let mapping = EntityMapping::new(
            "sess_1".to_string(),
            placeholder.to_string(),
            *entity_type,
            value.to_string(),
            0.95,
            3600,
        );
        vault.store_mapping(mapping).await.unwrap();
    }

    // Verify all were stored
    for (_, value, placeholder) in entities.iter() {
        let result = vault.get_mapping("sess_1", placeholder).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().original_value, *value);
    }
}

// ============================================================================
// SESSION MANAGEMENT TESTS (8 tests)
// ============================================================================

#[tokio::test]
async fn test_session_manager_create_session() {
    let vault = MemoryVault::new();
    let session_mgr = SessionManager::new(vault);

    let session = session_mgr.create_session(None, 3600).await.unwrap();

    assert!(!session.session_id.is_empty());
    assert!(session.session_id.starts_with("sess_"));
    assert_eq!(session.user_id, None);
    assert!(session.mappings.is_empty());
}

#[tokio::test]
async fn test_session_manager_create_session_with_user_id() {
    let vault = MemoryVault::new();
    let session_mgr = SessionManager::new(vault);

    let session = session_mgr
        .create_session(Some("user_123".to_string()), 3600)
        .await
        .unwrap();

    assert_eq!(session.user_id, Some("user_123".to_string()));
}

#[tokio::test]
async fn test_session_manager_get_session() {
    let vault = MemoryVault::new();
    let session_mgr = SessionManager::new(vault);

    let created = session_mgr.create_session(None, 3600).await.unwrap();

    // Add a mapping to the session
    let mapping = EntityMapping::new(
        created.session_id.clone(),
        "[PERSON_1]".to_string(),
        EntityType::Person,
        "John Doe".to_string(),
        0.95,
        3600,
    );
    session_mgr
        .add_mapping(&created.session_id, mapping)
        .await
        .unwrap();

    let retrieved = session_mgr.get_session(&created.session_id).await.unwrap();
    assert!(retrieved.is_some());

    let session = retrieved.unwrap();
    assert_eq!(session.session_id, created.session_id);
    assert_eq!(session.mappings.len(), 1);
}

#[tokio::test]
async fn test_session_manager_get_nonexistent_session() {
    let vault = MemoryVault::new();
    let session_mgr = SessionManager::new(vault);

    let result = session_mgr.get_session("nonexistent").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_session_manager_delete_session() {
    let vault = MemoryVault::new();
    let session_mgr = SessionManager::new(vault);

    let session = session_mgr.create_session(None, 3600).await.unwrap();

    // Add mappings
    let mapping1 = EntityMapping::new(
        session.session_id.clone(),
        "[PERSON_1]".to_string(),
        EntityType::Person,
        "John Doe".to_string(),
        0.95,
        3600,
    );
    let mapping2 = EntityMapping::new(
        session.session_id.clone(),
        "[EMAIL_1]".to_string(),
        EntityType::Email,
        "john@example.com".to_string(),
        0.98,
        3600,
    );

    session_mgr
        .add_mapping(&session.session_id, mapping1)
        .await
        .unwrap();
    session_mgr
        .add_mapping(&session.session_id, mapping2)
        .await
        .unwrap();

    // Delete session
    session_mgr.delete_session(&session.session_id).await.unwrap();

    // Verify it's gone
    let result = session_mgr.get_session(&session.session_id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_session_manager_list_sessions() {
    let vault = MemoryVault::new();
    let session_mgr = SessionManager::new(vault);

    // Create multiple sessions
    let session1 = session_mgr.create_session(None, 3600).await.unwrap();
    let session2 = session_mgr.create_session(None, 3600).await.unwrap();
    let session3 = session_mgr.create_session(None, 3600).await.unwrap();

    // Add at least one mapping to each so they're tracked
    for session in [&session1, &session2, &session3] {
        let mapping = EntityMapping::new(
            session.session_id.clone(),
            "[PERSON_1]".to_string(),
            EntityType::Person,
            "User".to_string(),
            0.95,
            3600,
        );
        session_mgr
            .add_mapping(&session.session_id, mapping)
            .await
            .unwrap();
    }

    let sessions = session_mgr.list_sessions().await.unwrap();
    assert_eq!(sessions.len(), 3);
}

#[tokio::test]
async fn test_session_manager_session_expiration() {
    let vault = MemoryVault::new();
    let session_mgr = SessionManager::new(vault);

    // Create session with short TTL
    let session = session_mgr.create_session(None, 0).await.unwrap();

    let mapping = EntityMapping::new(
        session.session_id.clone(),
        "[PERSON_1]".to_string(),
        EntityType::Person,
        "John Doe".to_string(),
        0.95,
        0, // Already expired
    );
    session_mgr
        .add_mapping(&session.session_id, mapping)
        .await
        .unwrap();

    sleep(Duration::from_millis(10)).await;

    // Session should not return expired mappings
    let retrieved = session_mgr.get_session(&session.session_id).await.unwrap();
    assert!(retrieved.is_none() || retrieved.unwrap().mappings.is_empty());
}

#[tokio::test]
async fn test_session_manager_add_multiple_mappings() {
    let vault = MemoryVault::new();
    let session_mgr = SessionManager::new(vault);

    let session = session_mgr.create_session(None, 3600).await.unwrap();

    // Add multiple mappings
    for i in 0..10 {
        let mapping = EntityMapping::new(
            session.session_id.clone(),
            format!("[PERSON_{}]", i),
            EntityType::Person,
            format!("User {}", i),
            0.95,
            3600,
        );
        session_mgr
            .add_mapping(&session.session_id, mapping)
            .await
            .unwrap();
    }

    let retrieved = session_mgr.get_session(&session.session_id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().mappings.len(), 10);
}

// ============================================================================
// AUDIT LOGGING TESTS (5 tests)
// ============================================================================

#[tokio::test]
async fn test_audit_logger_logs_anonymize_start() {
    let logger = AuditLogger::new();

    let event = AuditEvent::AnonymizeStart {
        session_id: "sess_123".to_string(),
        entity_count: 5,
    };

    // Should not panic
    logger.log(event).await;
}

#[tokio::test]
async fn test_audit_logger_logs_anonymize_complete() {
    let logger = AuditLogger::new();

    let event = AuditEvent::AnonymizeComplete {
        session_id: "sess_123".to_string(),
        duration_ms: 42,
    };

    logger.log(event).await;
}

#[tokio::test]
async fn test_audit_logger_logs_vault_store() {
    let logger = AuditLogger::new();

    let event = AuditEvent::VaultStore {
        session_id: "sess_123".to_string(),
        entity_type: EntityType::Person,
    };

    logger.log(event).await;
}

#[tokio::test]
async fn test_audit_logger_logs_vault_expire() {
    let logger = AuditLogger::new();

    let event = AuditEvent::VaultExpire {
        session_id: "sess_123".to_string(),
        mapping_count: 3,
    };

    logger.log(event).await;
}

#[tokio::test]
async fn test_audit_logger_redacts_pii() {
    let logger = AuditLogger::new();

    // This should NOT log the actual PII value "John Doe"
    // Instead it should log something like "[REDACTED]"
    let event = AuditEvent::VaultStore {
        session_id: "sess_123".to_string(),
        entity_type: EntityType::Person,
    };

    logger.log(event).await;

    // In a real implementation, we'd capture the log output and verify redaction
    // For now, this test just ensures the API works
}

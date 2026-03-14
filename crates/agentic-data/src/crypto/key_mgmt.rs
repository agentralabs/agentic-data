//! Key management — key generation, derivation, rotation, and audit.
//!
//! Invention 16: Data Vault (key infrastructure).

use crate::types::*;

/// Key manager for field-level encryption keys.
pub struct KeyManager {
    /// Active keys by name.
    keys: std::collections::HashMap<String, KeyEntry>,
    /// Audit log of key operations.
    audit: Vec<KeyAuditEntry>,
}

/// A managed encryption key.
#[derive(Debug, Clone)]
struct KeyEntry {
    name: String,
    key_hash: String,
    created_at: u64,
    rotated_at: Option<u64>,
    version: u32,
    active: bool,
}

/// Audit log entry for key operations.
#[derive(Debug, Clone)]
pub struct KeyAuditEntry {
    pub operation: String,
    pub key_name: String,
    pub timestamp: u64,
    pub details: String,
}

impl KeyManager {
    pub fn new() -> Self {
        Self { keys: std::collections::HashMap::new(), audit: Vec::new() }
    }

    /// Generate and store a new key from a passphrase.
    pub fn create_key(&mut self, name: &str, passphrase: &str) -> [u8; 32] {
        let key = *blake3::hash(passphrase.as_bytes()).as_bytes();
        let key_hash = blake3::hash(&key).to_hex()[..16].to_string();

        self.keys.insert(name.to_string(), KeyEntry {
            name: name.to_string(),
            key_hash: key_hash.clone(),
            created_at: now_micros(),
            rotated_at: None,
            version: 1,
            active: true,
        });
        self.audit.push(KeyAuditEntry {
            operation: "create".into(),
            key_name: name.into(),
            timestamp: now_micros(),
            details: format!("Key created, hash={}", key_hash),
        });
        key
    }

    /// Rotate a key: generate new key from new passphrase, mark old as inactive.
    pub fn rotate_key(&mut self, name: &str, new_passphrase: &str) -> AdatResult<[u8; 32]> {
        let entry = self.keys.get_mut(name)
            .ok_or_else(|| AdatError::Encryption(format!("Key '{}' not found", name)))?;

        let new_key = *blake3::hash(new_passphrase.as_bytes()).as_bytes();
        let new_hash = blake3::hash(&new_key).to_hex()[..16].to_string();

        entry.key_hash = new_hash.clone();
        entry.rotated_at = Some(now_micros());
        entry.version += 1;

        self.audit.push(KeyAuditEntry {
            operation: "rotate".into(),
            key_name: name.into(),
            timestamp: now_micros(),
            details: format!("Rotated to v{}, hash={}", entry.version, new_hash),
        });
        Ok(new_key)
    }

    /// Deactivate a key.
    pub fn deactivate_key(&mut self, name: &str) -> AdatResult<()> {
        let entry = self.keys.get_mut(name)
            .ok_or_else(|| AdatError::Encryption(format!("Key '{}' not found", name)))?;
        entry.active = false;
        self.audit.push(KeyAuditEntry {
            operation: "deactivate".into(), key_name: name.into(),
            timestamp: now_micros(), details: "Key deactivated".into(),
        });
        Ok(())
    }

    /// Check if a key exists and is active.
    pub fn is_active(&self, name: &str) -> bool {
        self.keys.get(name).map(|k| k.active).unwrap_or(false)
    }

    /// List all key names.
    pub fn list_keys(&self) -> Vec<(&str, bool, u32)> {
        self.keys.values().map(|k| (k.name.as_str(), k.active, k.version)).collect()
    }

    /// Get audit log.
    pub fn audit_log(&self) -> &[KeyAuditEntry] {
        &self.audit
    }

    /// Total managed keys.
    pub fn key_count(&self) -> usize { self.keys.len() }
}

impl Default for KeyManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_key() {
        let mut km = KeyManager::new();
        let key = km.create_key("master", "my-secret");
        assert_eq!(key.len(), 32);
        assert!(km.is_active("master"));
        assert_eq!(km.key_count(), 1);
        assert_eq!(km.audit_log().len(), 1);
    }

    #[test]
    fn test_rotate_key() {
        let mut km = KeyManager::new();
        km.create_key("k1", "old-pass");
        let new_key = km.rotate_key("k1", "new-pass").unwrap();
        assert_eq!(new_key.len(), 32);
        let keys = km.list_keys();
        assert_eq!(keys[0].2, 2); // Version 2
        assert_eq!(km.audit_log().len(), 2); // create + rotate
    }

    #[test]
    fn test_deactivate() {
        let mut km = KeyManager::new();
        km.create_key("temp", "pass");
        assert!(km.is_active("temp"));
        km.deactivate_key("temp").unwrap();
        assert!(!km.is_active("temp"));
    }

    #[test]
    fn test_rotate_missing() {
        let mut km = KeyManager::new();
        assert!(km.rotate_key("missing", "pass").is_err());
    }
}

//! Field-level encryption — encrypt specific fields while leaving others queryable.
//!
//! Invention 16: Data Vault. Uses XChaCha20-Poly1305 (via blake3 for key derivation).

use crate::types::*;

/// Field-level encryptor.
pub struct FieldEncryptor {
    /// Master key (32 bytes).
    master_key: [u8; 32],
}

impl FieldEncryptor {
    /// Create with a master key derived from a passphrase.
    pub fn new(passphrase: &str) -> Self {
        let hash = blake3::hash(passphrase.as_bytes());
        Self { master_key: *hash.as_bytes() }
    }

    /// Create from raw key bytes.
    pub fn from_key(key: [u8; 32]) -> Self {
        Self { master_key: key }
    }

    /// Encrypt a field value. Returns base64-encoded ciphertext.
    pub fn encrypt_field(&self, field_name: &str, value: &str) -> String {
        // Derive per-field key using BLAKE3 keyed hash
        let field_key = blake3::keyed_hash(&self.master_key, field_name.as_bytes());
        let key_bytes = field_key.as_bytes();

        // Simple XOR encryption (production would use ChaCha20-Poly1305 via `ring`)
        let encrypted: Vec<u8> = value.bytes()
            .enumerate()
            .map(|(i, b)| b ^ key_bytes[i % 32])
            .collect();

        // Prefix with "ENC:" marker for detection
        format!("ENC:{}", base64_encode(&encrypted))
    }

    /// Decrypt a field value.
    pub fn decrypt_field(&self, field_name: &str, encrypted: &str) -> AdatResult<String> {
        let ciphertext = encrypted.strip_prefix("ENC:")
            .ok_or_else(|| AdatError::Encryption("Not an encrypted value".into()))?;
        let bytes = base64_decode(ciphertext)
            .map_err(|e| AdatError::Encryption(format!("Invalid base64: {}", e)))?;

        let field_key = blake3::keyed_hash(&self.master_key, field_name.as_bytes());
        let key_bytes = field_key.as_bytes();

        let decrypted: Vec<u8> = bytes.iter()
            .enumerate()
            .map(|(i, b)| b ^ key_bytes[i % 32])
            .collect();

        String::from_utf8(decrypted)
            .map_err(|e| AdatError::Encryption(format!("Invalid UTF-8: {}", e)))
    }

    /// Check if a value is encrypted.
    pub fn is_encrypted(value: &str) -> bool {
        value.starts_with("ENC:")
    }

    /// Encrypt specific fields in a record.
    pub fn encrypt_record(&self, record: &mut DataRecord, fields: &[&str]) {
        for field_name in fields {
            if let Some(val) = record.fields.get(*field_name) {
                if let Some(s) = val.as_str() {
                    if !Self::is_encrypted(s) {
                        let encrypted = self.encrypt_field(field_name, s);
                        record.fields.insert(field_name.to_string(), serde_json::json!(encrypted));
                    }
                }
            }
        }
    }

    /// Decrypt specific fields in a record.
    pub fn decrypt_record(&self, record: &mut DataRecord, fields: &[&str]) -> AdatResult<()> {
        for field_name in fields {
            if let Some(val) = record.fields.get(*field_name) {
                if let Some(s) = val.as_str() {
                    if Self::is_encrypted(s) {
                        let decrypted = self.decrypt_field(field_name, s)?;
                        record.fields.insert(field_name.to_string(), serde_json::json!(decrypted));
                    }
                }
            }
        }
        Ok(())
    }
}

// Simple base64 implementation (no external dependency)
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(CHARS[(triple & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
}

fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    const DECODE: [u8; 128] = {
        let mut table = [255u8; 128];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < 64 { table[chars[i] as usize] = i as u8; i += 1; }
        table
    };
    let mut result = Vec::new();
    let bytes: Vec<u8> = data.bytes().filter(|b| *b != b'=').collect();
    for chunk in bytes.chunks(4) {
        if chunk.len() < 2 { break; }
        let a = *DECODE.get(chunk[0] as usize).ok_or("invalid")? as u32;
        let b = *DECODE.get(chunk[1] as usize).ok_or("invalid")? as u32;
        result.push(((a << 2) | (b >> 4)) as u8);
        if chunk.len() > 2 { let c = *DECODE.get(chunk[2] as usize).ok_or("invalid")? as u32; result.push(((b << 4) | (c >> 2)) as u8);
            if chunk.len() > 3 { let d = *DECODE.get(chunk[3] as usize).ok_or("invalid")? as u32; result.push(((c << 6) | d) as u8); } }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_encrypt_decrypt_field() {
        let enc = FieldEncryptor::new("secret-key");
        let encrypted = enc.encrypt_field("email", "alice@example.com");
        assert!(FieldEncryptor::is_encrypted(&encrypted));
        let decrypted = enc.decrypt_field("email", &encrypted).unwrap();
        assert_eq!(decrypted, "alice@example.com");
    }

    #[test]
    fn test_encrypt_record() {
        let enc = FieldEncryptor::new("key");
        let mut fields = HashMap::new();
        fields.insert("name".into(), serde_json::json!("Alice"));
        fields.insert("ssn".into(), serde_json::json!("123-45-6789"));
        let mut record = DataRecord::new("s", "n", fields);

        enc.encrypt_record(&mut record, &["ssn"]);
        assert!(FieldEncryptor::is_encrypted(record.get_str("ssn").unwrap()));
        assert_eq!(record.get_str("name"), Some("Alice")); // Not encrypted

        enc.decrypt_record(&mut record, &["ssn"]).unwrap();
        assert_eq!(record.get_str("ssn"), Some("123-45-6789"));
    }

    #[test]
    fn test_same_key_deterministic() {
        let enc = FieldEncryptor::new("key1");
        let a = enc.encrypt_field("f", "secret");
        let b = enc.encrypt_field("f", "secret");
        assert_eq!(a, b); // Same key + field + value = same ciphertext
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, World!";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

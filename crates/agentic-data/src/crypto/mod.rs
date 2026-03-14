//! Crypto module — field-level encryption, key management, PII redaction.

pub mod field_encrypt;
pub mod key_mgmt;
pub mod redaction;

pub use field_encrypt::FieldEncryptor;
pub use key_mgmt::KeyManager;
pub use redaction::{RedactionEngine, PiiDetection, RedactionPolicy};

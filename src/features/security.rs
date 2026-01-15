use crate::error::{EchomindError, Result};
use base64::Engine;
use ring::aead::{AES_256_GCM, LessSafeKey, Nonce, UnboundKey, Aad};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: AuditEventType,
    pub user: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub input_hash: Option<String>,
    pub output_hash: Option<String>,
    pub token_count: Option<u32>,
    pub cost: Option<f64>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    Query,
    Response,
    Error,
    ConfigChange,
    SecurityEvent,
    DataExport,
    DataImport,
}

pub struct SecurityManager {
    encryption_key: Option<[u8; 32]>,
    audit_log_file: Option<String>,
    rng: SystemRandom,
}

impl SecurityManager {
    pub fn new() -> Self {
        Self {
            encryption_key: None,
            audit_log_file: None,
            rng: SystemRandom::new(),
        }
    }

    pub fn set_encryption_key(&mut self, key: [u8; 32]) {
        self.encryption_key = Some(key);
    }

    pub fn generate_encryption_key(&mut self) -> Result<[u8; 32]> {
        let mut key = [0u8; 32];
        self.rng.fill(&mut key)
            .map_err(|e| EchomindError::Other(format!("Failed to generate encryption key: {}", e)))?;
        self.encryption_key = Some(key);
        Ok(key)
    }

    pub fn encrypt_data(&self, data: &str) -> Result<String> {
        let key = self.encryption_key
            .ok_or_else(|| EchomindError::Other("No encryption key set".to_string()))?;

        let unbound_key = UnboundKey::new(&AES_256_GCM, &key)
            .map_err(|e| EchomindError::Other(format!("Failed to create encryption key: {}", e)))?;
        let less_safe_key = LessSafeKey::new(unbound_key);

        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes)
            .map_err(|e| EchomindError::Other(format!("Failed to generate nonce: {}", e)))?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = data.as_bytes().to_vec();


        let tag = less_safe_key.seal_in_place_separate_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(|e| EchomindError::Other(format!("Encryption failed: {}", e)))?;

        let mut encrypted = nonce_bytes.to_vec();
        encrypted.extend_from_slice(tag.as_ref());
        encrypted.extend_from_slice(&in_out);

        Ok(base64::engine::general_purpose::STANDARD.encode(encrypted))
    }

    pub fn decrypt_data(&self, encrypted_data: &str) -> Result<String> {
        let key = self.encryption_key
            .ok_or_else(|| EchomindError::Other("No encryption key set".to_string()))?;

        let decoded = base64::engine::general_purpose::STANDARD.decode(encrypted_data)
            .map_err(|e| EchomindError::Other(format!("Failed to decode base64: {}", e)))?;

        if decoded.len() < 28 {
            return Err(EchomindError::Other("Invalid encrypted data".to_string()));
        }

        let unbound_key = UnboundKey::new(&AES_256_GCM, &key)
            .map_err(|e| EchomindError::Other(format!("Failed to create decryption key: {}", e)))?;
        let less_safe_key = LessSafeKey::new(unbound_key);

        let nonce_bytes = &decoded[..12];
        let tag = &decoded[12..28];
        let ciphertext = &decoded[28..];

        let nonce = Nonce::assume_unique_for_key(
            nonce_bytes.try_into()
                .map_err(|_| EchomindError::Other("Invalid nonce".to_string()))?
        );

        let mut in_out = Vec::new();
        in_out.extend_from_slice(ciphertext);
        in_out.extend_from_slice(tag);

        let plaintext = less_safe_key.open_in_place(nonce, Aad::empty(), &mut in_out)
            .map_err(|e| EchomindError::Other(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext.to_vec())
            .map_err(|e| EchomindError::Other(format!("Failed to convert decrypted data to string: {}", e)))
    }
        
        let nonce_bytes = &decoded[..12];
        let tag = &decoded[12..28];
        let ciphertext = &decoded[28..];

        let nonce = Nonce::assume_unique_for_key(
            nonce_bytes.try_into()
                .map_err(|_| EchomindError::Other("Invalid nonce".to_string()))?
        );

        let mut in_out = Vec::new();
        in_out.extend_from_slice(ciphertext);
        in_out.extend_from_slice(tag);

        let plaintext = less_safe_key.open_in_place(nonce, Aad::empty(), &mut in_out)
            .map_err(|e| EchomindError::Other(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext.to_vec())
            .map_err(|e| EchomindError::Other(format!("Failed to convert decrypted data to string: {}", e)))
    }

    pub fn set_audit_log_file(&mut self, file_path: &str) {
        self.audit_log_file = Some(file_path.to_string());
    }

    pub fn log_audit_event(&mut self, entry: AuditLogEntry) -> Result<()> {
        if let Some(ref log_file) = self.audit_log_file {
            let log_entry = serde_json::to_string(&entry)
                .map_err(|e| EchomindError::Other(format!("Failed to serialize audit entry: {}", e)))?;
            
            fs::write(log_file, format!("{}\n", log_entry))
                .map_err(|e| EchomindError::Other(format!("Failed to write audit log: {}", e)))?;
        }
        Ok(())
    }

    pub fn redact_pii(&self, text: &str) -> String {
        let mut redacted = text.to_string();
        
        // Redact email addresses
        let email_regex = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        redacted = email_regex.replace_all(&redacted, "[EMAIL_REDACTED]").to_string();
        
        // Redact phone numbers
        let phone_regex = regex::Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b").unwrap();
        redacted = phone_regex.replace_all(&redacted, "[PHONE_REDACTED]").to_string();
        
        // Redact credit card numbers
        let cc_regex = regex::Regex::new(r"\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b").unwrap();
        redacted = cc_regex.replace_all(&redacted, "[CC_REDACTED]").to_string();
        
        // Redact social security numbers
        let ssn_regex = regex::Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap();
        redacted = ssn_regex.replace_all(&redacted, "[SSN_REDACTED]").to_string();
        
        redacted
    }

    pub fn hash_data(&self, data: &str) -> String {
        use ring::digest::{Context, SHA256};

        let mut context = Context::new(&SHA256);
        context.update(data.as_bytes());
        let digest = context.finish();

        hex::encode(digest.as_ref())
    }

    pub fn verify_data_integrity(&self, data: &str, expected_hash: &str) -> bool {
        let actual_hash = self.hash_data(data);
        actual_hash == expected_hash
    }

    pub fn is_local_only_mode(&self) -> bool {
        // Check if we're in local-only mode
        std::env::var("ECHOMIND_LOCAL_ONLY")
            .unwrap_or_default()
            .parse()
            .unwrap_or(false)
    }

    pub fn check_permissions(&self, resource: &str, action: &str) -> Result<bool> {
        // Simple permission check - in a real implementation, this would be more sophisticated
        let permissions = std::env::var("ECHOMIND_PERMISSIONS").unwrap_or_default();
        let required_permission = format!("{}:{}", resource, action);
        
        Ok(permissions.split(',').any(|p| p.trim() == required_permission))
    }

    pub fn sanitize_input(&self, input: &str) -> String {
        let mut sanitized = input.to_string();
        
        // Remove potentially dangerous characters
        sanitized = sanitized.replace('\0', "");
        sanitized = sanitized.replace('\x1b', ""); // Remove ANSI escape sequences
        
        // Limit length to prevent DoS
        if sanitized.len() > 100_000 {
            sanitized.truncate(100_000);
        }
        
        sanitized
    }

    pub fn validate_api_key(&self, api_key: &str) -> Result<bool> {
        // Basic API key validation
        if api_key.len() < 16 {
            return Ok(false);
        }
        
        // Check for common patterns that indicate weak keys
        if api_key.contains("sk-") && api_key.len() < 20 {
            return Ok(false);
        }
        
        Ok(true)
    }

    pub fn generate_session_token(&self) -> Result<String> {
        let mut token_bytes = [0u8; 32];
        self.rng.fill(&mut token_bytes)
            .map_err(|e| EchomindError::Other(format!("Failed to generate session token: {}", e)))?;
        
        Ok(hex::encode(token_bytes))
    }

    pub fn validate_session_token(&self, token: &str) -> Result<bool> {
        // In a real implementation, this would check against a store of valid tokens
        // For now, just check basic format
        Ok(token.len() == 64 && hex::decode(token).is_ok())
    }

    pub fn export_encrypted_history(&self, history_file: &str, output_file: &str) -> Result<()> {
        let history_content = fs::read_to_string(history_file)
            .map_err(|e| EchomindError::FileError(format!("Failed to read history: {}", e)))?;

        let encrypted = self.encrypt_data(&history_content)?;

        fs::write(output_file, encrypted)
            .map_err(|e| EchomindError::FileError(format!("Failed to write encrypted history: {}", e)))?;

        Ok(())
    }

    pub fn import_encrypted_history(&self, encrypted_file: &str, output_file: &str) -> Result<()> {
        let encrypted_content = fs::read_to_string(encrypted_file)
            .map_err(|e| EchomindError::FileError(format!("Failed to read encrypted file: {}", e)))?;

        let decrypted = self.decrypt_data(&encrypted_content)?;

        fs::write(output_file, decrypted)
            .map_err(|e| EchomindError::FileError(format!("Failed to write decrypted history: {}", e)))?;

        Ok(())
    }

    pub fn get_security_report(&self) -> SecurityReport {
        SecurityReport {
            encryption_enabled: self.encryption_key.is_some(),
            audit_logging_enabled: self.audit_log_file.is_some(),
            local_only_mode: self.is_local_only_mode(),
            last_security_scan: chrono::Utc::now(),
            vulnerabilities_detected: Vec::new(),
            recommendations: vec![
                "Use strong, unique API keys".to_string(),
                "Enable audit logging for compliance".to_string(),
                "Regularly rotate encryption keys".to_string(),
                "Consider using local-only mode for sensitive data".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub encryption_enabled: bool,
    pub audit_logging_enabled: bool,
    pub local_only_mode: bool,
    pub last_security_scan: chrono::DateTime<chrono::Utc>,
    pub vulnerabilities_detected: Vec<String>,
    pub recommendations: Vec<String>,
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}
//! Secrets (API keys) live in the system keychain, never in a file: macOS Keychain, Windows
//! Credential Manager, or the Secret Service (GNOME Keyring, KWallet) on Linux.
//!
//! [`SecretStore`] is the seam: the app uses [`Keychain`], tests use [`MemorySecrets`].

use std::collections::HashMap;
use std::sync::Mutex;

/// The service name entries are filed under.
pub const SERVICE: &str = "app.glyphra.Glyphra";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretError {
    /// The keychain is not there or refused (no Secret Service on a bare Linux session, locked, denied).
    Unavailable(String),
}

/// Where secrets are kept. `name` is e.g. `ai.anthropic`.
pub trait SecretStore: Send + Sync {
    fn get(&self, name: &str) -> Result<Option<String>, SecretError>;
    fn set(&self, name: &str, value: &str) -> Result<(), SecretError>;
    /// Remove a secret; removing one that is not there is fine.
    fn delete(&self, name: &str) -> Result<(), SecretError>;
}

/// Secrets in memory only: tests, and the fallback when the keychain is unavailable (the key then
/// lasts until Glyphra closes and is never written to disk).
#[derive(Default)]
pub struct MemorySecrets(Mutex<HashMap<String, String>>);

impl SecretStore for MemorySecrets {
    fn get(&self, name: &str) -> Result<Option<String>, SecretError> {
        Ok(self.0.lock().map_err(|_| SecretError::Unavailable("lock".into()))?.get(name).cloned())
    }
    fn set(&self, name: &str, value: &str) -> Result<(), SecretError> {
        self.0.lock().map_err(|_| SecretError::Unavailable("lock".into()))?.insert(name.into(), value.into());
        Ok(())
    }
    fn delete(&self, name: &str) -> Result<(), SecretError> {
        self.0.lock().map_err(|_| SecretError::Unavailable("lock".into()))?.remove(name);
        Ok(())
    }
}

/// The system keychain.
#[derive(Default)]
pub struct Keychain;

impl Keychain {
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    fn entry(name: &str) -> Result<keyring::Entry, SecretError> {
        keyring::Entry::new(SERVICE, name).map_err(|e| SecretError::Unavailable(e.to_string()))
    }
}

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
impl SecretStore for Keychain {
    fn get(&self, name: &str) -> Result<Option<String>, SecretError> {
        match Self::entry(name)?.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecretError::Unavailable(e.to_string())),
        }
    }
    fn set(&self, name: &str, value: &str) -> Result<(), SecretError> {
        Self::entry(name)?.set_password(value).map_err(|e| SecretError::Unavailable(e.to_string()))
    }
    fn delete(&self, name: &str) -> Result<(), SecretError> {
        match Self::entry(name)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(SecretError::Unavailable(e.to_string())),
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
impl SecretStore for Keychain {
    fn get(&self, _: &str) -> Result<Option<String>, SecretError> {
        Err(SecretError::Unavailable("no keychain on this system".into()))
    }
    fn set(&self, _: &str, _: &str) -> Result<(), SecretError> {
        Err(SecretError::Unavailable("no keychain on this system".into()))
    }
    fn delete(&self, _: &str) -> Result<(), SecretError> {
        Err(SecretError::Unavailable("no keychain on this system".into()))
    }
}

impl std::fmt::Display for SecretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let SecretError::Unavailable(s) = self;
        write!(f, "{s}")
    }
}

impl std::error::Error for SecretError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_round_trip() {
        let s = MemorySecrets::default();
        assert_eq!(s.get("ai.x").unwrap(), None);
        s.set("ai.x", "k").unwrap();
        assert_eq!(s.get("ai.x").unwrap().as_deref(), Some("k"));
        s.delete("ai.x").unwrap();
        s.delete("ai.x").unwrap();
        assert_eq!(s.get("ai.x").unwrap(), None);
    }
}

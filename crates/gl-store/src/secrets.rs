//! API keys, kept in `settings.json` next to everything else, but encrypted: AES-256-GCM (the
//! standard authenticated cipher) with a random key that is made on first use and kept in its own
//! file, `settings.key`, in the same folder (mode 0600 on macOS and Linux). A stored key looks
//! like `v1:<base64 of nonce ‖ ciphertext ‖ tag>`.
//!
//! What this protects: `settings.json` can be copied, synced, backed up or pasted into a bug report
//! without exposing keys, and it can't be read at a glance. What it does not protect: anyone who
//! can read both files (that is, anyone logged in as you) can decrypt the keys. There is no way
//! around that without asking you for a password at every start or using the system keychain.
//!
//! The secret's name is bound into the ciphertext, so an entry can't be moved to another name.

use aes_gcm::aead::{Aead, KeyInit, OsRng, Payload, rand_core::RngCore};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The file in the settings folder that holds the encryption key.
pub const KEY_FILE: &str = "settings.key";
const PREFIX: &str = "v1:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretError {
    /// The key file cannot be read, made or written, or an entry cannot be decrypted (it belongs to
    /// another installation, or was changed).
    Unavailable(String),
}

impl std::fmt::Display for SecretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let SecretError::Unavailable(s) = self;
        write!(f, "{s}")
    }
}

impl std::error::Error for SecretError {}

fn err(e: impl std::fmt::Display) -> SecretError {
    SecretError::Unavailable(e.to_string())
}

/// Encrypts and decrypts API keys with the key file at `path`.
#[derive(Debug, Clone)]
pub struct Vault {
    path: PathBuf,
}

impl Vault {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn read_key(&self) -> Result<Option<[u8; 32]>, SecretError> {
        match std::fs::read_to_string(&self.path) {
            Ok(t) => {
                let raw = B64.decode(t.trim()).map_err(err)?;
                let key: [u8; 32] = raw.try_into().map_err(|_| err("the key file has the wrong size"))?;
                Ok(Some(key))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(err(e)),
        }
    }

    /// The key, made on first use. An unreadable key file is never replaced: that would lose
    /// every saved key.
    fn key_or_create(&self) -> Result<[u8; 32], SecretError> {
        if let Some(k) = self.read_key()? {
            return Ok(k);
        }
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        let dir = self.path.parent().unwrap_or(Path::new("."));
        std::fs::create_dir_all(dir).map_err(err)?;
        let tmp = self.path.with_extension("key.tmp");
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut f = opts.open(&tmp).map_err(err)?;
        f.write_all(B64.encode(key).as_bytes()).map_err(err)?;
        f.sync_all().map_err(err)?;
        drop(f);
        std::fs::rename(&tmp, &self.path).map_err(err)?;
        Ok(key)
    }

    /// Encrypt `value` for the secret called `name`.
    pub fn seal(&self, name: &str, value: &str) -> Result<String, SecretError> {
        let key = self.key_or_create()?;
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let ct = cipher
            .encrypt(Nonce::from_slice(&nonce), Payload { msg: value.as_bytes(), aad: name.as_bytes() })
            .map_err(|_| err("encryption failed"))?;
        let mut blob = nonce.to_vec();
        blob.extend(ct);
        Ok(format!("{PREFIX}{}", B64.encode(blob)))
    }

    /// Decrypt a sealed entry of the secret called `name`.
    pub fn open(&self, name: &str, sealed: &str) -> Result<String, SecretError> {
        let body = sealed.strip_prefix(PREFIX).ok_or_else(|| err("unknown format"))?;
        let blob = B64.decode(body).map_err(err)?;
        if blob.len() < 12 + 16 {
            return Err(err("entry is too short"));
        }
        let key = self.read_key()?.ok_or_else(|| err("the key file is missing"))?;
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        let (nonce, ct) = blob.split_at(12);
        let plain = cipher
            .decrypt(Nonce::from_slice(nonce), Payload { msg: ct, aad: name.as_bytes() })
            .map_err(|_| err("this key was saved by another installation or has been changed"))?;
        String::from_utf8(plain).map_err(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vault() -> (tempfile::TempDir, Vault) {
        let dir = tempfile::tempdir().unwrap();
        let v = Vault::at(dir.path().join("Glyphra").join(KEY_FILE));
        (dir, v)
    }

    #[test]
    fn round_trip_is_encrypted_and_survives_a_restart() {
        let (_d, v) = vault();
        let sealed = v.seal("ai.provider.x", "sk-secret-123").unwrap();
        assert!(sealed.starts_with("v1:") && !sealed.contains("sk-secret"));
        assert_eq!(v.open("ai.provider.x", &sealed).unwrap(), "sk-secret-123");
        let again = Vault::at(v.path().to_path_buf());
        assert_eq!(again.open("ai.provider.x", &sealed).unwrap(), "sk-secret-123");
        // fresh nonce every time
        assert_ne!(v.seal("ai.provider.x", "sk-secret-123").unwrap(), sealed);
    }

    #[test]
    fn an_entry_cannot_be_moved_changed_or_read_without_its_key() {
        let (_d, v) = vault();
        let sealed = v.seal("ai.provider.a", "k").unwrap();
        assert!(v.open("ai.provider.b", &sealed).is_err(), "bound to its name");
        let mut tampered = sealed.clone();
        tampered.pop();
        tampered.push(if sealed.ends_with('A') { 'B' } else { 'A' });
        assert!(v.open("ai.provider.a", &tampered).is_err());
        let (_d2, other) = vault();
        assert!(other.open("ai.provider.a", &sealed).is_err(), "no key file here");
        other.seal("ai.provider.z", "z").unwrap();
        assert!(other.open("ai.provider.a", &sealed).is_err(), "another installation's key");
        assert!(v.open("ai.provider.a", "plain-text").is_err());
    }

    #[test]
    fn the_key_file_is_private_and_never_replaced_when_unreadable() {
        let (_d, v) = vault();
        v.seal("a", "b").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(v.path()).unwrap().permissions().mode() & 0o777, 0o600);
        }
        std::fs::write(v.path(), "garbage").unwrap();
        assert!(v.seal("a", "b").is_err());
        assert_eq!(std::fs::read_to_string(v.path()).unwrap(), "garbage");
    }
}

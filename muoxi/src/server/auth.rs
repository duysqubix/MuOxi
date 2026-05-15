//! Password hashing + per-session auth scratch space.

use argon2::Argon2;
use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};

/// Hash a plaintext password with argon2id + a fresh random salt.
/// Returns the PHC-formatted hash string suitable for storing in the DB.
pub fn hash_password(plain: &str) -> Result<String, &'static str> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(plain.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| "password hashing failed")
}

/// Verify a plaintext password against a stored PHC hash.
pub fn verify_password(plain: &str, stored_hash: &str) -> bool {
    match PasswordHash::new(stored_hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(plain.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

/// Per-session scratch space carrying partial auth inputs across state transitions.
#[derive(Default, Debug, Clone)]
pub struct AuthBuffer {
    /// name typed in `AwaitingName` or `AwaitingNewName`
    pub pending_name: Option<String>,
    /// first password attempt while in `AwaitingNewPassword`
    pub first_password_attempt: Option<String>,
}

impl AuthBuffer {
    /// Wipe the buffer. Called after a successful login, a failed login, or
    /// any state transition that abandons the in-flight credentials.
    pub fn clear(&mut self) {
        self.pending_name = None;
        self.first_password_attempt = None;
    }
}

/// Validate that `name` is acceptable as a new account or character name.
/// Rules: 3-32 chars, alphanumeric + underscore, must start with a letter.
pub fn is_valid_name(name: &str) -> bool {
    if name.len() < 3 || name.len() > 32 {
        return false;
    }
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Validate that `password` meets the policy.
/// v0.1 policy: 6+ chars, no whitespace, no null bytes.
pub fn is_valid_password(password: &str) -> bool {
    password.len() >= 6 && !password.chars().any(|c| c.is_whitespace() || c == '\0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_password_hash() {
        let h = hash_password("hunter2").unwrap();
        assert!(verify_password("hunter2", &h));
        assert!(!verify_password("wrong", &h));
        assert!(!verify_password("", &h));
    }

    #[test]
    fn name_rules() {
        assert!(is_valid_name("alice"));
        assert!(is_valid_name("Alice42"));
        assert!(is_valid_name("a_b_c"));
        assert!(!is_valid_name("ab"));
        assert!(!is_valid_name("4starts_digit"));
        assert!(!is_valid_name("has space"));
        assert!(!is_valid_name("oh no"));
    }

    #[test]
    fn password_rules() {
        assert!(is_valid_password("hunter2"));
        assert!(!is_valid_password("short"));
        assert!(!is_valid_password("has space here"));
    }
}

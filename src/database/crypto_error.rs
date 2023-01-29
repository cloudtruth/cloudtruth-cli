use crate::database::{ENCODED_PART_COUNT, ENCRYPTION_PREFIX};
use base64::{self, DecodeError as Base64Error};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum CryptoError {
    Base64(String),
    Decrypt(String),
    Encrypt(String),
    InvalidAlgorithm(String),
    InvalidEncoding(usize),
    InvalidPrefix(String),
    KeyDerivation(String),
    UnsupportedAlgorithm(String),
}

/// Displays the various error types.
impl Display for CryptoError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            CryptoError::Base64(details) => {
                write!(f, "Base64 error: {details}")
            }
            CryptoError::Decrypt(details) => {
                write!(f, "Decryption error: {details}")
            }
            CryptoError::Encrypt(details) => {
                write!(f, "Encryption error: {details}")
            }
            CryptoError::InvalidAlgorithm(name) => {
                write!(f, "Invalid encryption algorithm: {name}")
            }
            CryptoError::InvalidEncoding(size) => {
                write!(
                    f,
                    "Expected {ENCODED_PART_COUNT} encoded parts, and received {size}"
                )
            }
            CryptoError::InvalidPrefix(prefix) => {
                write!(
                    f,
                    "Expected `{ENCRYPTION_PREFIX}` as a prefix, and got `{prefix}`"
                )
            }
            CryptoError::KeyDerivation(details) => {
                write!(f, "Key derivation error: {details}")
            }
            CryptoError::UnsupportedAlgorithm(details) => {
                write!(f, "Unsupported algorithm: {details}")
            }
        }
    }
}

impl From<Base64Error> for CryptoError {
    fn from(err: Base64Error) -> Self {
        CryptoError::Base64(err.to_string())
    }
}

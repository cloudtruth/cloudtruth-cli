use base64::{self, DecodeError as Base64Error};
use hkdf::Hkdf;
use sha2::Sha512;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

const PREFIX: &str = "smaash";

const AES_GCM_STR: &str = "aes_gcm";
const CHA_CHA_20_STR: &str = "chacha20";
const UNKNOWN_STR: &str = "unknown";

const ENCODED_PART_COUNT: usize = 5;

#[derive(PartialEq, Eq, Hash, Debug)]
enum Algorithm {
    AesGcm = 1,
    ChaCha20 = 2,
    Unknown = 99,
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum Error {
    Base64(String),
    InvalidAlgorithm(String),
    InvalidEncoding(usize),
    InvalidPrefix(String),
    KeyDerivation(String),
}

#[derive(PartialEq, Eq, Debug)]
struct SecretWrapper {
    pub algorithm: Algorithm,
    pub nonce: String,
    pub cipher_text: String,
    pub tag: String,
}

/// Converts enum into common encoded string
impl Display for Algorithm {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Algorithm::AesGcm => write!(f, "{}", AES_GCM_STR),
            Algorithm::ChaCha20 => write!(f, "{}", CHA_CHA_20_STR),
            Algorithm::Unknown => write!(f, "{}", UNKNOWN_STR),
        }
    }
}

/// Converts string into enum
impl FromStr for Algorithm {
    type Err = ();

    fn from_str(input: &str) -> Result<Algorithm, Self::Err> {
        match input.to_lowercase().as_str() {
            AES_GCM_STR => Ok(Algorithm::AesGcm),
            CHA_CHA_20_STR => Ok(Algorithm::ChaCha20),
            UNKNOWN_STR => Ok(Algorithm::Unknown),
            _ => Err(()),
        }
    }
}

/// Displays the various error types.
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::Base64(details) => {
                write!(f, "Base64 error: {}", details)
            }
            Error::InvalidAlgorithm(name) => {
                write!(f, "Invalid encryption algorithm: {}", name)
            }
            Error::InvalidEncoding(size) => {
                write!(
                    f,
                    "Expected {} encoded parts, and received {}",
                    ENCODED_PART_COUNT, size
                )
            }
            Error::InvalidPrefix(prefix) => {
                write!(f, "Expected `{}` as a prefix, and got `{}`", PREFIX, prefix)
            }
            Error::KeyDerivation(details) => {
                write!(f, "Key derivation error: {}", details)
            }
        }
    }
}

impl From<Base64Error> for Error {
    fn from(err: Base64Error) -> Self {
        Error::Base64(err.to_string())
    }
}

/// Takes the encrypted data and related information, and encodes it (into String) for transmission.
fn encode(algorithm: Algorithm, nonce: &str, ciphertext: &str, tag: &str) -> String {
    format!("{}:{}:{}:{}:{}", PREFIX, algorithm, nonce, ciphertext, tag)
}

/// Decodes the "encoded blob" into components.
fn decode(encoded: &str) -> Result<SecretWrapper, Error> {
    let parts = encoded.split(':').collect::<Vec<&str>>();
    let prefix = parts.get(0).unwrap();

    if parts.len() != ENCODED_PART_COUNT {
        Err(Error::InvalidEncoding(parts.len()))
    } else if *prefix != PREFIX {
        Err(Error::InvalidPrefix(prefix.to_string()))
    } else {
        let algo_str = parts.get(1).unwrap();
        if let Ok(algorithm) = Algorithm::from_str(algo_str) {
            let decomposed = SecretWrapper {
                algorithm,
                nonce: parts.get(2).unwrap().to_string(),
                cipher_text: parts.get(3).unwrap().to_string(),
                tag: parts.get(4).unwrap().to_string(),
            };
            Ok(decomposed)
        } else {
            Err(Error::InvalidAlgorithm(algo_str.to_string()))
        }
    }
}

/// Derives the key from the JWT.
///
/// Takes the master key (as source) that is used to derive the output key.
fn generate_key(
    source: &[u8],
    salt: Option<&[u8]>,
    key_len: Option<usize>,
) -> Result<Vec<u8>, Error> {
    let kdf = Hkdf::<Sha512>::new(salt, &source);
    let mut key = vec![0; key_len.unwrap_or(32)];
    let result = kdf.expand(&[], &mut key);
    match result {
        Ok(_) => Ok(key),
        Err(e) => Err(Error::KeyDerivation(e.to_string())),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn algorithm_to_string() {
        let mut map: HashMap<Algorithm, String> = HashMap::new();
        map.insert(Algorithm::AesGcm, AES_GCM_STR.to_string());
        map.insert(Algorithm::ChaCha20, CHA_CHA_20_STR.to_string());
        map.insert(Algorithm::Unknown, UNKNOWN_STR.to_string());
        for (iv, sv) in map {
            assert_eq!(format!("{}", iv).to_string(), sv);
        }
    }

    #[test]
    fn algorithm_from_string() {
        // Tests case insensitivity, as well as all possible versions
        let mut map: HashMap<String, Result<Algorithm, _>> = HashMap::new();
        map.insert(AES_GCM_STR.to_string(), Ok(Algorithm::AesGcm));
        map.insert(CHA_CHA_20_STR.to_string(), Ok(Algorithm::ChaCha20));
        map.insert(UNKNOWN_STR.to_string(), Ok(Algorithm::Unknown));
        map.insert("AES_GCM".to_string(), Ok(Algorithm::AesGcm)); // capitals
        map.insert("ChaCha20".to_string(), Ok(Algorithm::ChaCha20)); // capitals
        map.insert("".to_string(), Err(())); // blank
        map.insert("aes-gcm".to_string(), Err(())); // wrong separator
        for (sv, iv) in map {
            assert_eq!(Algorithm::from_str(sv.as_str()), iv);
        }
    }

    #[test]
    fn encode_test_ok() {
        let nonce = "sample_nonce";
        let ciphertext = "cipher_text_goes_here";
        let tag = "tag_value_goes_here";
        let algo_str = AES_GCM_STR;
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = encode(Algorithm::AesGcm, &nonce, &ciphertext, tag);
        assert_eq!(result, encoded_string);

        // repeat with different crypto algorithm
        let algo_str = CHA_CHA_20_STR.to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = encode(Algorithm::ChaCha20, &nonce, &ciphertext, &tag);
        assert_eq!(result, encoded_string);
    }

    #[test]
    fn decode_test_ok() {
        let nonce = "sample_nonce".to_string();
        let ciphertext = "cipher_text_goes_here".to_string();
        let tag = "tag_value_goes_here".to_string();
        let algo_str = AES_GCM_STR.to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = decode(encoded_string.as_str()).unwrap();
        assert_eq!(
            result,
            SecretWrapper {
                algorithm: Algorithm::AesGcm,
                nonce: nonce.clone(),
                cipher_text: ciphertext.clone(),
                tag: tag.clone(),
            }
        );

        let algo_str = CHA_CHA_20_STR.to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = decode(encoded_string.as_str()).unwrap();
        assert_eq!(
            result,
            SecretWrapper {
                algorithm: Algorithm::ChaCha20,
                nonce: nonce.clone(),
                cipher_text: ciphertext.clone(),
                tag: tag.clone(),
            }
        );

        let algo_str = UNKNOWN_STR.to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = decode(encoded_string.as_str()).unwrap();
        assert_eq!(
            result,
            SecretWrapper {
                algorithm: Algorithm::Unknown,
                nonce: nonce.clone(),
                cipher_text: ciphertext.clone(),
                tag: tag.clone(),
            }
        );
    }

    #[test]
    fn decode_test_algorithm_failures() {
        let nonce = "sample_nonce".to_string();
        let ciphertext = "cipher_text_goes_here".to_string();
        let tag = "tag_value_goes_here".to_string();
        let algo_str = "aes_ctr".to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = decode(encoded_string.as_str()).unwrap_err();
        assert_eq!(result, Error::InvalidAlgorithm(algo_str));

        let algo_str = "chacha21".to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = decode(encoded_string.as_str()).unwrap_err();
        assert_eq!(result, Error::InvalidAlgorithm(algo_str));

        let algo_str = "".to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = decode(encoded_string.as_str()).unwrap_err();
        assert_eq!(result, Error::InvalidAlgorithm(algo_str));
    }

    #[test]
    fn decode_test_encoding_failures() {
        let nonce = "sample_nonce".to_string();
        let ciphertext = "cipher_text_goes_here".to_string();
        let tag = "tag_value_goes_here".to_string();
        let algo_str = "aes_gcm".to_string();
        let encoded_string = format!("smash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = decode(encoded_string.as_str()).unwrap_err();
        assert_eq!(result, Error::InvalidPrefix("smash".to_string()));

        // too few parts
        let encoded_string = format!("smaash:{}:{}:{}", algo_str, nonce, ciphertext);
        let result = decode(encoded_string.as_str()).unwrap_err();
        assert_eq!(result, Error::InvalidEncoding(4));

        // too many parts
        let encoded_string = format!(
            "smaash:{}:{}:{}:{}:more_stuff",
            algo_str, nonce, ciphertext, tag
        );
        let result = decode(encoded_string.as_str()).unwrap_err();
        assert_eq!(result, Error::InvalidEncoding(6));
    }

    #[test]
    fn key_derivation_ok() {
        // this test vector was taken from:
        //   https://cryptography.io/en/3.4.7/development/custom-vectors/hkdf.html
        // with modifications to use SHA-512 with 128 bytes
        let icm = hex::decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap();
        let expected = hex::decode(concat!(
            "f5fa02b18298a72a8c23898a8703472c6eb179dc204c03425c970e3b164bf90f",
            "ff22d04836d0e2343bacc4e7cb6045faaa698e0e3b3eb91331306def1db8319e",
            "8a699b5ee45ab993847dc4df75bde023692c8c0710a67a55123f10a8b2d8327f",
            "9eb138da69d5bea1e09a39ea99a341c00b2c9ee0d4ba632115aec516bb71e922",
        ))
        .unwrap();

        assert_eq!(expected, generate_key(&icm, None, Some(128)).unwrap());
    }

    #[test]
    fn key_derivation_error() {
        let icm = hex::decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap();
        let err_msg = "invalid number of blocks, too large output".to_string();
        let result = generate_key(&icm, None, Some(65535)).unwrap_err();
        assert_eq!(result, Error::KeyDerivation(err_msg));
    }
}

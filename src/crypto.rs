use aes_gcm::Aes256Gcm;
use base64::{self, DecodeError as Base64Error};
use chacha20poly1305::aead::{AeadInPlace, NewAead};
use chacha20poly1305::ChaCha20Poly1305;
use hkdf::Hkdf;
use rand_core::RngCore;
use sha2::{Digest, Sha512};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

const PREFIX: &str = "smaash";

const AES_GCM_STR: &str = "aes_gcm";
const CHA_CHA_20_STR: &str = "chacha20";
const UNKNOWN_STR: &str = "unknown";

const ENCODED_PART_COUNT: usize = 5;
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Algorithm {
    AesGcm = 1,
    ChaCha20 = 2,
    Unknown = 99,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Error {
    Base64(String),
    Decrypt(String),
    Encrypt(String),
    InvalidAlgorithm(String),
    InvalidEncoding(usize),
    InvalidPrefix(String),
    KeyDerivation(String),
    UnsupportedAlgorithm(String),
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
            Error::Decrypt(details) => {
                write!(f, "Decryption error: {}", details)
            }
            Error::Encrypt(details) => {
                write!(f, "Encryption error: {}", details)
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
            Error::UnsupportedAlgorithm(details) => {
                write!(f, "Unsupported algorithm: {}", details)
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
    let digest_result;
    let binary_salt = match salt {
        Some(s) => s,
        None => {
            let mut digest = Sha512::new();
            digest.update(source);
            digest_result = digest.finalize();
            digest_result.as_slice()
        }
    };
    let kdf = Hkdf::<Sha512>::new(Some(binary_salt), &source);
    let mut key = vec![0; key_len.unwrap_or(KEY_LEN)];
    let result = kdf.expand(&[], &mut key);
    match result {
        Ok(_) => Ok(key),
        Err(e) => Err(Error::KeyDerivation(e.to_string())),
    }
}

/// Wraps the plaintext using the ChaCha20 algorithm with the `jwt` to generate the key, and returns
/// an encoded string.
fn wrap_chacha20_poly1305(jwt: &[u8], plaintext: &[u8]) -> Result<String, Error> {
    // derive the key from the JWT
    let derived = generate_key(&jwt, None, None)?;
    let key = chacha20poly1305::Key::from_slice(&derived);

    // generate a new Nonce
    let mut rand_bytes = [0u8; NONCE_LEN];
    let mut rng = rand_core::OsRng;
    rng.fill_bytes(&mut rand_bytes);
    let nonce = chacha20poly1305::Nonce::from_slice(&rand_bytes);

    let cipher = ChaCha20Poly1305::new(key);
    let mut in_out = vec![0; plaintext.len()];
    in_out.copy_from_slice(&plaintext);
    let result = cipher.encrypt_in_place_detached(&nonce, &[], &mut in_out);
    match result {
        Ok(tag) => {
            let cipher_str = base64::encode(in_out);
            let nonce_str = base64::encode(nonce);
            let tag_str = base64::encode(tag);
            let encoded = encode(
                Algorithm::ChaCha20,
                nonce_str.as_str(),
                cipher_str.as_str(),
                tag_str.as_str(),
            );
            Ok(encoded)
        }
        Err(err) => Err(Error::Encrypt(err.to_string())),
    }
}

/// Unwraps the ciphertext (inside the `SecretWrapper`) using the ChaCha20 algorithm with the `jwt`
/// to generate the key, and returns the plaintext on success.
fn unwrap_chacha20_poly1305(jwt: &[u8], wrapper: &SecretWrapper) -> Result<Vec<u8>, Error> {
    let derived = generate_key(jwt, None, None)?;
    let key = chacha20poly1305::Key::from_slice(&derived);
    let cipher = ChaCha20Poly1305::new(key);
    let nonce_bytes = base64::decode(&wrapper.nonce)?;
    let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes);
    let mut in_out = base64::decode(wrapper.cipher_text.as_str())?;
    let tag_bytes = base64::decode(&wrapper.tag)?;
    let tag = chacha20poly1305::Tag::from_slice(&tag_bytes);
    let result = cipher.decrypt_in_place_detached(&nonce, &[], &mut in_out, tag);
    match result {
        Ok(_) => Ok(in_out),
        Err(err) => Err(Error::Decrypt(err.to_string())),
    }
}

/// Wraps the plaintext using the AES-GCM algorithm with the `jwt` to generate the key, and returns
/// an encoded string.
fn wrap_aes_gcm(jwt: &[u8], plaintext: &[u8]) -> Result<String, Error> {
    // derive the key from the JWT
    let derived = generate_key(&jwt, None, None)?;
    let key = aes_gcm::Key::from_slice(&derived);

    // generate a new Nonce
    let mut rand_bytes = [0u8; NONCE_LEN];
    let mut rng = rand_core::OsRng;
    rng.fill_bytes(&mut rand_bytes);
    let nonce = aes_gcm::Nonce::from_slice(&rand_bytes);

    let cipher = Aes256Gcm::new(key);
    let mut in_out = vec![0; plaintext.len()];
    in_out.copy_from_slice(&plaintext);
    let result = cipher.encrypt_in_place_detached(&nonce, &[], &mut in_out);
    match result {
        Ok(tag) => {
            let cipher_str = base64::encode(in_out);
            let nonce_str = base64::encode(nonce);
            let tag_str = base64::encode(tag);
            let encoded = encode(
                Algorithm::AesGcm,
                nonce_str.as_str(),
                cipher_str.as_str(),
                tag_str.as_str(),
            );
            Ok(encoded)
        }
        Err(err) => Err(Error::Encrypt(err.to_string())),
    }
}

/// Unwraps the ciphertext (inside the `SecretWrapper`) using the AES-GCM algorithm with the `jwt`
/// to generate the key, and returns the plaintext on success.
fn unwrap_aes_gcm(jwt: &[u8], wrapper: &SecretWrapper) -> Result<Vec<u8>, Error> {
    let derived = generate_key(jwt, None, None)?;
    let key = aes_gcm::Key::from_slice(&derived);
    let cipher = Aes256Gcm::new(key);
    let nonce_bytes = base64::decode(&wrapper.nonce)?;
    let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);
    let mut in_out = base64::decode(wrapper.cipher_text.as_str())?;
    let tag_bytes = base64::decode(&wrapper.tag)?;
    let tag = aes_gcm::Tag::from_slice(&tag_bytes);
    let result = cipher.decrypt_in_place_detached(&nonce, &[], &mut in_out, tag);
    match result {
        Ok(_) => Ok(in_out),
        Err(err) => Err(Error::Decrypt(err.to_string())),
    }
}

/// Use the JWT to wrap the plaintext string in the specified algorithm
#[allow(dead_code)]
pub fn wrap(algorithm: Algorithm, jwt: &[u8], plaintext: &[u8]) -> Result<String, Error> {
    match algorithm {
        Algorithm::AesGcm => wrap_aes_gcm(jwt, plaintext),
        Algorithm::ChaCha20 => wrap_chacha20_poly1305(jwt, plaintext),
        _ => Err(Error::UnsupportedAlgorithm(format!("{}", algorithm))),
    }
}

/// Uses the JWT to unwrap the encrypted string
#[allow(dead_code)]
pub fn unwrap(jwt: &[u8], encoded: &str) -> Result<Vec<u8>, Error> {
    let wrapper = decode(encoded)?;
    match wrapper.algorithm {
        Algorithm::AesGcm => unwrap_aes_gcm(jwt, &wrapper),
        Algorithm::ChaCha20 => unwrap_chacha20_poly1305(jwt, &wrapper),
        _ => Err(Error::UnsupportedAlgorithm(format!(
            "{}",
            wrapper.algorithm
        ))),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    const TEST_JWT: &str = concat!(
        "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJ0ZXN0ZXJfYnJvL3VzZXJpbmZvIiwi",
        "ZW1haWwiOiJ0ZXN0ZXJAdGVzdG1lLmJybyIsImVtYWlsX3ZlcmlmaWVkIjp0cnVlLCJleHAiOjE2M",
        "TkxMjYyNjIsImZhbWlseV9uYW1lIjoiRXIiLCJnaXZlbl9uYW1lIjoiVGVzdCIsImh0dHBzOi8vY2",
        "xvdWR0cnV0aC5jb20vb3JnaWQiOiJteV9vcmciLCJodHRwczovL2Nsb3VkdHJ1dGguY29tL3VzZXJ",
        "pZCI6Im15X3VzZXIiLCJpYXQiOjE2MTkwMzk4NjIsImlzcyI6Imh0dHBzOi8vbG9jYWxob3N0Iiwi",
        "anRpIjoiaWsySDVIVFhPQzNJRnFhNFppZHVmUSIsImxvY2FsZSI6ImVuIiwibmFtZSI6IlRlc3QgR",
        "XIiLCJuYmYiOjE2MTkwMzk4NjIsIm5pY2tuYW1lIjoidGVzdGVyIiwic2NvcGUiOiJvcGVuaWQgcH",
        "JvZmlsZSBlbWFpbCIsInN1YiI6InRlc3RlcmJyb3wxMDY3ODcxNjg2NTQ1MTI2OTQ5NTcifQ.puCu",
        "W4V24yX1rl7EPhyGitUYQSIKpiRApC90BA3rXnYTM_QWCN0c4Z6UEmYam0qkw8zMxDm7HzxnW_5xZ",
        "K8tVuhZ2N8hWfOhnPzS54GYaFCT2K39jbqSLf8OvNdGtyr2TayiL2MVXS02Wkyt3HWZnxSJ0NH_sB",
        "jjFboQFGSF6IKIMRxSr9yuj3g9YDZ6riCFJ8mz8kxEDKUa7XPPfLYCJflcTxnfbINVEWU3foDExse",
        "cL9Vdf_wY_Swv7hOltmRczguEkna0v00vj0p7bGHqhX2uKpuavRGsdgNs1A0CgvaAKnt49NFm9iyt",
        "Cjq3vm0Rmcu_Eg84ZY01LA828WZIcQ",
    );
    const TEST_SECRET: &str = "shhh - I'm hunting rabbits!";

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
        let salt: &[u8] = &[];

        assert_eq!(expected, generate_key(&icm, Some(salt), Some(128)).unwrap());

        // another test here to make sure we have a default key length
        let result = generate_key(&icm, Some(salt), None).unwrap();
        assert_eq!(result.len(), KEY_LEN);
        assert_eq!(&expected[0..KEY_LEN], result);

        // verify salt is generated
        let result = generate_key(&icm, None, Some(128)).unwrap();
        assert_eq!(result.len(), 128);
        assert_ne!(expected, result);
    }

    #[test]
    fn key_derivation_error() {
        let icm = hex::decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap();
        let err_msg = "invalid number of blocks, too large output".to_string();
        let result = generate_key(&icm, None, Some(65535)).unwrap_err();
        assert_eq!(result, Error::KeyDerivation(err_msg));
    }

    #[test]
    fn wrap_unsupported() {
        let jwt = b"fake-jwt-key";
        let plaintext = "this is sample plaintext";
        let result = wrap(Algorithm::Unknown, jwt, plaintext.as_bytes()).unwrap_err();
        assert_eq!(result, Error::UnsupportedAlgorithm(UNKNOWN_STR.to_string()));
    }

    #[test]
    fn unwrap_unsupported() {
        let jwt = b"fake-jwt-key";
        let nonce = "sample_nonce";
        let ciphertext = "cipher_text_goes_here";
        let tag = "tag_value_goes_here";
        let algo_str = UNKNOWN_STR.to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = unwrap(jwt, encoded_string.as_str()).unwrap_err();
        assert_eq!(result, Error::UnsupportedAlgorithm(algo_str));
    }

    #[test]
    fn chacha20_known_answer() {
        // this was generated by vaas server code, so make sure we can decrypt it
        let wrapped = "smaash:chacha20:082zVRh/NfzCSMyb:RFYIyf1J/yTRdKZKwrG35o8BMV6OqeXUjoHRqbDxEVzRZwaO:8kzWn7kXU6aPw1y5Q4hoHw==";
        let unwrapped = unwrap(TEST_JWT.as_bytes(), wrapped).unwrap();
        let decoded = base64::decode(&unwrapped).unwrap();
        assert_eq!(TEST_SECRET.as_bytes(), decoded);
    }

    #[test]
    fn chacha20_wrap_and_unwrap() {
        let wrapped = wrap(
            Algorithm::ChaCha20,
            TEST_JWT.as_bytes(),
            TEST_SECRET.as_bytes(),
        )
        .unwrap();
        assert!(wrapped.contains(CHA_CHA_20_STR));
        let unwrapped = unwrap(TEST_JWT.as_bytes(), wrapped.as_str()).unwrap();
        let result = std::str::from_utf8(&unwrapped).unwrap();
        assert_eq!(TEST_SECRET, result);
    }

    #[test]
    fn aes_gcm_known_answer() {
        // this was generated by vaas server code, so make sure we can decrypt it
        let wrapped = "smaash:aes_gcm:+CFANT6cgczDMH1X:3H5W3RZ4XZUt5Jkm81Z50NmC4SvIxKLFRtEx2yvMiKd/OihU:4CrkcHCcaW8nOSy60RZsCw==";
        let unwrapped = unwrap(TEST_JWT.as_bytes(), wrapped).unwrap();
        let decoded = base64::decode(&unwrapped).unwrap();
        assert_eq!(TEST_SECRET.as_bytes(), decoded);
    }

    #[test]
    fn aes_gcm_wrap_and_unwrap() {
        let wrapped = wrap(
            Algorithm::AesGcm,
            TEST_JWT.as_bytes(),
            TEST_SECRET.as_bytes(),
        )
        .unwrap();
        assert!(wrapped.contains(AES_GCM_STR));
        let unwrapped = unwrap(TEST_JWT.as_bytes(), wrapped.as_str()).unwrap();
        let result = std::str::from_utf8(&unwrapped).unwrap();
        assert_eq!(TEST_SECRET, result);
    }
}

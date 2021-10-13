use crate::database::{CryptoAlgorithm, CryptoError};
use aes_gcm::Aes256Gcm;
use chacha20poly1305::aead::{AeadInPlace, NewAead};
use chacha20poly1305::ChaCha20Poly1305;
use hkdf::Hkdf;
use rand_core::RngCore;
use sha2::{Digest, Sha512};
use std::str::FromStr;

pub const ENCRYPTION_PREFIX: &str = "smaash";

pub const ENCODED_PART_COUNT: usize = 5;
pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 12;

#[derive(PartialEq, Eq, Debug)]
struct SecretWrapper {
    pub algorithm: CryptoAlgorithm,
    pub nonce: String,
    pub cipher_text: String,
    pub tag: String,
}

/// Takes the encrypted data and related information, and encodes it (into String) for transmission.
fn encode(algorithm: CryptoAlgorithm, nonce: &str, ciphertext: &str, tag: &str) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        ENCRYPTION_PREFIX, algorithm, nonce, ciphertext, tag
    )
}

/// Decodes the "encoded blob" into components.
fn decode(encoded: &str) -> Result<SecretWrapper, CryptoError> {
    let parts = encoded.split(':').collect::<Vec<&str>>();
    let prefix = parts.get(0).unwrap();

    if parts.len() != ENCODED_PART_COUNT {
        Err(CryptoError::InvalidEncoding(parts.len()))
    } else if *prefix != ENCRYPTION_PREFIX {
        Err(CryptoError::InvalidPrefix(prefix.to_string()))
    } else {
        let algo_str = parts.get(1).unwrap();
        if let Ok(algorithm) = CryptoAlgorithm::from_str(algo_str) {
            let decomposed = SecretWrapper {
                algorithm,
                nonce: parts.get(2).unwrap().to_string(),
                cipher_text: parts.get(3).unwrap().to_string(),
                tag: parts.get(4).unwrap().to_string(),
            };
            Ok(decomposed)
        } else {
            Err(CryptoError::InvalidAlgorithm(algo_str.to_string()))
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
) -> Result<Vec<u8>, CryptoError> {
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
    let kdf = Hkdf::<Sha512>::new(Some(binary_salt), source);
    let mut key = vec![0; key_len.unwrap_or(KEY_LEN)];
    let result = kdf.expand(&[], &mut key);
    match result {
        Ok(_) => Ok(key),
        Err(e) => Err(CryptoError::KeyDerivation(e.to_string())),
    }
}

/// Wraps the plaintext using the ChaCha20 algorithm with the `token` to generate the key, and returns
/// an encoded string.
fn wrap_chacha20_poly1305(token: &[u8], plaintext: &[u8]) -> Result<String, CryptoError> {
    // derive the key from the JWT
    let derived = generate_key(token, None, None)?;
    let key = chacha20poly1305::Key::from_slice(&derived);

    // generate a new Nonce
    let mut rand_bytes = [0u8; NONCE_LEN];
    let mut rng = rand_core::OsRng;
    rng.fill_bytes(&mut rand_bytes);
    let nonce = chacha20poly1305::Nonce::from_slice(&rand_bytes);

    let cipher = ChaCha20Poly1305::new(key);
    let mut in_out = vec![0; plaintext.len()];
    in_out.copy_from_slice(plaintext);
    let result = cipher.encrypt_in_place_detached(nonce, &[], &mut in_out);
    match result {
        Ok(tag) => {
            let cipher_str = base64::encode(in_out);
            let nonce_str = base64::encode(nonce);
            let tag_str = base64::encode(tag);
            let encoded = encode(
                CryptoAlgorithm::ChaCha20,
                nonce_str.as_str(),
                cipher_str.as_str(),
                tag_str.as_str(),
            );
            Ok(encoded)
        }
        Err(err) => Err(CryptoError::Encrypt(err.to_string())),
    }
}

/// Unwraps the ciphertext (inside the `SecretWrapper`) using the ChaCha20 algorithm with the `token`
/// to generate the key, and returns the plaintext on success.
fn unwrap_chacha20_poly1305(token: &[u8], wrapper: &SecretWrapper) -> Result<Vec<u8>, CryptoError> {
    let derived = generate_key(token, None, None)?;
    let key = chacha20poly1305::Key::from_slice(&derived);
    let cipher = ChaCha20Poly1305::new(key);
    let nonce_bytes = base64::decode(&wrapper.nonce)?;
    let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes);
    let mut in_out = base64::decode(wrapper.cipher_text.as_str())?;
    let tag_bytes = base64::decode(&wrapper.tag)?;
    let tag = chacha20poly1305::Tag::from_slice(&tag_bytes);
    let result = cipher.decrypt_in_place_detached(nonce, &[], &mut in_out, tag);
    match result {
        Ok(_) => Ok(in_out),
        Err(err) => Err(CryptoError::Decrypt(err.to_string())),
    }
}

/// Wraps the plaintext using the AES-GCM algorithm with the `token` to generate the key, and returns
/// an encoded string.
fn wrap_aes_gcm(token: &[u8], plaintext: &[u8]) -> Result<String, CryptoError> {
    // derive the key from the JWT
    let derived = generate_key(token, None, None)?;
    let key = aes_gcm::Key::from_slice(&derived);

    // generate a new Nonce
    let mut rand_bytes = [0u8; NONCE_LEN];
    let mut rng = rand_core::OsRng;
    rng.fill_bytes(&mut rand_bytes);
    let nonce = aes_gcm::Nonce::from_slice(&rand_bytes);

    let cipher = Aes256Gcm::new(key);
    let mut in_out = vec![0; plaintext.len()];
    in_out.copy_from_slice(plaintext);
    let result = cipher.encrypt_in_place_detached(nonce, &[], &mut in_out);
    match result {
        Ok(tag) => {
            let cipher_str = base64::encode(in_out);
            let nonce_str = base64::encode(nonce);
            let tag_str = base64::encode(tag);
            let encoded = encode(
                CryptoAlgorithm::AesGcm,
                nonce_str.as_str(),
                cipher_str.as_str(),
                tag_str.as_str(),
            );
            Ok(encoded)
        }
        Err(err) => Err(CryptoError::Encrypt(err.to_string())),
    }
}

/// Unwraps the ciphertext (inside the `SecretWrapper`) using the AES-GCM algorithm with the `token`
/// to generate the key, and returns the plaintext on success.
fn unwrap_aes_gcm(token: &[u8], wrapper: &SecretWrapper) -> Result<Vec<u8>, CryptoError> {
    let derived = generate_key(token, None, None)?;
    let key = aes_gcm::Key::from_slice(&derived);
    let cipher = Aes256Gcm::new(key);
    let nonce_bytes = base64::decode(&wrapper.nonce)?;
    let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);
    let mut in_out = base64::decode(wrapper.cipher_text.as_str())?;
    let tag_bytes = base64::decode(&wrapper.tag)?;
    let tag = aes_gcm::Tag::from_slice(&tag_bytes);
    let result = cipher.decrypt_in_place_detached(nonce, &[], &mut in_out, tag);
    match result {
        Ok(_) => Ok(in_out),
        Err(err) => Err(CryptoError::Decrypt(err.to_string())),
    }
}

/// Use the JWT to wrap the plaintext string in the specified algorithm
#[allow(dead_code)]
pub fn secret_wrap(
    algorithm: CryptoAlgorithm,
    token: &[u8],
    plaintext: &[u8],
) -> Result<String, CryptoError> {
    match algorithm {
        CryptoAlgorithm::AesGcm => wrap_aes_gcm(token, plaintext),
        CryptoAlgorithm::ChaCha20 => wrap_chacha20_poly1305(token, plaintext),
        _ => Err(CryptoError::UnsupportedAlgorithm(format!("{}", algorithm))),
    }
}

/// Uses the JWT to unwrap the encrypted string
#[allow(dead_code)]
pub fn secret_unwrap(token: &[u8], encoded: &str) -> Result<Vec<u8>, CryptoError> {
    let wrapper = decode(encoded)?;
    match wrapper.algorithm {
        CryptoAlgorithm::AesGcm => unwrap_aes_gcm(token, &wrapper),
        CryptoAlgorithm::ChaCha20 => unwrap_chacha20_poly1305(token, &wrapper),
        _ => Err(CryptoError::UnsupportedAlgorithm(format!(
            "{}",
            wrapper.algorithm
        ))),
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
    fn encode_test_ok() {
        let nonce = "sample_nonce";
        let ciphertext = "cipher_text_goes_here";
        let tag = "tag_value_goes_here";
        let algo_str = CryptoAlgorithm::AesGcm.to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = encode(CryptoAlgorithm::AesGcm, &nonce, &ciphertext, tag);
        assert_eq!(result, encoded_string);

        // repeat with different crypto algorithm
        let algo_str = CryptoAlgorithm::ChaCha20.to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = encode(CryptoAlgorithm::ChaCha20, &nonce, &ciphertext, &tag);
        assert_eq!(result, encoded_string);
    }

    #[test]
    fn decode_test_ok() {
        let nonce = "sample_nonce".to_string();
        let ciphertext = "cipher_text_goes_here".to_string();
        let tag = "tag_value_goes_here".to_string();
        let algo_str = CryptoAlgorithm::AesGcm.to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = decode(encoded_string.as_str()).unwrap();
        assert_eq!(
            result,
            SecretWrapper {
                algorithm: CryptoAlgorithm::AesGcm,
                nonce: nonce.clone(),
                cipher_text: ciphertext.clone(),
                tag: tag.clone(),
            }
        );

        let algo_str = CryptoAlgorithm::ChaCha20.to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = decode(encoded_string.as_str()).unwrap();
        assert_eq!(
            result,
            SecretWrapper {
                algorithm: CryptoAlgorithm::ChaCha20,
                nonce: nonce.clone(),
                cipher_text: ciphertext.clone(),
                tag: tag.clone(),
            }
        );

        let algo_str = CryptoAlgorithm::Unknown.to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = decode(encoded_string.as_str()).unwrap();
        assert_eq!(
            result,
            SecretWrapper {
                algorithm: CryptoAlgorithm::Unknown,
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
        assert_eq!(result, CryptoError::InvalidAlgorithm(algo_str));

        let algo_str = "chacha21".to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = decode(encoded_string.as_str()).unwrap_err();
        assert_eq!(result, CryptoError::InvalidAlgorithm(algo_str));

        let algo_str = "".to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = decode(encoded_string.as_str()).unwrap_err();
        assert_eq!(result, CryptoError::InvalidAlgorithm(algo_str));
    }

    #[test]
    fn decode_test_encoding_failures() {
        let nonce = "sample_nonce".to_string();
        let ciphertext = "cipher_text_goes_here".to_string();
        let tag = "tag_value_goes_here".to_string();
        let algo_str = "aes_gcm".to_string();
        let encoded_string = format!("smash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = decode(encoded_string.as_str()).unwrap_err();
        assert_eq!(result, CryptoError::InvalidPrefix("smash".to_string()));

        // too few parts
        let encoded_string = format!("smaash:{}:{}:{}", algo_str, nonce, ciphertext);
        let result = decode(encoded_string.as_str()).unwrap_err();
        assert_eq!(result, CryptoError::InvalidEncoding(4));

        // too many parts
        let encoded_string = format!(
            "smaash:{}:{}:{}:{}:more_stuff",
            algo_str, nonce, ciphertext, tag
        );
        let result = decode(encoded_string.as_str()).unwrap_err();
        assert_eq!(result, CryptoError::InvalidEncoding(6));
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
        assert_eq!(result, CryptoError::KeyDerivation(err_msg));
    }

    #[test]
    fn wrap_unsupported() {
        let token = b"fake-token-key";
        let plaintext = "this is sample plaintext";
        let result =
            secret_wrap(CryptoAlgorithm::Unknown, token, plaintext.as_bytes()).unwrap_err();
        assert_eq!(
            result,
            CryptoError::UnsupportedAlgorithm(CryptoAlgorithm::Unknown.to_string())
        );
    }

    #[test]
    fn unwrap_unsupported() {
        let token = b"fake-token-key";
        let nonce = "sample_nonce";
        let ciphertext = "cipher_text_goes_here";
        let tag = "tag_value_goes_here";
        let algo_str = CryptoAlgorithm::Unknown.to_string();
        let encoded_string = format!("smaash:{}:{}:{}:{}", algo_str, nonce, ciphertext, tag);
        let result = secret_unwrap(token, encoded_string.as_str()).unwrap_err();
        assert_eq!(result, CryptoError::UnsupportedAlgorithm(algo_str));
    }

    #[test]
    fn chacha20_known_answer() {
        // this was generated by vaas server code, so make sure we can decrypt it
        let wrapped = "smaash:chacha20:082zVRh/NfzCSMyb:RFYIyf1J/yTRdKZKwrG35o8BMV6OqeXUjoHRqbDxEVzRZwaO:8kzWn7kXU6aPw1y5Q4hoHw==";
        let unwrapped = secret_unwrap(TEST_JWT.as_bytes(), wrapped).unwrap();
        let decoded = base64::decode(&unwrapped).unwrap();
        assert_eq!(TEST_SECRET.as_bytes(), decoded);
    }

    #[test]
    fn chacha20_wrap_and_unwrap() {
        let wrapped = secret_wrap(
            CryptoAlgorithm::ChaCha20,
            TEST_JWT.as_bytes(),
            TEST_SECRET.as_bytes(),
        )
        .unwrap();
        let algo_name = CryptoAlgorithm::ChaCha20.to_string();
        assert!(wrapped.contains(&algo_name));
        let unwrapped = secret_unwrap(TEST_JWT.as_bytes(), wrapped.as_str()).unwrap();
        let result = std::str::from_utf8(&unwrapped).unwrap();
        assert_eq!(TEST_SECRET, result);
    }

    #[test]
    fn aes_gcm_known_answer() {
        // this was generated by vaas server code, so make sure we can decrypt it
        let wrapped = "smaash:aes_gcm:+CFANT6cgczDMH1X:3H5W3RZ4XZUt5Jkm81Z50NmC4SvIxKLFRtEx2yvMiKd/OihU:4CrkcHCcaW8nOSy60RZsCw==";
        let unwrapped = secret_unwrap(TEST_JWT.as_bytes(), wrapped).unwrap();
        let decoded = base64::decode(&unwrapped).unwrap();
        assert_eq!(TEST_SECRET.as_bytes(), decoded);
    }

    #[test]
    fn aes_gcm_wrap_and_unwrap() {
        let wrapped = secret_wrap(
            CryptoAlgorithm::AesGcm,
            TEST_JWT.as_bytes(),
            TEST_SECRET.as_bytes(),
        )
        .unwrap();
        let algo_name = CryptoAlgorithm::AesGcm.to_string();
        assert!(wrapped.contains(&algo_name));
        let unwrapped = secret_unwrap(TEST_JWT.as_bytes(), wrapped.as_str()).unwrap();
        let result = std::str::from_utf8(&unwrapped).unwrap();
        assert_eq!(TEST_SECRET, result);
    }
}

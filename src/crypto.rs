use base64::{self, DecodeError as Base64Error};
use hkdf::Hkdf;
use sha2::Sha256;
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
/// Takes the base64 encoded master key (as source) that is used to derive the output key.
fn generate_key(
    source: &str,
    salt: Option<&[u8]>,
    key_len: Option<usize>,
) -> Result<Vec<u8>, Error> {
    let decoded_src = base64::decode(source)?;
    let kdf = Hkdf::<Sha256>::new(salt, &decoded_src);
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
        let icm = hex::decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap();
        let icm_str = base64::encode(icm);
        let expected = hex::decode(concat!(
            "8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d9d201395faa4b61a",
            "96c8b2fb61057244b36c6ddd287f634795e7d80d5fe26bfc36def6dc129c29271a0eb7ab14bd2ca8",
            "8259f8a3a92ac2ec0e3e4fa046a4b90b137ce44a2105152c2db1480a0ac2e999db47e00083385609",
            "3a641da082a94429eccf5143b1780234baacfa9d89b850473f2c678f7caa96b00efdace2d71fd424",
            "12465308f2374960b6c3c35d3eaaea0b8f7e595fa9cc5db25a91bc0120bad6223b25e378f514686e",
            "edfc9f102e045868e64ab02f1c021868b4e6686e295a10a94524e8730d79b4d3f890c387763a19a2",
            "48577e9aad8e23ba2bacfc9230fb144a7730d3ceaf799d1de1032951dc1fddea17f96981fbf6b05e",
            "4878df782b497f60c3eb4cff0c249d8c1bbeabca6e66c71cc79314f2bfc758dd41c9872cbb461e17",
            "cff0002a9fdee546953f98dde7d4aa828a68f4ccd950d371d1eb3839219bfdb9ef911d3602d78bd2",
            "f6a970a6c437763c81821e131325fa77fc8963a021b87caa8e35ad2a37f2237a2143c1966817e4da",
            "9912a2013cd7239c5fad2e4599f9ce20bcd8cd0ff1516f9f2496ecce94ccd82cde5c9ef120bbaf64",
            "0d6292d6916d40d99d9d4657dd2fdf2d4e5a5bbb27446ed58a3d637d0811c555273b0fd3699161ac",
            "b58205fe6cfd45245249a7ecfc4c3d0668de270e6962d1d19fffa4b8df099cd4022cb0df18ffcfff",
            "f5b046f0fc082ed16f99b416b7bbd4982ad0afc8b1dc332a8058729065538dbe9422b795a887af9c",
            "5d50ee85a60871be14c7f2d1111f197378d99065fb89a0f57b7342792798963f00910e5fad47a644",
            "77f41c07ac6058e01932502eb6faa88a6cc21e115b8e3ddfae8fdcaab60d13d808f3206b4e9da8bd",
            "4ae1108b2a01d256a02c9131ea0f6203c8c6e55ec7ae16bb19cf3239490085713679e7c304ce254e",
            "2897c0fd3bc97263a562fe161dcf6d21e841eb2266aa1cfaaaf6fc094111ad4b2e4d8e05b50854ae",
            "5de83d81842c689a55b1be7d575ac50e81d7708c262c1f70452884c7714abef03b88b85a41e895a0",
            "e7529b8d631e5e77583175c80e86e45802763eaba0471d11fc885b34fa4b5309a9fe49a5215d4aa2",
            "1041c53a30a1e97250f6445ce537bb3efb1fa17f141db69c7d97ab48cb34c33bef0ded5d4c320fe5",
            "54a0faea353a5579cb08f072565bbd49d167186f39a298a553f320bb89eaee54151b08deef49b7b6",
            "30af62b4d7be1f4965a53c67e7d3e34a6d8263ee86f44dfbe019cbe8e3bd4ed0cda06985127ff8d1",
            "794e6321891a950f329aca2b36b16f8a2bb910b1206a5c238ef079df12ecb0f0f7e3e4f8a64bfd23",
            "b57e9d286a1c8d2e9290d9a4f1d20ec100aac7dc90783cb2ecfd69d71a91dcc3913494ebf7a7a00d",
            "1051102d7f268e761855b985c2599350f15ee0d4093244113185bc7031d2431ccd9391fcd58a85e0",
            "68458b644ed265b3f103852a2d7bbf0d2c1d7c02e30ff1ec552f09bc60e36393391cec0592600952",
            "0af12d96387cc55b9553e79da8b2eb9303ecf15bb289530c3d65c4cc5a68f8ece60a37522fe3d0e6",
            "ba4ddfb560a45717456cf91c5dc5b8117da68bc49968ec1e35852bbc54e554fb839b35f6c3b5c095",
            "30855d8691fc0f126f67346f949bd813a6db44c513d1e61b8c8789eb9e823d1a38862dca1c5331da"
        ))
        .unwrap();

        assert_eq!(
            expected,
            generate_key(icm_str.as_str(), None, Some(1200)).unwrap()
        );
    }

    #[test]
    fn key_derivation_error() {
        let icm_str = "ThisContains!non-base65#characters";
        // NOTE: following error message may change with new base64 versions, but needed to match
        //       the Error text.
        let err_msg = "Invalid byte 33, offset 12.".to_string();
        let result = generate_key(icm_str, None, None).unwrap_err();
        assert_eq!(result, Error::Base64(err_msg));

        let icm = hex::decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap();
        let icm_str = base64::encode(icm);
        let err_msg = "invalid number of blocks, too large output".to_string();
        let result = generate_key(icm_str.as_str(), None, Some(65535)).unwrap_err();
        assert_eq!(result, Error::KeyDerivation(err_msg));
    }
}

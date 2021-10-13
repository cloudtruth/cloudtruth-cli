use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

const AES_GCM_STR: &str = "aes_gcm";
const CHA_CHA_20_STR: &str = "chacha20";
const UNKNOWN_STR: &str = "unknown";

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum CryptoAlgorithm {
    AesGcm = 1,
    ChaCha20 = 2,
    Unknown = 99,
}

/// Converts enum into common encoded string
impl Display for CryptoAlgorithm {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CryptoAlgorithm::AesGcm => write!(f, "{}", AES_GCM_STR),
            CryptoAlgorithm::ChaCha20 => write!(f, "{}", CHA_CHA_20_STR),
            CryptoAlgorithm::Unknown => write!(f, "{}", UNKNOWN_STR),
        }
    }
}

/// Converts string into enum
impl FromStr for CryptoAlgorithm {
    type Err = ();

    fn from_str(input: &str) -> Result<CryptoAlgorithm, Self::Err> {
        match input.to_lowercase().as_str() {
            AES_GCM_STR => Ok(CryptoAlgorithm::AesGcm),
            CHA_CHA_20_STR => Ok(CryptoAlgorithm::ChaCha20),
            UNKNOWN_STR => Ok(CryptoAlgorithm::Unknown),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn algorithm_to_string() {
        let mut map: HashMap<CryptoAlgorithm, String> = HashMap::new();
        map.insert(CryptoAlgorithm::AesGcm, AES_GCM_STR.to_string());
        map.insert(CryptoAlgorithm::ChaCha20, CHA_CHA_20_STR.to_string());
        map.insert(CryptoAlgorithm::Unknown, UNKNOWN_STR.to_string());
        for (iv, sv) in map {
            assert_eq!(format!("{}", iv).to_string(), sv);
        }
    }

    #[test]
    fn algorithm_from_string() {
        // Tests case insensitivity, as well as all possible versions
        let mut map: HashMap<String, Result<CryptoAlgorithm, _>> = HashMap::new();
        map.insert(AES_GCM_STR.to_string(), Ok(CryptoAlgorithm::AesGcm));
        map.insert(CHA_CHA_20_STR.to_string(), Ok(CryptoAlgorithm::ChaCha20));
        map.insert(UNKNOWN_STR.to_string(), Ok(CryptoAlgorithm::Unknown));
        map.insert("AES_GCM".to_string(), Ok(CryptoAlgorithm::AesGcm)); // capitals
        map.insert("ChaCha20".to_string(), Ok(CryptoAlgorithm::ChaCha20)); // capitals
        map.insert("".to_string(), Err(())); // blank
        map.insert("aes-gcm".to_string(), Err(())); // wrong separator
        for (sv, iv) in map {
            assert_eq!(CryptoAlgorithm::from_str(sv.as_str()), iv);
        }
    }
}

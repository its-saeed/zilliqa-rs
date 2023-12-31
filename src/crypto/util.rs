use std::ops::BitAnd;

use secp256k1::{PublicKey, Secp256k1, SecretKey};
use sha2::Digest;

use crate::{
    util::validation::{is_address, is_private_key},
    Error,
};

pub fn normalize_private_key(private_key: &str) -> Result<String, Error> {
    if !is_private_key(private_key) {
        return Err(Error::IncorrectPrivateKey);
    }

    // TODO: Consider performance here
    let normalized = private_key.to_lowercase().replace("0x", "");

    Ok(normalized)
}

pub fn get_pub_key_from_private_key(private_key: &str) -> Result<String, Error> {
    let private_key = normalize_private_key(private_key)?;
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&hex::decode(private_key)?)?;
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    Ok(public_key.to_string())
}

pub fn get_address_from_public_key(public_key: &str) -> Result<String, Error> {
    let normalized = public_key.to_lowercase().replace("0x", "");

    let mut hasher = sha2::Sha256::new();
    hasher.update(hex::decode(normalized)?);
    to_checksum_address(&hex::encode(hasher.finalize())[24..])
}

pub fn to_checksum_address(address: &str) -> Result<String, Error> {
    let address = address.replace("0x", "");
    if !is_address(&address) {
        return Err(Error::InvalidAddress(address.to_string()));
    }

    let mut hasher = sha2::Sha256::new();
    hasher.update(hex::decode(&address)?);
    let v = primitive_types::U256::from_big_endian(&hasher.finalize());
    let ret = address
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_ascii_digit() {
                c
            } else {
                let cond = v
                    .bitand(primitive_types::U256::from(2).pow(primitive_types::U256::from(255 - 6 * i)))
                    .ge(&primitive_types::U256::one());
                if cond {
                    c.to_ascii_uppercase()
                } else {
                    c.to_ascii_lowercase()
                }
            }
        })
        .collect::<String>();

    Ok(format!("0x{}", ret))
}

pub fn is_valid_checksum_address(address: &str) -> Result<bool, Error> {
    Ok(to_checksum_address(address)? == address)
}

pub fn generate_private_key() -> String {
    let secret_key = SecretKey::new(&mut rand::thread_rng());
    format!("{}", secret_key.display_secret())
}

#[cfg(test)]
mod tests {
    use crate::{
        crypto::util::{
            get_address_from_public_key, get_pub_key_from_private_key, is_valid_checksum_address, to_checksum_address,
        },
        util::validation::is_private_key,
    };

    use super::generate_private_key;

    #[test]
    fn get_pub_key_from_private_key_should_return_correct_public_key_for_a_valid_private_key() {
        let private_key = "d96e9eb5b782a80ea153c937fa83e5948485fbfc8b7e7c069d7b914dbc350aba";
        let expected_pub_key = "03bfad0f0b53cff5213b5947f3ddd66acee8906aba3610c111915aecc84092e052";

        assert_eq!(get_pub_key_from_private_key(private_key).unwrap(), expected_pub_key);
    }

    #[test]
    fn to_checksum_address_should_return_correct_value_for_valid_input() {
        let address = "11223344556677889900aabbccddeeff11223344";
        let checksum = "0x11223344556677889900AabbccdDeefF11223344";

        assert_eq!(checksum, to_checksum_address(address).unwrap())
    }

    #[test]
    fn is_valid_checksum_address_should_return_true_if_address_is_correctly_checksumed() {
        let checksum = "0x11223344556677889900AabbccdDeefF11223344";

        assert!(is_valid_checksum_address(checksum).unwrap())
    }

    #[test]
    fn is_valid_checksum_address_should_return_false_if_address_is_not_correctly_checksumed() {
        let checksum = "0x11223344556677889900AabbccdDEEfF11223344";

        assert!(!is_valid_checksum_address(checksum).unwrap())
    }

    #[test]
    fn get_address_from_public_key_should_return_correct_address() {
        let pub_key = "03bfad0f0b53cff5213b5947f3ddd66acee8906aba3610c111915aecc84092e052";
        let expected_address = "0x381f4008505e940AD7681EC3468a719060caF796";

        assert_eq!(get_address_from_public_key(pub_key).unwrap(), expected_address)
    }

    #[test]
    fn generate_private_key_should_generate_a_valid_private_key() {
        let private_key = generate_private_key();
        assert!(is_private_key(&private_key))
    }
}

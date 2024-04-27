use std::{collections::HashMap, hash::Hash, str::FromStr};

use crypto::{digest::Digest, ripemd160};
use log::{error, info};
use serde::{Serialize, Deserialize};
use sled::Db;
use secp256k1; 
use secp256k1::schnorr::Signature;
use sha2::{Sha256, Digest as Sha256Digest};
use bs58; 

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl Wallet {
    fn new() -> Self {
        let (private_key, public_key) = Wallet::generate_keypair();
        Wallet {
            private_key,
            public_key,
        }
    }

    pub fn get_pub_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }

    fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
        let secp = secp256k1::Secp256k1::new();
        let random_bytes = rand::random::<[u8; 32]>();
        let private_key = secp256k1::SecretKey::from_slice(&random_bytes).unwrap();
        let public_key = secp256k1::PublicKey::from_secret_key(&secp, &private_key);
        (private_key.secret_bytes().to_vec(), public_key.serialize().to_vec())
    }

    fn sign(&self, message: &[u8]) -> Vec<u8> {
        let secp = secp256k1::Secp256k1::new();
        let message = secp256k1::Message::from_digest_slice(message).unwrap();
        let private_key = secp256k1::SecretKey::from_slice(&self.private_key).unwrap();
        let public_key = secp256k1::PublicKey::from_slice(&self.public_key).unwrap();
        let keypair = secp256k1::Keypair::from_secret_key(&secp, &private_key);
        let signature = secp.sign_schnorr_no_aux_rand(&message, &keypair);
        signature.serialize().to_vec()
    }

    fn verify(public_key: &[u8], message: &[u8], signature: &str) -> bool {
        let secp = secp256k1::Secp256k1::new();
        let message = secp256k1::Message::from_digest_slice(message).unwrap();
        let signature = secp256k1::schnorr::Signature::from_str(signature).unwrap();
        let xonly_pubkey = secp256k1::XOnlyPublicKey::from_slice(&public_key).unwrap();

        secp.verify_schnorr(&signature, &message, &xonly_pubkey).is_ok()
    }

    fn get_address_helper(public_key: &[u8]) -> String {
        let public_key = secp256k1::PublicKey::from_slice(public_key).unwrap();
        let public_key = public_key.serialize().to_vec();
        let mut hasher = Sha256::new();
        hasher.update(&public_key);
        let hash = hasher.finalize();
        let mut ripemd = ripemd160::Ripemd160::new();
        ripemd.input(&hash);
        let num_bytes = ripemd.output_bytes();
        let mut out = vec![0; num_bytes];
        ripemd.result(&mut out);
        let mut payload = vec![0x00];
        payload.extend(out);
        hasher = Sha256::new();
        hasher.update(&payload);
        let checksum = hasher.finalize();

        payload.extend(&checksum[..4]);
        bs58::encode(payload).into_string()
    }

    fn get_address(&self) -> String {
        Wallet::get_address_helper(&self.public_key)
    }

    fn verify_address(address: &str) -> bool {
        let decoded = bs58::decode(address).into_vec().unwrap();
        let checksum = &decoded[decoded.len() - 4..];
        let payload = &decoded[..decoded.len() - 4];
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        let new_checksum = hasher.finalize();
        checksum == &new_checksum[..4]
    }

    fn from_address(address: &str) -> Wallet {
        let decoded = bs58::decode(address).into_vec().unwrap();
        let public_key_hash = &decoded[1..decoded.len() - 4];
        let public_key = secp256k1::PublicKey::from_slice(public_key_hash).unwrap();
        Wallet {
            private_key: vec![],
            public_key: public_key.serialize().to_vec(),
        }
    }

    fn from_pub_key(public_key: &[u8]) -> Wallet {
        Wallet {
            private_key: vec![],
            public_key: public_key.to_vec(),
        }
    }

    fn from_pub_key_hash(public_key_hash: &[u8]) -> Wallet {
        let public_key = secp256k1::PublicKey::from_slice(public_key_hash).unwrap();
        Wallet {
            private_key: vec![],
            public_key: public_key.serialize().to_vec(),
        }
    }

    fn to_string(&self) -> String {
        let private_key = bs58::encode(&self.private_key).into_string();
        let public_key = bs58::encode(&self.public_key).into_string();
        format!("Private key: {}\nPublic key: {}", private_key, public_key)
    }
}

pub fn get_pub_key_hash(public_key: &[u8]) -> Vec<u8> {
    let public_key = secp256k1::PublicKey::from_slice(public_key).unwrap();
    let public_key = public_key.serialize().to_vec();
    let mut hasher = Sha256::new();
    hasher.update(&public_key);
    let hash = hasher.finalize();
    let mut ripemd = ripemd160::Ripemd160::new();
    ripemd.input(&hash);
    let num_bytes = ripemd.output_bytes();
    let mut out = vec![0; num_bytes];
    ripemd.result(&mut out);
    out.to_vec()
}

#[derive(Debug)]
pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets {
    pub fn new() -> Self {
        let mut wallets = HashMap::new();

        let db = sled::open("data/wallets").unwrap();

        for wallet in db.iter() {
            let i = wallet.unwrap();
            let addr = String::from_utf8(i.0.to_vec()).unwrap();
            let wallet = Wallet::from_pub_key_hash(&i.1);
            wallets.insert(addr, wallet);
        }

        drop(db);
        Wallets { wallets }
    }

    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        
        let db = sled::open("data/wallets").unwrap();
        db.insert(address.as_bytes(), wallet.public_key).unwrap();
        db.flush().unwrap();
        drop(db);
        info!("Created wallet with address '{}'", address);
        address    
    }

    pub fn get_wallet(&self, address: &str) -> Option<Wallet> {
        let db = sled::open("data/wallets").unwrap();

        let wallet = db.get(address.as_bytes()).unwrap();
        let wallet = match wallet {
            Some(wallet) => Some(Wallet::from_pub_key_hash(&wallet)),
            None => None,
        };

        drop(db);
        wallet.clone()
    }

    pub fn get_wallets(&self) -> HashMap<String, Wallet> {
        let mut wallets = HashMap::new();

        let db = sled::open("data/wallets").unwrap();

        for wallet in db.iter() {
            let i = wallet.unwrap();
            let addr = String::from_utf8(i.0.to_vec()).unwrap();
            let wallet = Wallet::from_pub_key_hash(&i.1);
            wallets.insert(addr, wallet);
        }

        drop(db);
        wallets
    }

    pub fn get_addresses(&self) -> Vec<String> {
        let mut addresses = Vec::new();

        let db = sled::open("data/wallets").unwrap();

        for wallet in db.iter() {
            let i = wallet.unwrap();
            let addr = String::from_utf8(i.0.to_vec()).unwrap();
            addresses.push(addr);
        }

        drop(db);
        addresses
    }
}

pub fn new_wallets() -> Wallets {
    Wallets::new()
}

pub fn new_wallet() -> Wallet {
    Wallet::new()
}
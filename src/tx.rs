use std::collections::HashMap;
use crate::errors::Result;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    txid: String,
    vout: f32,
    pub signature: Vec<u8>,
    pub pub_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: f32,
    pub pub_key_hash: Vec<u8>,
}

impl TXInput {
    pub fn new(txid: String, vout: f32, signature: Vec<u8>, pub_key: Vec<u8>) -> TXInput {
        TXInput {
            txid,
            vout,
            signature,
            pub_key
        }
    }

    pub fn can_be_unlocked_with(&self, unlocking_data: String) -> bool {
        // Compare the 
        self.pub_key == unlocking_data.as_bytes()
    }

    pub fn get_txid(&self) -> String {
        self.txid.clone()
    }

    pub fn get_vout(&self) -> f32 {
        self.vout
    }

    pub fn get_signature(&self) -> Vec<u8> {
        self.signature.clone()
    }

    pub fn get_pub_key(&self) -> Vec<u8> {
        self.pub_key.clone()
    }

    pub fn set_signature(&mut self, signature: Vec<u8>) {
        self.signature = signature;
    }

    pub fn set_pub_key(&mut self, pub_key: Vec<u8>) {
        self.pub_key = pub_key;
    }
}

impl TXOutput {
    pub fn new(value: f32, pub_key_hash: String) -> TXOutput {
        TXOutput {
            value,
            pub_key_hash: pub_key_hash.as_bytes().to_vec()
        }
    }

    pub fn can_be_unlocked_with(&self, unlocking_data: String) -> bool {
        self.pub_key_hash == unlocking_data.as_bytes()
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }

    pub fn get_script_pub_key(&self) -> String {
        String::from_utf8(self.pub_key_hash.clone()).unwrap()
    }
}

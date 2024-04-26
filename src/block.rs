
use log::info;
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use bincode;
use crate::errors::{Result};
use crate::transaction::{Transaction};


const TARGET_HEXT: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    timestamp: u128,
    transactions: Vec<Transaction>,
    prev_hash: String,
    hash: String,
    height: usize,
    nonce: i32,
}

impl Block {
    pub fn new(transactions: Vec<Transaction>, prev_hash: String, height: usize) -> Result<Block> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        let mut block = Block {
            timestamp: timestamp,
            transactions : transactions,
            prev_hash,
            hash: String::new(),
            height,
            nonce: 0,
        };

        block.verify()?;
        info!("Block created: {:?}", block);

        Ok(block)
    }

    pub fn get_prev_hash(&self) -> String {
        self.prev_hash.clone()
    }

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_nonce(&self) -> i32 {
        self.nonce
    }

    pub fn get_transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    fn verify(&mut self) -> Result<()> {
        log::info!("Mining the block {}", self.nonce);

        while !self.validate()? {
            self.nonce += 1;
        }

        let data = self.prepare_hash()?;

        let mut hasher = Sha256::new();
        hasher.update(&data[..]);
        self.hash = hasher.finalize().to_vec().iter().map(|b| format!("{:02x}", b)).collect::<String>();
        Ok(())
    }

    fn prepare_hash(&self) -> Result<Vec<u8>>{
        let content = (
            self.prev_hash.clone(),
            self.transactions.clone(),
            self.timestamp,
            TARGET_HEXT,
            self.nonce
        );

        let bytes = bincode::serialize(&content)?;
        Ok(bytes)
    } 

    fn validate(&self) -> Result<bool> {
        let data = self.prepare_hash()?;
        let mut hasher = Sha256::new();
        hasher.update(&data[..]);
        let mut vec = vec![];
        vec.resize(TARGET_HEXT, '0' as u8);
        println!("{:?}", vec);
        let hash = hasher.finalize();

        let hash_str = hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();

        Ok(&hash_str[0..TARGET_HEXT] == String::from_utf8(vec)?)
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block() {
        // let mut block = Block::new("Hello".to_string(), "0".to_string(), 0).unwrap();
        // assert_eq!(block.transactions, "Hello");
        // assert_eq!(block.prev_hash, "0");
        // assert_eq!(block.height, 0);
        // dbg!(block);
    }
}

use std::collections::HashMap;
use std::hash::Hash;

use bincode::{self, deserialize};
use crate::block::Block;
use crate::errors::Result;
use crate::transaction::Transaction;
use crate::tx::{TXOutput};
use log::info;

const TARGET_HEXT: usize = 4;
const GENESIS_COINBASE: &str = "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

#[derive(Debug, Clone)]
pub struct Blockchain {
    tip: String,
    db: sled::Db,
}

#[derive(Debug, Clone)]
pub struct BlockchainIterator<'a> {
    curr_hash: String,
    bc: &'a Blockchain,
}

impl Blockchain {
    pub fn new() -> Result<Blockchain> {
        let db = sled::open("data/blocks")?;
        let db_last = db.get("LAST_BLOCK")?.
            expect("No last block found");
        info!("Loading blockchain");
        let tip = String::from_utf8(db_last.to_vec())?;
        Ok(Blockchain {
            tip,
            db,
        })
    }

    pub fn create_blockchain(address: String) -> Result<Blockchain> {
        info!("Creating a new blockchain");

        let db = sled::open("data/blocks")?;
        info!("Creating a new block database");
        let cbtx = Transaction::new_coinbase(
            address, String::from(GENESIS_COINBASE)); 
        let tx = match cbtx {
            Ok(tx) => tx,
            Err(e) => {
                panic!("Error generating genesis block. Cause: {}", e);
            }
        };
        let genesis = Block::new(vec![tx], String::from("GENESIS ARRIVED"), TARGET_HEXT)?;
        db.insert(genesis.get_hash(), bincode::serialize(&genesis)?)?;
        db.insert("LAST_BLOCK", genesis.get_hash().as_bytes())?;

        let bc = Blockchain {
            tip: genesis.get_hash(),
            db,
        };
        bc.db.flush()?;

        Ok(bc)
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<()> {
        let last_block = self.db.get("LAST_BLOCK")?.unwrap();

        let new_block = Block::new(transactions, String::from_utf8(last_block.to_vec())?, TARGET_HEXT)?;
        self.db.insert(new_block.get_hash(), bincode::serialize(&new_block)?)?;
        self.db.insert("LAST_BLOCK", new_block.get_hash().as_bytes())?;
        self.db.flush()?;
        self.tip = new_block.get_hash();
        Ok(())
    }

    pub fn iter(&self) -> BlockchainIterator {
        BlockchainIterator {
            curr_hash: self.tip.clone(),
            bc: &self
        }
    }

    fn find_unspent_transactions(&self, address: &str) -> Vec<Transaction> {
        let mut spent_tx0s = HashMap::new();
        let mut unspent_tx0s = Vec::new();

        for block in self.iter() {
            for tx in block.get_transactions() {
                let txid = tx.get_id();

                'outputs: for (out_idx, out) in tx.get_outs().iter().enumerate() {
                    if spent_tx0s.contains_key(&txid) {
                        let spent_outs = spent_tx0s.get(&txid).unwrap();
                        for &spent_out in spent_outs {
                            if  spent_out == out_idx as f32 {
                                continue 'outputs;
                            }
                        }
                    }

                    if out.can_be_unlocked_with(address.to_owned()) {
                        unspent_tx0s.push(tx.clone());
                    }
                }

                if !tx.is_coinbase() {
                    for input in tx.get_ins() {
                        if input.can_unlock_output_with(address.to_owned()) {
                            let in_txid = input.get_txid();
                            let in_out = input.get_vout();

                            spent_tx0s.entry(in_txid).or_insert(vec![]).push(in_out);
                        }
                    }
                }
            }
        }

        unspent_tx0s
    }

    // Finds and returns all unspent transaction outputs
    pub fn find_UTX0(&self, address: &str) -> HashMap<String, TXOutput> {
        let mut utxos: HashMap<String, TXOutput> = HashMap::new();
        let mut spent_utxos = HashMap::<String, Vec<f32>>::new();

        for block in self.iter() {
            for tx in block.get_transactions() {
                for index in 0..tx.get_outs().len() {
                    if let Some(ids) = spent_utxos.get(&tx.get_id()) {
                        if ids.contains(&(index as f32)) {
                            continue;
                        }
                    }

                    match utxos.get_mut(&tx.get_id()) {
                        Some(out) => {
                            out.value += tx.get_outs()[index].get_value();
                        }
                        None => {
                            utxos.insert(
                                tx.get_id().clone(),
                                TXOutput {
                                    value: tx.get_outs()[index].get_value(),
                                    script_pub_key: tx.get_outs()[index].get_script_pub_key().clone(),
                                }
                            );
                        }
                    }
                }
            }
        }
        utxos
    }

    pub fn find_spendable_outputs(
        &self, 
        address: &str,
        amount: f32
    ) -> (f32, HashMap<String, Vec<f32>>) {
        let mut unspent_outputs = HashMap::<String, Vec<f32>>::new();
        let mut accumulated = 0.0;

        let unspent_txs = self.find_unspent_transactions(address);

        'outputs: for tx in unspent_txs {
            let txid = tx.get_id();

            for (out_idx, out) in tx.get_outs().iter().enumerate() {
                if out.can_be_unlocked_with(address.to_owned()) && accumulated < amount {
                    accumulated += out.get_value();
                    unspent_outputs.entry(txid.clone()).or_insert(vec![]).push(out_idx as f32);

                    if accumulated >= amount {
                        break 'outputs;
                    }
                }
            }
        }

        (accumulated, unspent_outputs)
    }

    
}

impl<'a> Iterator for BlockchainIterator<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(encoded_block) = self.bc.db.get(&self.curr_hash) {
            return match encoded_block {
                Some(b) => {
                    if let Ok(block) = deserialize::<Block>(&b) {
                        self.curr_hash = block.get_prev_hash();
                        Some(block)
                    } else {
                        None
                    }
                }
                None => None,
            };
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain() {
        let mut bc = Blockchain::new().unwrap();
        let mut tx = Transaction::new_coinbase("Alice".to_string(), "Bob".to_string()).unwrap();
        bc.add_block(vec![tx]).unwrap();
        tx = Transaction::new_coinbase("Bob".to_string(), "Alice".to_string()).unwrap();
        bc.add_block(vec![tx]).unwrap();

        // Check the blocks]
        let mut iter = bc.iter();
        let block = iter.next().unwrap();

        let mut txs = block.get_transactions();
        assert_eq!(txs.len(), 1);

        let block = iter.next().unwrap();

        txs = block.get_transactions();
        assert_eq!(txs.len(), 1);
        dbg!(block);
    }
}
use std::collections::HashMap;
use std::fmt::Display;

use crypto::digest::Digest;
use crypto::{ed25519, ripemd160};
use failure::{format_err, Error};
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use crate::errors::{Result};
use crate::blockchain::Blockchain;
use crate::tx::{TXInput, TXOutput};
use crate::wallet::Wallets;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    id: String,
    vin: Vec<TXInput>,
    vout: Vec<TXOutput>
}

impl Transaction {
    
    pub fn new_utxo(from: &str, to: &str, amount: f32, bc: &Blockchain) -> Result<Transaction> {
        let mut vin = Vec::new();

        let wallets = Wallets::new();
        let wallet = match wallets.get_wallet(from) {
            Some(wallet) => wallet,
            None => {
                return Err(format_err!("ERROR: Wallet not found"))
            }
        };

        if let None = wallets.get_wallet(&to) {
            return Err(format_err!("ERROR: Wallet not found"));
        }

        let mut pub_key_hash = wallet.get_pub_key();
        hash_pub_key(&mut pub_key_hash);


        let acc_v = bc.find_spendable_outputs(from, amount);

        if acc_v.0 < amount {
            return Err(format_err!("ERROR: Not enough funds"));
        }

        let mut acc_txs = acc_v.1;
        for (txid, outs) in acc_txs.iter() {
            let txid = txid.clone();
            for out in outs {
                vin.push(
                    TXInput::new(
                        txid.clone(),
                        out.clone(),
                        vec![],
                        wallet.get_pub_key().clone()
                    ));   
            }
        } 

        let mut vout = vec![TXOutput::new(
            amount,
            to.to_string()
        )];
        
        if acc_v.0 > amount {
            vout.push(TXOutput::new(
                acc_v.0 - amount,
                from.to_string()
            ));
        }

        let mut tx = Transaction {
            id: String::new(),
            vin,
            vout
        };

        tx.set_id()?;

        Ok(tx)
    }

    pub fn new_coinbase(to: String, mut data: String) -> Result<Transaction> {
        if data == String::new() {
            data += &format!("Reward to '{}'", to);
        }

        let mut tx = Transaction {
            id: String::new(),
            vin: vec![],
            vout: vec![TXOutput::new(100.0, to)]
        };

        tx.set_id()?;
        Ok(tx)
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_ins(&self) -> Vec<TXInput> {
        self.vin.clone()
    }

    pub fn get_outs(&self) -> Vec<TXOutput> {
        self.vout.clone()
    }

    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.get_ins()[0].get_txid() == String::new() && self.get_ins()[0].get_vout() == -1.0
    }

    pub fn verify(&mut self, prev_txs: HashMap<String, Transaction>) -> Result<bool> {
        if self.is_coinbase() {
            return Ok(true);
        }

        for vin in &self.get_ins() {
            if prev_txs.get(&vin.get_txid()).is_none() {
                return Err(format_err!("ERROR: Previous transaction is not correct"));
            
            }
        }

        let mut tx_copy = self.trim_copy();

        for id in 0..self.get_ins().len() {
            let prev_tx = prev_txs.get(&self.get_ins()[id].get_txid()).unwrap();
            tx_copy.get_ins()[id].set_signature(vec![]);
            tx_copy.get_ins()[id].set_pub_key(prev_tx.get_outs()[self.get_ins()[id].get_vout() as usize].get_script_pub_key().clone().into());

            tx_copy.set_id()?;
            tx_copy.get_ins()[id].set_pub_key(vec![]);

            if !ed25519::verify(&tx_copy.hash(), &self.get_ins()[id].get_signature(), &self.get_ins()[id].get_pub_key()) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn set_id(&mut self) -> Result<()> {
        let hash = self.hash();
        self.id = hash.iter().map(|b| format!("{:02x}", b)).collect();
        Ok(())
    }

    pub fn sign(
        &mut self,
        private_key: &[u8],
        prev_txs: HashMap<String, Transaction>
    ) -> Result<()> {
        if self.is_coinbase() {
            return Ok(());
        }

        for vin in &self.get_ins() {
            if prev_txs.get(&vin.get_txid()).is_none() {
                return Err(format_err!("ERROR: Previous transaction is not correct"));
            }
        }

        let mut tx_copy = self.trim_copy();

        for id in 0..tx_copy.get_ins().len() {
            let prev_tx = prev_txs.get(&tx_copy.get_ins()[id].get_txid()).unwrap();
            tx_copy.get_ins()[id].set_signature(vec![]);
            tx_copy.get_ins()[id].set_pub_key(prev_tx.get_outs()[tx_copy.get_ins()[id].get_vout() as usize].get_script_pub_key().clone().into());
            tx_copy.set_id()?;
            tx_copy.get_ins()[id].set_pub_key(vec![]);
            let signature = 
                ed25519::signature(&tx_copy.hash(), private_key).to_vec();
            self.get_ins()[id].set_signature(signature);
        }

        Ok(())
    }

    fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        let data = bincode::serialize(&self).unwrap();
        hasher.update(&data);
        hasher.finalize().to_vec()
    }

    fn trim_copy(&self) -> Transaction {
        let mut ins = Vec::new();
        let mut outs = Vec::new();

        for vin in &self.get_ins() {
            ins.push(TXInput::new(
                vin.get_txid().clone(),
                vin.get_vout().clone(),
                Vec::new(),
                Vec::new())
            );
        }

        for vout in &self.get_outs() {
            outs.push(TXOutput::new(
                vout.get_value(),
                vout.get_script_pub_key().clone())
            )
        }

        Transaction {
            id: self.get_id(),
            vin: ins,
            vout: outs
        }
    }
}

pub fn hash_pub_key(pub_key: &mut Vec<u8>) {
    let mut hasher = Sha256::new();
    hasher.update(pub_key);
    *pub_key = hasher.finalize().to_vec();
    let mut hasher_ripemd = ripemd160::Ripemd160::new();
    hasher_ripemd.input(pub_key);
    pub_key.resize(20, 0);
    hasher_ripemd.result(&mut pub_key);
}
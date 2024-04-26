use std::fmt::Display;

use failure::{format_err, Error};
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use crate::errors::{Result};
use crate::blockchain::Blockchain;
use crate::tx::{TXInput, TXOutput};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    id: String,
    vin: Vec<TXInput>,
    vout: Vec<TXOutput>
}

impl Transaction {
    
    pub fn new_UTX0(from: &str, to: &str, amount: f32, bc: &Blockchain) -> Result<Transaction> {
        let mut vin = Vec::new();
        let acc_v = bc.find_spendable_outputs(from, amount);

        if acc_v.0 < amount {
            return Err(format_err!("ERROR: Not enough funds"));
        }

        let mut acc = 0.0;
        let mut acc_txs = acc_v.1;
        for (txid, outs) in acc_txs.iter() {
            let txid = txid.clone();
            for out in outs {
                acc += out;
                vin.push(TXInput::new(txid.clone(), *out, from.to_string()));

                if acc >= amount {
                    break;
                }
            }
        } 

        let mut vout = Vec::new();
        vout.push(TXOutput::new(amount, to.to_string()));
        if acc > amount {
            vout.push(TXOutput::new(acc - amount, from.to_string()));
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
            vin: vec![TXInput::new(String::new(), -1.0, data)],
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

    pub fn set_id(&mut self) -> Result<()> {
        let mut hasher = Sha256::new();
        let data = bincode::serialize(&self)?;
        hasher.update(&data);
        self.id = hasher.finalize().to_vec().iter().map(|b| format!("{:02x}", b)).collect::<String>();
        Ok(())
    }
}
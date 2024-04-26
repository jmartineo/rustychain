use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    txid: String,
    vout: f32,
    pub script_sig: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: f32,
    pub script_pub_key: String
}

impl TXInput {
    pub fn new(txid: String, vout: f32, script_sig: String) -> TXInput {
        TXInput {
            txid,
            vout,
            script_sig
        }
    }

    pub fn can_unlock_output_with(&self, unlocking_data: String) -> bool {
        self.script_sig == unlocking_data
    }

    pub fn get_txid(&self) -> String {
        self.txid.clone()
    }

    pub fn get_vout(&self) -> f32 {
        self.vout
    }
}

impl TXOutput {
    pub fn new(value: f32, script_pub_key: String) -> TXOutput {
        TXOutput {
            value,
            script_pub_key
        }
    }

    pub fn can_be_unlocked_with(&self, unlocking_data: String) -> bool {
        self.script_pub_key == unlocking_data
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }

    pub fn get_script_pub_key(&self) -> String {
        self.script_pub_key.clone()
    }
}

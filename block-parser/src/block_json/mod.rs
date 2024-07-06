pub mod input;
pub use input::*;

pub mod transaction;
pub use transaction::*;

pub mod transition;
pub use transition::*;

use anyhow::{bail, Result};
use serde_json::{Map, Value};
use std::str::FromStr;

pub struct BlockJSON {
    // The block height.
    height: u32,
    // The transactions in the block.
    transactions: Vec<TransactionJSON>,
}

impl BlockJSON {
    // Constructs a new `BlockJSON` from a JSON string.
    // Note that this method does not validate the block.
    pub fn new(string: String) -> Result<Self> {
        // Construct the JSON value.
        let json = match Value::from_str(&string)? {
            Value::Object(json) => json,
            _ => bail!("Invalid JSON object"),
        };
        // Get the block height.
        let height = match json["header"]["metadata"]["height"].as_u64() {
            Some(height) => height as u32,
            None => bail!("Invalid block height"),
        };
        // Get the transactions in the block.
        let transactions = match json["transactions"].as_array() {
            Some(transactions) => transactions
                .iter()
                .map(|transaction| TransactionJSON::new(transaction.clone()))
                .collect::<Result<Vec<_>>>()?,
            None => bail!("Invalid transactions"),
        };
        Ok(Self {
            height,
            transactions,
        })
    }

    // Returns the block height.
    pub fn height(&self) -> u32 {
        self.height
    }

    // Returns the transactions in the block.
    pub fn transactions(&self) -> &Vec<TransactionJSON> {
        &self.transactions
    }
}

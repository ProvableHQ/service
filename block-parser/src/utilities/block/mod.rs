use super::*;

pub mod input;
pub use input::*;

pub mod transaction;
pub use transaction::*;

pub mod transition;
pub use transition::*;

pub struct BlockJSON<N: Network> {
    // The JSON representation of the block.
    json: Map<String, Value>,
    // The block header.
    header: Header<N>,
    _phantom: PhantomData<N>,
}

impl<N: Network> BlockJSON<N> {
    // Constructs a new `BlockJSON` from a JSON string.
    // Note that this method does not validate the block.
    pub fn new(string: String) -> Result<Self> {
        // Construct the JSON value.
        let json = match Value::from_str(&string)? {
            Value::Object(json) => json,
            _ => bail!("Invalid JSON object"),
        };
        // Get the block metadata.
        let header = serde_json::from_value(json["header"].clone())?;
        // Check that it has the required fields.
        if json["transactions"].as_array().is_none() {
            bail!("Missing valid `transactions` field");
        }
        Ok(Self {
            json,
            header,
            _phantom: Default::default(),
        })
    }

    // Returns the block header.
    pub fn header(&self) -> &Header<N> {
        &self.header
    }

    // Returns the transactions in the block.
    pub fn transactions(&self) -> Result<Vec<TransactionJSON<N>>> {
        match self.json["transactions"].as_array() {
            Some(transactions) => transactions
                .iter()
                .map(|transaction| TransactionJSON::new(transaction.clone()))
                .collect(),
            None => bail!("Invalid transactions"),
        }
    }

    // Returns the JSON representation of the block.
    pub fn json(&self) -> &Map<String, Value> {
        &self.json
    }

    // Returns a `Block` if the string is valid.
    pub fn block(&self) -> Result<Block<N>> {
        serde_json::from_value(Value::Object(self.json.clone())).map_err(Into::into)
    }
}

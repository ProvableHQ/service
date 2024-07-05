use super::*;

pub struct TransactionJSON<N: Network> {
    // The JSON representation of the transaction.
    json: Map<String, Value>,
    _phantom: PhantomData<N>,
}

impl<N: Network> TransactionJSON<N> {
    // Constructs a new `TransactionJSON` from a JSON object.
    // Note that this method does not validate the transaction.
    pub fn new(json: Value) -> Result<Self> {
        let json = match json {
            Value::Object(object) => object,
            _ => bail!("Invalid JSON object"),
        };
        // Check that it has the required fields.
        if json["status"].as_str().is_none() {
            bail!("Missing valid `status` field");
        }
        if json["type"].as_str().is_none() {
            bail!("Missing valid `type` field");
        }
        match json["transaction"].as_object() {
            Some(transaction) => {
                if transaction["id"].as_str().is_none() {
                    bail!("Missing valid `transaction.id` field");
                }
            }
            None => bail!("Missing valid `transaction` field"),
        }
        Ok(Self {
            json,
            _phantom: Default::default(),
        })
    }

    // Returns whether or not the transaction is accepted.
    pub fn is_accepted(&self) -> bool {
        // Note that this is safe because we check that the field exists in the constructor.
        self.json["status"].as_str().unwrap() == "accepted"
    }

    // Returns whether or not the transaction is an execution.
    pub fn is_execution(&self) -> bool {
        // Note that this is safe because we check that the field exists in the constructor.
        self.json["type"].as_str().unwrap() == "execution"
    }

    // Returns the ID of the transaction.
    pub fn id(&self) -> &str {
        // Note that this is safe because we check that the field exists in the constructor.
        self.json["transaction"].as_object().unwrap()["id"]
            .as_str()
            .unwrap()
    }

    // Returns the transitions in the transaction, if it is an execution.
    pub fn transitions(&self) -> Result<Vec<TransitionJSON<N>>> {
        match self.json["execution"].as_object() {
            Some(execution) => match execution.get("transitions") {
                Some(transitions) => match transitions.as_array() {
                    Some(transitions) => transitions
                        .iter()
                        .map(|transition| TransitionJSON::new(transition.clone()))
                        .collect(),
                    None => bail!("Invalid `transitions`"),
                },
                None => bail!("Missing valid `transitions` field"),
            },
            None => bail!("Transaction is not an execution"),
        }
    }

    // Returns the JSON representation of the transaction.
    pub fn json(&self) -> &Map<String, Value> {
        &self.json
    }

    // Returns a `Transaction` if the JSON object is valid.
    pub fn transaction(&self) -> Result<Transaction<N>> {
        serde_json::from_value(Value::Object(self.json.clone())).map_err(Into::into)
    }
}

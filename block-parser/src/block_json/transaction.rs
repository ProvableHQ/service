use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionJSON {
    // The JSON representation of the transaction.
    json: Map<String, Value>,
    // The status of the transaction.
    status: String,
    // The type of the transaction.
    type_: String,
    // The transitions in the transaction.
    transitions: Vec<TransitionJSON>,
}

impl TransactionJSON {
    // Constructs a new `TransactionJSON` from a JSON object.
    // Note that this method does not validate the transaction.
    pub fn new(json: Value) -> Result<Self> {
        let json = match json {
            Value::Object(object) => object,
            _ => bail!("Invalid JSON object"),
        };
        // Get the status of the transaction.
        let status = match json["status"].as_str() {
            Some(status) => status.to_string(),
            None => bail!("Invalid transaction status"),
        };
        // Get the type of the transaction.
        let type_ = match json["type"].as_str() {
            Some(type_) => type_.to_string(),
            None => bail!("Invalid transaction type"),
        };
        // Get the transitions in the transaction.
        let transitions = match json["transaction"]["execution"]["transitions"].as_array() {
            Some(transitions) => transitions
                .iter()
                .map(|transition| TransitionJSON::new(transition.clone()))
                .collect::<Result<Vec<_>>>()?,
            None => bail!("Invalid transitions"),
        };
        Ok(Self {
            json,
            status,
            type_,
            transitions,
        })
    }

    // Returns whether or not the transaction is accepted.
    pub fn is_accepted(&self) -> bool {
        self.status == "accepted"
    }

    // Returns whether or not the transaction is an execution.
    pub fn is_execute(&self) -> bool {
        self.type_ == "execute"
    }

    // Returns the transitions in the transaction, if it is an execution.
    pub fn transitions(&self) -> &Vec<TransitionJSON> {
        &self.transitions
    }
}

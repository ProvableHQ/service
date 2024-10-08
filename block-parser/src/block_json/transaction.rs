use super::*;

#[derive(Debug, Clone,Copy, PartialEq, Eq, Hash)]
pub enum TransactionStatus {
    Accepted,
    Rejected,
    Aborted,
}

impl FromStr for TransactionStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "aborted" => Ok(Self::Aborted),
            x => Err(anyhow::anyhow!("Invalid transaction status '{x}'")),
        }
    }
}

impl ToString for TransactionStatus {
    fn to_string(&self) -> String {
        String::from(match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Aborted => "aborted",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionJSON {
    // The ID of the transaction.
    id: String,
    // The JSON representation of the transaction.
    json: Map<String, Value>,
    // The status of the transaction.
    status: TransactionStatus,
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
        // Get the ID of the transaction.
        let id = match json["transaction"]["id"].as_str() {
            Some(id) => id.to_string(),
            None => bail!("Invalid transaction ID"),
        };
        // Get the status of the transaction.
        let status = match json["status"].as_str() {
            Some(status) => TransactionStatus::from_str(status)?,
            None => bail!("Invalid transaction status"),
        };
        // Get the type of the transaction.
        let type_ = match json["type"].as_str() {
            Some(type_) => type_.to_string(),
            None => bail!("Invalid transaction type"),
        };
        // Get the transitions in the transaction.
        let transitions = match json["transaction"]["type"].as_str() {
            Some("execute") => match json["transaction"]["execution"]["transitions"].as_array() {
                Some(transitions) => transitions
                    .iter()
                    .map(|transition| TransitionJSON::new(transition.clone()))
                    .collect::<Result<Vec<_>>>()?,
                None => bail!("Invalid transitions"),
            },
            _ => Vec::new(),
        };
        Ok(Self {
            id,
            json,
            status,
            type_,
            transitions,
        })
    }

    // Returns the transaction ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn status(&self) -> TransactionStatus {
        self.status
    }

    // Returns whether or not the transaction is accepted.
    pub fn is_accepted(&self) -> bool {
        self.status == TransactionStatus::Accepted
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

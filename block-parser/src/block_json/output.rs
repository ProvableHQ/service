use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutputJSON {
    // The type of the output.
    type_: String,
    // The ID of the output.
    id: String,
    // The checksum of the output.
    checksum: Option<String>,
    // The value of the output.
    value: Option<String>,
}

impl OutputJSON {
    // Constructs a new `OutputJSON` from a JSON object.
    // Note that this method does not validate the output.
    pub fn new(json: Value) -> Result<Self> {
        let json = match json {
            Value::Object(object) => object,
            _ => bail!("Invalid JSON object"),
        };
        // Get the type of the output.
        let type_ = match json["type"].as_str() {
            Some(type_) => type_.to_string(),
            None => bail!("Invalid output type"),
        };
        // Get the ID of the output.
        let id = match json["id"].as_str() {
            Some(id) => id.to_string(),
            None => bail!("Invalid output ID"),
        };
        // Get the checksum of the output.
        let checksum = match json.get("checksum").and_then(|v| v.as_str()) {
            Some(checksum) => Some(checksum.to_string()),
            None => None
        };
        // Get the value of the output.
        let value = match json.get("value").and_then(|v| v.as_str()) {
            Some(value) => Some(value.to_string()),
            None => None,
        };
        Ok(Self { type_, id, checksum, value })
    }

    // Returns the type of the output.
    pub fn output_type(&self) -> &str {
        &self.type_
    }

    // Returns the ID of the output.
    pub fn id(&self) -> &str {
        &self.id
    }

    // Returns the checksum of the output.
    pub fn checksum(&self) -> &Option<String> {
        &self.checksum
    }

    // Returns the value of the output.
    pub fn value(&self) -> &Option<String> {
        &self.value
    }
}
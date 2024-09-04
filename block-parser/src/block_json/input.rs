use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputJSON {
    // The type of the input.
    type_: String,
    // The ID of the input.
    id: String,
    // The value of the input.
    value: Option<String>,
}

impl InputJSON {
    // Constructs a new `InputJSON` from a JSON object.
    // Note that this method does not validate the input.
    pub fn new(json: Value) -> Result<Self> {
        let json = match json {
            Value::Object(object) => object,
            _ => bail!("Invalid JSON object"),
        };
        // Get the type of the input.
        let type_ = match json["type"].as_str() {
            Some(type_) => type_.to_string(),
            None => bail!("Invalid input type"),
        };
        // Get the ID of the input.
        let id = match json["id"].as_str() {
            Some(id) => id.to_string(),
            None => bail!("Invalid input ID"),
        };
        // Get the value of the input.
        let value = match json.get("value").and_then(|v| v.as_str()) {
            Some(value) => Some(value.to_string()),
            None => None,
        };
        Ok(Self { type_, id, value })
    }

    // Returns the type of the input.
    pub fn input_type(&self) -> &str {
        &self.type_
    }

    // Returns the ID of the input.
    pub fn id(&self) -> &str {
        &self.id
    }

    // Returns the value of the input.
    pub fn value(&self) -> &Option<String> {
        &self.value
    }
}

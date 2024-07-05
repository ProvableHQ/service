use super::*;

pub struct InputJSON<N: Network> {
    // The JSON representation of the input.
    json: Map<String, Value>,
    _phantom: PhantomData<N>,
}

impl<N: Network> InputJSON<N> {
    // Constructs a new `InputJSON` from a JSON object.
    // Note that this method does not validate the input.
    pub fn new(json: Value) -> Result<Self> {
        let json = match json {
            Value::Object(object) => object,
            _ => bail!("Invalid JSON object"),
        };
        // Check that it has the required fields.
        if json["type"].as_str().is_none() {
            bail!("Missing valid `type` field");
        }
        if json["id"].as_str().is_none() {
            bail!("Missing valid `id` field");
        }
        if json["value"].as_str().is_none() {
            bail!("Missing valid `value` field");
        }
        Ok(Self {
            json,
            _phantom: Default::default(),
        })
    }

    // Returns the type of the input.
    pub fn input_type(&self) -> &str {
        self.json["type"].as_str().unwrap()
    }

    // Returns the ID of the input.
    pub fn id(&self) -> &str {
        self.json["id"].as_str().unwrap()
    }

    // Returns the value of the input.
    pub fn value(&self) -> &str {
        self.json["value"].as_str().unwrap()
    }

    // Returns the JSON representation of the input.
    pub fn json(&self) -> &Map<String, Value> {
        &self.json
    }

    // Returns a `Input` if the JSON object is valid.
    pub fn input(&self) -> Result<Input<N>> {
        serde_json::from_value(Value::Object(self.json.clone())).map_err(Into::into)
    }
}

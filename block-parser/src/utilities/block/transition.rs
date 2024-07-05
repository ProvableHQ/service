use super::*;

pub struct TransitionJSON<N: Network> {
    // The JSON representation of the transition.
    json: Map<String, Value>,
    _phantom: PhantomData<N>,
}

impl<N: Network> TransitionJSON<N> {
    // Constructs a new `TransitionJSON` from a JSON object.
    // Note that this method does not validate the transition.
    pub fn new(json: Value) -> Result<Self> {
        let json = match json {
            Value::Object(object) => object,
            _ => bail!("Invalid JSON object"),
        };
        // Check that it has the required fields.
        if json["program"].as_str().is_none() {
            bail!("Missing valid `program` field");
        }
        if json["function"].as_str().is_none() {
            bail!("Missing valid `function` field");
        }
        if json["inputs"].as_array().is_none() {
            bail!("Missing valid `inputs` field");
        }
        Ok(Self {
            json,
            _phantom: Default::default(),
        })
    }

    // Returns the program ID of the transition.
    pub fn program_id(&self) -> &str {
        // Note that this is safe because we check that the field exists in the constructor.
        self.json["program"].as_str().unwrap()
    }

    // Returns the function name of the transition.
    pub fn function_name(&self) -> &str {
        // Note that this is safe because we check that the field exists in the constructor.
        self.json["function"].as_str().unwrap()
    }

    // Returns the inputs of the transition.
    pub fn inputs(&self) -> Result<Vec<InputJSON<N>>> {
        // Note that this is safe because we check that the field exists in the constructor.
        self.json["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|input| InputJSON::new(input.clone()))
            .collect()
    }

    // Returns the JSON representation of the transition.
    pub fn json(&self) -> &Map<String, Value> {
        &self.json
    }

    // Returns a `Transition` if the JSON object is valid.
    pub fn transition(&self) -> Result<Transition<N>> {
        serde_json::from_value(Value::Object(self.json.clone())).map_err(Into::into)
    }
}

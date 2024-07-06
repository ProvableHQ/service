use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransitionJSON {
    // The transition ID.
    id: String,
    // The program ID of the transition.
    program: String,
    // The function name of the transition.
    function: String,
    // The inputs of the transition.
    inputs: Vec<InputJSON>,
}

impl TransitionJSON {
    // Constructs a new `TransitionJSON` from a JSON object.
    // Note that this method does not validate the transition.
    pub fn new(json: Value) -> Result<Self> {
        let json = match json {
            Value::Object(object) => object,
            _ => bail!("Invalid JSON object"),
        };
        // Get the transition ID.
        let id = match json["id"].as_str() {
            Some(id) => id.to_string(),
            None => bail!("Invalid transition ID"),
        };
        // Get the program ID of the transition.
        let program = match json["program"].as_str() {
            Some(program) => program.to_string(),
            None => bail!("Invalid program ID"),
        };
        // Get the function name of the transition.
        let function = match json["function"].as_str() {
            Some(function) => function.to_string(),
            None => bail!("Invalid function name"),
        };
        // Get the inputs of the transition.
        let inputs = match json["inputs"].as_array() {
            Some(inputs) => inputs
                .iter()
                .map(|input| InputJSON::new(input.clone()))
                .collect::<Result<Vec<_>>>()?,
            None => bail!("Invalid inputs"),
        };
        Ok(Self {
            id,
            program,
            function,
            inputs,
        })
    }

    // Returns the transition ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    // Returns the program ID of the transition.
    pub fn program_id(&self) -> &str {
        &self.program
    }

    // Returns the function name of the transition.
    pub fn function_name(&self) -> &str {
        &self.function
    }

    // Returns the inputs of the transition.
    pub fn inputs(&self) -> &Vec<InputJSON> {
        &self.inputs
    }
}

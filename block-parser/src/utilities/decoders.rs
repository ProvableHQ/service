use super::*;

// Decodes the block from a JSON string.
pub fn decode_block<N: Network>(string: String) -> Result<BlockJSON<N>> {
    BlockJSON::new(string)
}

// Decodes the bonded mapping from a JSON string.
pub fn decode_bonded_mapping<N: Network>(string: &str) -> Result<BondedMapping<N>> {
    // Construct a JSON value from the string.
    let entries = match Value::from_str(string)? {
        Value::Array(array) => array,
        _ => bail!("Expected an array"),
    };

    // Initialize the bonded mapping.
    let mut bonded = BondedMapping::with_capacity(entries.len());
    // Decode the key-value pairs.
    for entry in entries {
        // The entry is an array of 2 elements.
        let mut entry = match entry {
            Value::Array(array) => array,
            _ => bail!("Expected an array"),
        };
        // Check that there are 2 elements.
        ensure!(entry.len() == 2, "Expected 2 elements");
        // Get the first and second elements, both of which are strings.
        let second = match entry.pop().unwrap() {
            Value::String(string) => string,
            _ => bail!("Expected a string"),
        };
        let first = match entry.pop().unwrap() {
            Value::String(string) => string,
            _ => bail!("Expected a string"),
        };

        // Initialize an address string from the first element.
        let key = AddressString::<N>::new(first);

        let value = {
            // The second element is a struct encoded as a string.
            // It has the form:
            // "{\n  validator: aleo1lzv9f68n4hnldtg9rwc5yskkc8drnuynnvqhdquw5d0p6qt5myrqad7agl,\n  microcredits: 10005940908u64\n}"
            // Extract the address of the validator and the amount of microcredits from the string, using regular expressions.
            let re = regex::Regex::new(r"validator:\s*(.*),\s*microcredits:\s*(\d+)u64").unwrap();
            let captures = re.captures(&second).unwrap();
            let address = AddressString::<N>::new(captures.get(1).unwrap().as_str().to_string());
            let amount = captures.get(2).unwrap().as_str().parse::<u64>().unwrap();

            (address, amount)
        };
        bonded.insert(key, value);
    }
    Ok(bonded)
}

// Decodes the unbonding mapping from a JSON string.
pub fn decode_unbonding_mapping<N: Network>(string: &str) -> Result<UnbondedMapping<N>> {
    // Construct a JSON value from the string.
    let entries = match Value::from_str(string)? {
        Value::Array(array) => array,
        _ => bail!("Expected an array"),
    };

    // Initialize the unbonding mapping.
    let mut unbonded = UnbondedMapping::with_capacity(entries.len());
    // Decode the key-value pairs.
    for entry in entries {
        // The entry is an array of 2 elements.
        let mut entry = match entry {
            Value::Array(array) => array,
            _ => bail!("Expected an array"),
        };
        // Check that there are 2 elements.
        ensure!(entry.len() == 2, "Expected 2 elements");
        // Get the first and second elements, both of which are strings.
        let second = match entry.pop().unwrap() {
            Value::String(string) => string,
            _ => bail!("Expected a string"),
        };
        let first = match entry.pop().unwrap() {
            Value::String(string) => string,
            _ => bail!("Expected a string"),
        };

        // Initialize an address string from the first element.
        let key = AddressString::<N>::new(first);

        let value = {
            // The second element is a struct encoded as a string.
            // It has the form:
            // "{\n  microcredits: 10005940908u64,\n  height: 100u32\n}"
            // Extract the amount and duration from the string, using regular expressions.
            let re = regex::Regex::new(r"microcredits:\s*(\d+)u64,\s*height:\s*(\d+)u32").unwrap();
            let captures = re.captures(&second).unwrap();
            let amount = captures.get(1).unwrap().as_str().parse::<u64>().unwrap();
            let duration = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();

            (amount, duration)
        };
        unbonded.insert(key, value);
    }
    Ok(unbonded)
}

// Decodes the withdraw mapping from a JSON string.
pub fn decode_withdraw_mapping<N: Network>(string: &str) -> Result<WithdrawMapping<N>> {
    // Construct a JSON value from the string.
    let entries = match Value::from_str(string)? {
        Value::Array(array) => array,
        _ => bail!("Expected an array"),
    };

    // Initialize the withdraw mapping.
    let mut withdraw = WithdrawMapping::with_capacity(entries.len());
    // Decode the key-value pairs.
    for entry in entries {
        // The entry is an array of 2 elements.
        let mut entry = match entry {
            Value::Array(array) => array,
            _ => bail!("Expected an array"),
        };
        // Check that there are 2 elements.
        ensure!(entry.len() == 2, "Expected 2 elements");
        // Get the first and second elements, both of which are strings.
        let second = match entry.pop().unwrap() {
            Value::String(string) => string,
            _ => bail!("Expected a string"),
        };
        let first = match entry.pop().unwrap() {
            Value::String(string) => string,
            _ => bail!("Expected a string"),
        };

        // Initialize an address string from the first element.
        let key = AddressString::<N>::new(first);
        // Initialize an address string from the second element.
        let value = AddressString::<N>::new(second);

        withdraw.insert(key, value);
    }
    Ok(withdraw)
}

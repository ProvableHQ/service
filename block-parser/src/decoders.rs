use super::*;

use nom::character::complete::digit1;
use nom::sequence::tuple;
use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, multispace0},
    IResult,
};

// Decodes the block from a JSON string into a sequence of `credits.aleo` operations and the block height.
// Note that this method checks that the block is valid at the expense of performance.
pub fn decode_block<N: Network>(string: &str) -> Result<(Vec<CreditsOperations>, u32)> {
    // Decode the block.
    let block: Block<N> = serde_json::from_str(string)?;
    // Get the transactions from the block.
    let transactions = block.transactions();
    // Initialize storage for the credits transactions.
    let mut credits_transactions = Vec::new();
    // Extract the desired `credits.aleo` transitions.
    for transaction in transactions.iter() {
        // If the transaction is accepted, an execution, and the correct function, extract the appropriate information.
        if transaction.is_accepted() && transaction.is_execute() {
            // Process the transitions in the transaction.
            for transition in transaction.transitions() {
                match (
                    transition.program_id().to_string().as_str(),
                    transition.function_name().to_string().as_str(),
                ) {
                    ("credits.aleo", "bond_public") => {
                        // Get the transition ID.
                        let id = transition.id().to_string();
                        // Get the inputs of the transition.
                        let inputs = transition.inputs();
                        // Check that there are 3 inputs.
                        ensure!(inputs.len() == 3, "Expected 3 inputs");
                        // Get the validator address, withdrawal address, and amount from the inputs.
                        let validator = match inputs.first().unwrap() {
                            Input::Public(
                                _,
                                Some(Plaintext::Literal(Literal::Address(validator), _)),
                            ) => validator.to_string(),
                            _ => bail!("Expected an address"),
                        };
                        let withdrawal = match inputs.get(1).unwrap() {
                            Input::Public(
                                _,
                                Some(Plaintext::Literal(Literal::Address(withdrawal), _)),
                            ) => withdrawal.to_string(),
                            _ => bail!("Expected an address"),
                        };
                        let amount = match inputs.get(2).unwrap() {
                            Input::Public(_, Some(Plaintext::Literal(Literal::U64(amount), _))) => {
                                **amount
                            }
                            _ => bail!("Expected an integer"),
                        };
                        // Add the `bond_public` operation to the credits transactions.
                        credits_transactions.push(CreditsOperations::BondPublic {
                            id,
                            validator,
                            withdrawal,
                            amount,
                        });
                    }
                    ("credits.aleo", "claim_unbond_public") => {
                        // Get the transition ID.
                        let id = transition.id().to_string();
                        // Get the inputs of the transition.
                        let inputs = transition.inputs();
                        // Check that there is 1 input.
                        ensure!(inputs.len() == 1, "Expected 1 input");
                        // Get the staker address from the inputs.
                        let staker = match inputs.first().unwrap() {
                            Input::Public(
                                _,
                                Some(Plaintext::Literal(Literal::Address(staker), _)),
                            ) => staker.to_string(),
                            _ => bail!("Expected an address"),
                        };
                        // Add the `claim_unbond_public` operation to the credits transactions.
                        credits_transactions
                            .push(CreditsOperations::ClaimUnbondPublic { id, staker });
                    }
                    ("credits.aleo", "unbond_public") => {
                        // Get the transition ID.
                        let id = transition.id().to_string();
                        // Get the inputs of the transition.
                        let inputs = transition.inputs();
                        // Check that there are 2 inputs.
                        ensure!(inputs.len() == 2, "Expected 2 inputs");
                        // Get the staker address and amount from the inputs.
                        let staker = match inputs.first().unwrap() {
                            Input::Public(
                                _,
                                Some(Plaintext::Literal(Literal::Address(staker), _)),
                            ) => staker.to_string(),
                            _ => bail!("Expected an address"),
                        };
                        let amount = match inputs.get(1).unwrap() {
                            Input::Public(_, Some(Plaintext::Literal(Literal::U64(amount), _))) => {
                                **amount
                            }
                            _ => bail!("Expected an integer"),
                        };
                        // Add the `unbond_public` operation to the credits transactions.
                        credits_transactions.push(CreditsOperations::UnbondPublic {
                            id,
                            staker,
                            amount,
                        });
                    }
                    _ => {} // Do nothing.
                }
            }
        }
    }
    Ok((credits_transactions, block.height()))
}

// Decodes the block from a JSON string into a sequence of `credits.aleo` operations and the block height.
// Note that this method does **not** checks that the block is valid for improved performance.
pub fn decode_block_unchecked<N: Network>(string: &str) -> Result<(Vec<CreditsOperations>, u32)> {
    // Construct the unchecked BlockJSON from the string.
    let block = BlockJSON::new(string.to_string())?;
    // Get the transactions from the block.
    let transactions = block.transactions();
    // Initialize storage for the credits transactions.
    let mut credits_transactions = Vec::new();
    // Extract the desired `credits.aleo` transitions.
    for transaction in transactions {
        // If the transaction is accepted, an execution, and the correct function, extract the appropriate information.
        if transaction.is_accepted() && transaction.is_execute() {
            // Process the transitions in the transaction.
            for transition in transaction.transitions() {
                match (transition.program_id(), transition.function_name()) {
                    ("credits.aleo", "bond_public") => {
                        // Get the transition ID.
                        let id = transition.id().to_string();
                        // Get the inputs of the transition.
                        let inputs = transition.inputs();
                        // Check that there are 3 inputs.
                        ensure!(inputs.len() == 3, "Expected 3 inputs");
                        // Get the validator address, withdrawal address, and amount from the inputs.
                        let validator = inputs.first().unwrap().value().to_owned().unwrap();
                        let withdrawal = inputs.get(1).unwrap().value().to_owned().unwrap();
                        let inputs_value = inputs.get(2).unwrap().value();
                        let amount = match inputs_value {
                            Some(v) => *U64::<N>::from_str(&v)?,
                            None => bail!("Invalid JSON object"),
                        };
                        // Add the `bond_public` operation to the credits transactions.
                        credits_transactions.push(CreditsOperations::BondPublic {
                            id,
                            validator,
                            withdrawal,
                            amount,
                        });
                    }
                    ("credits.aleo", "claim_unbond_public") => {
                        // Get the transition ID.
                        let id = transition.id().to_string();
                        // Get the inputs of the transition.
                        let inputs = transition.inputs();
                        // Check that there is 1 input.
                        ensure!(inputs.len() == 1, "Expected 1 input");
                        // Get the staker address from the inputs.
                        let staker = match inputs.first().unwrap().value() {
                            Some(v) => v.to_string(),
                            None => bail!("Invalid response for staker address"),
                        };
                        // Add the `claim_unbond_public` operation to the credits transactions.
                        credits_transactions
                            .push(CreditsOperations::ClaimUnbondPublic { id, staker });
                    }
                    ("credits.aleo", "unbond_public") => {
                        // Get the transition ID.
                        let id = transition.id().to_string();
                        // Get the inputs of the transition.
                        let inputs = transition.inputs();
                        // Check that there are 2 inputs.
                        ensure!(inputs.len() == 2, "Expected 2 inputs");
                        // Get the staker address and amount from the inputs.
                        let staker = match inputs.first().unwrap().value() {
                            Some(v) => v.to_string(),
                            None => bail!("Invalid response for staker address"),
                        };
                        let inputs_value = inputs.get(1).unwrap().value();
                        let amount = match inputs_value {
                            Some(v) => *U64::<N>::from_str(&v)?,
                            None => bail!("Invalid JSON object"),
                        };
                        // Add the `unbond_public` operation to the credits transactions.
                        credits_transactions.push(CreditsOperations::UnbondPublic {
                            id,
                            staker,
                            amount,
                        });
                    }
                    _ => {} // Do nothing.
                }
            }
        }
    }
    Ok((credits_transactions, block.height()))
}

// Decodes the bonded mapping from a JSON string.
// Note that this method checks that the bonded mapping is valid at the expense of performance.
pub fn decode_bonded_mapping<N: Network>(string: &str) -> Result<BondedMapping> {
    // Deserialize the JSON string into a sequence of key-value pairs.
    let entries: Vec<(Plaintext<N>, Value<N>)> = serde_json::from_str(string)?;
    // Initialize the bonded mapping.
    let mut bonded = BondedMapping::with_capacity(entries.len());
    // Extract the address, validator, and microcredits from the key-value pairs.
    for (plaintext, value) in entries {
        // Get the address from the plaintext.
        let address = match plaintext {
            Plaintext::Literal(Literal::Address(address), _) => address.to_string(),
            _ => bail!("Expected an address"),
        };
        // Get the validator and microcredits from the value.
        let (validator, microcredits) = match value {
            Value::Plaintext(Plaintext::Struct(members, _)) => {
                // Check that the struct has 2 members.
                ensure!(members.len() == 2, "Expected 2 members");
                // Get the validator and microcredits from the members.
                let validator = match members.get(&Identifier::from_str("validator")?) {
                    Some(Plaintext::Literal(Literal::Address(address), _)) => address,
                    _ => bail!("Expected an address"),
                };
                let microcredits = match members.get(&Identifier::from_str("microcredits")?) {
                    Some(Plaintext::Literal(Literal::U64(microcredits), _)) => **microcredits,
                    _ => bail!("Expected an integer"),
                };
                (validator.to_string(), microcredits)
            }
            _ => bail!("Expected a struct"),
        };
        bonded.insert(address, (validator, microcredits));
    }
    Ok(bonded)
}

// Decodes the bonded mapping from a JSON string.
// Note that this method does not check that the bonded mapping is valid for improved performance.
pub fn decode_bonded_mapping_unchecked(string: &str) -> Result<BondedMapping> {
    // Construct a JSON value from the string.
    let entries = match serde_json::Value::from_str(string)? {
        serde_json::Value::Array(array) => array,
        _ => bail!("Expected an array"),
    };

    // Initialize the bonded mapping.
    let mut bonded = BondedMapping::with_capacity(entries.len());
    // Decode the key-value pairs.
    for entry in entries {
        // The entry is an array of 2 elements.
        let mut entry = match entry {
            serde_json::Value::Array(array) => array,
            _ => bail!("Expected an array"),
        };
        // Check that there are 2 elements.
        ensure!(entry.len() == 2, "Expected 2 elements");
        // Get the first and second elements, both of which are strings.
        let second = match entry.pop().unwrap() {
            serde_json::Value::String(string) => string,
            _ => bail!("Expected a string"),
        };
        let key = match entry.pop().unwrap() {
            serde_json::Value::String(string) => string,
            _ => bail!("Expected a string"),
        };

        let value = {
            // The second element is a struct encoded as a string.
            // It has the form:
            // "{\n  validator: aleo1lzv9f68n4hnldtg9rwc5yskkc8drnuynnvqhdquw5d0p6qt5myrqad7agl,\n  microcredits: 10005940908u64\n}"
            // `parse_data` is a helper function to extract the address and amount from the string.
            fn parse_data(input: &str) -> IResult<&str, (&str, &str)> {
                // Parse "{\n validator: ".
                let (input, _) =
                    tuple((tag("{"), multispace0, tag("validator:"), multispace0))(input)?;
                // Parse the validator address.
                let (input, validator) = alphanumeric1(input)?;
                // Parse ",\n microcredits: ".
                let (input, _) = tuple((
                    multispace0,
                    tag(","),
                    multispace0,
                    tag("microcredits:"),
                    multispace0,
                ))(input)?;
                // Parse the microcredits amount.
                let (input, microcredits) = digit1(input)?;
                // Parse "u64\n}".
                let (input, _) = tuple((multispace0, tag("u64"), multispace0, tag("}")))(input)?;
                Ok((input, (validator, microcredits)))
            }
            let (address, amount) = match parse_data(&second) {
                Ok((_, (validator, microcredits))) => {
                    (validator.to_string(), u64::from_str(microcredits)?)
                }
                Err(_) => bail!("Failed to parse data"),
            };
            (address, amount)
        };
        bonded.insert(key, value);
    }
    Ok(bonded)
}

// Decodes the unbonding mapping from a JSON string.
// Note that this method checks that the unbonding mapping is valid at the expense of performance.
pub fn decode_unbonding_mapping<N: Network>(string: &str) -> Result<UnbondedMapping> {
    // Deserialize the JSON string into a sequence of key-value pairs.
    let entries: Vec<(Plaintext<N>, Value<N>)> = serde_json::from_str(string)?;
    // Initialize the unbonding mapping.
    let mut unbonded = UnbondedMapping::with_capacity(entries.len());
    // Extract the address, amount, and height from the key-value pairs.
    for (plaintext, value) in entries {
        // Get the address from the plaintext.
        let address = match plaintext {
            Plaintext::Literal(Literal::Address(address), _) => address.to_string(),
            _ => bail!("Expected an address"),
        };
        // Get the amount and height from the value.
        let (amount, height) = match value {
            Value::Plaintext(Plaintext::Struct(members, _)) => {
                // Check that the struct has 2 members.
                ensure!(members.len() == 2, "Expected 2 members");
                // Get the amount and height from the members.
                let amount = match members.get(&Identifier::from_str("microcredits")?) {
                    Some(Plaintext::Literal(Literal::U64(amount), _)) => **amount,
                    _ => bail!("Expected an integer"),
                };
                let height = match members.get(&Identifier::from_str("height")?) {
                    Some(Plaintext::Literal(Literal::U32(height), _)) => **height,
                    _ => bail!("Expected an integer"),
                };
                (amount, height)
            }
            _ => bail!("Expected a struct"),
        };
        unbonded.insert(address, (amount, height));
    }
    Ok(unbonded)
}

// Decodes the unbonding mapping from a JSON string.
// Note that this method does not check that the unbonding mapping is valid for improved performance.
pub fn decode_unbonding_mapping_unchecked(string: &str) -> Result<UnbondedMapping> {
    // Construct a JSON value from the string.
    let entries = match serde_json::Value::from_str(string)? {
        serde_json::Value::Array(array) => array,
        _ => bail!("Expected an array"),
    };

    // Initialize the unbonding mapping.
    let mut unbonded = UnbondedMapping::with_capacity(entries.len());
    // Decode the key-value pairs.
    for entry in entries {
        // The entry is an array of 2 elements.
        let mut entry = match entry {
            serde_json::Value::Array(array) => array,
            _ => bail!("Expected an array"),
        };
        // Check that there are 2 elements.
        ensure!(entry.len() == 2, "Expected 2 elements");
        // Get the first and second elements, both of which are strings.
        let second = match entry.pop().unwrap() {
            serde_json::Value::String(string) => string,
            _ => bail!("Expected a string"),
        };
        let key = match entry.pop().unwrap() {
            serde_json::Value::String(string) => string,
            _ => bail!("Expected a string"),
        };

        let value = {
            // The second element is a struct encoded as a string.
            // It has the form:
            // "{\n  microcredits: 10005940908u64,\n  height: 100u32\n}"
            // `parse_data` is a helper function to extract the amount and height from the string.
            fn parse_data(input: &str) -> IResult<&str, (&str, &str)> {
                // Parse "{\n microcredits: ".
                let (input, _) =
                    tuple((tag("{"), multispace0, tag("microcredits:"), multispace0))(input)?;
                // Parse the microcredits amount.
                let (input, microcredits) = digit1(input)?;
                // Parse "u64,\n height: ".
                let (input, _) = tuple((
                    tag("u64"),
                    multispace0,
                    tag(","),
                    multispace0,
                    tag("height:"),
                    multispace0,
                ))(input)?;
                // Parse the height.
                let (input, height) = digit1(input)?;
                // Parse "u32\n}".
                let (input, _) = tuple((tag("u32"), multispace0, tag("}")))(input)?;
                Ok((input, (microcredits, height)))
            }
            let (amount, height) = match parse_data(&second) {
                Ok((_, (amount, duration))) => (amount.parse::<u64>()?, duration.parse::<u32>()?),
                Err(_) => bail!("Failed to parse data"),
            };

            (amount, height)
        };
        unbonded.insert(key, value);
    }
    Ok(unbonded)
}

// Decodes the withdraw mapping from a JSON string.
// Note that this method checks that the withdraw mapping is valid at the expense of performance.
pub fn decode_withdraw_mapping<N: Network>(string: &str) -> Result<WithdrawMapping> {
    // Deserialize the JSON string into a sequence of key-value pairs.
    let entries: Vec<(Plaintext<N>, Value<N>)> = serde_json::from_str(string)?;
    // Initialize the withdraw mapping.
    let mut withdraw = WithdrawMapping::with_capacity(entries.len());
    // Extract the staker address and withdrawal address from the key-value pairs.
    for (plaintext, value) in entries {
        // Get the staker address from the plaintext.
        let staker = match plaintext {
            Plaintext::Literal(Literal::Address(address), _) => address.to_string(),
            _ => bail!("Expected an address"),
        };
        // Get the withdrawal address from the value.
        let withdrawal = match value {
            Value::Plaintext(Plaintext::Literal(Literal::Address(address), _)) => {
                address.to_string()
            }
            _ => bail!("Expected an address"),
        };
        withdraw.insert(staker, withdrawal);
    }
    Ok(withdraw)
}

// Decodes the withdraw mapping from a JSON string.
// Note that this method does not check that the withdraw mapping is valid for improved performance.
pub fn decode_withdraw_mapping_unchecked(string: &str) -> Result<WithdrawMapping> {
    // Construct a JSON value from the string.
    let entries = match serde_json::Value::from_str(string)? {
        serde_json::Value::Array(array) => array,
        _ => bail!("Expected an array"),
    };

    // Initialize the withdraw mapping.
    let mut withdraw = WithdrawMapping::with_capacity(entries.len());
    // Decode the key-value pairs.
    for entry in entries {
        // The entry is an array of 2 elements.
        let mut entry = match entry {
            serde_json::Value::Array(array) => array,
            _ => bail!("Expected an array"),
        };
        // Check that there are 2 elements.
        ensure!(entry.len() == 2, "Expected 2 elements");
        // Get the first and second elements, both of which are strings.
        let value = match entry.pop().unwrap() {
            serde_json::Value::String(string) => string,
            _ => bail!("Expected a string"),
        };
        let key = match entry.pop().unwrap() {
            serde_json::Value::String(string) => string,
            _ => bail!("Expected a string"),
        };

        withdraw.insert(key, value);
    }
    Ok(withdraw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::prelude::CanaryV0;

    type CurrentNetwork = CanaryV0;

    #[test]
    fn test_decode_block() {
        let block_json = include_str!("../tests/test_bond_public/block.json");
        let (operations_checked, height_checked) =
            decode_block::<CurrentNetwork>(block_json).unwrap();
        let (operations_unchecked, height_unchecked) =
            decode_block_unchecked::<CurrentNetwork>(block_json).unwrap();
        assert_eq!(operations_checked, operations_unchecked);
        assert_eq!(height_checked, height_unchecked);

        // TODO: Update these with valid blocks from CanaryV0.
        // These blocks have old `bond_public` transactions that are no longer valid (needs 3 inputs instead of 2).
        // let block_json = include_str!("../tests/test_claim_unbond_public/block.json");
        // let operations_checked = decode_block::<CurrentNetwork>(block_json).unwrap();
        // let operations_unchecked = decode_block_unchecked::<CurrentNetwork>(block_json).unwrap();
        // assert_eq!(operations_checked, operations_unchecked);
        //
        // let block_json = include_str!("../tests/test_complex_bond_and_unbond/block.json");
        // let operations_checked = decode_block::<CurrentNetwork>(block_json).unwrap();
        // let operations_unchecked = decode_block_unchecked::<CurrentNetwork>(block_json).unwrap();
        // assert_eq!(operations_checked, operations_unchecked);

        let block_json = include_str!("../tests/test_empty_block/block.json");
        let (operations_checked, height_checked) =
            decode_block::<CurrentNetwork>(block_json).unwrap();
        let (operations_unchecked, height_unchecked) =
            decode_block_unchecked::<CurrentNetwork>(block_json).unwrap();
        assert_eq!(operations_checked, operations_unchecked);
        assert_eq!(height_checked, height_unchecked);
        assert_eq!(operations_checked.len(), 0);
    }

    #[test]
    fn test_decode_bonded_mapping() {
        let bonded_json = include_str!("../tests/test_bond_public/bonded.json");
        let bonded_checked = decode_bonded_mapping::<CurrentNetwork>(bonded_json).unwrap();
        let bonded_unchecked = decode_bonded_mapping_unchecked(bonded_json).unwrap();
        assert_eq!(bonded_checked, bonded_unchecked);

        let bonded_json = include_str!("../tests/test_claim_unbond_public/bonded.json");
        let bonded_checked = decode_bonded_mapping::<CurrentNetwork>(bonded_json).unwrap();
        let bonded_unchecked = decode_bonded_mapping_unchecked(bonded_json).unwrap();
        assert_eq!(bonded_checked, bonded_unchecked);

        let bonded_json = include_str!("../tests/test_complex_bond_and_unbond/bonded.json");
        let bonded_checked = decode_bonded_mapping::<CurrentNetwork>(bonded_json).unwrap();
        let bonded_unchecked = decode_bonded_mapping_unchecked(bonded_json).unwrap();
        assert_eq!(bonded_checked, bonded_unchecked);

        let bonded_json = include_str!("../tests/test_empty_block/bonded.json");
        let bonded_checked = decode_bonded_mapping::<CurrentNetwork>(bonded_json).unwrap();
        let bonded_unchecked = decode_bonded_mapping_unchecked(bonded_json).unwrap();
        assert_eq!(bonded_checked, bonded_unchecked);
    }

    #[test]
    fn test_decode_unbonding_mapping() {
        let unbonded_json = include_str!("../tests/test_bond_public/unbonding.json");
        let unbonded_checked = decode_unbonding_mapping::<CurrentNetwork>(unbonded_json).unwrap();
        let unbonded_unchecked = decode_unbonding_mapping_unchecked(unbonded_json).unwrap();
        assert_eq!(unbonded_checked, unbonded_unchecked);

        let unbonded_json = include_str!("../tests/test_claim_unbond_public/unbonding.json");
        let unbonded_checked = decode_unbonding_mapping::<CurrentNetwork>(unbonded_json).unwrap();
        let unbonded_unchecked = decode_unbonding_mapping_unchecked(unbonded_json).unwrap();
        assert_eq!(unbonded_checked, unbonded_unchecked);

        let unbonded_json = include_str!("../tests/test_complex_bond_and_unbond/unbonding.json");
        let unbonded_checked = decode_unbonding_mapping::<CurrentNetwork>(unbonded_json).unwrap();
        let unbonded_unchecked = decode_unbonding_mapping_unchecked(unbonded_json).unwrap();
        assert_eq!(unbonded_checked, unbonded_unchecked);

        let unbonded_json = include_str!("../tests/test_empty_block/unbonding.json");
        let unbonded_checked = decode_unbonding_mapping::<CurrentNetwork>(unbonded_json).unwrap();
        let unbonded_unchecked = decode_unbonding_mapping_unchecked(unbonded_json).unwrap();
        assert_eq!(unbonded_checked, unbonded_unchecked);
    }

    #[test]
    fn test_decode_withdraw_mapping() {
        let withdraw_json = include_str!("../tests/test_bond_public/withdraw.json");
        let withdraw_checked = decode_withdraw_mapping::<CurrentNetwork>(withdraw_json).unwrap();
        let withdraw_unchecked = decode_withdraw_mapping_unchecked(withdraw_json).unwrap();
        assert_eq!(withdraw_checked, withdraw_unchecked);

        let withdraw_json = include_str!("../tests/test_claim_unbond_public/withdraw.json");
        let withdraw_checked = decode_withdraw_mapping::<CurrentNetwork>(withdraw_json).unwrap();
        let withdraw_unchecked = decode_withdraw_mapping_unchecked(withdraw_json).unwrap();
        assert_eq!(withdraw_checked, withdraw_unchecked);

        let withdraw_json = include_str!("../tests/test_complex_bond_and_unbond/withdraw.json");
        let withdraw_checked = decode_withdraw_mapping::<CurrentNetwork>(withdraw_json).unwrap();
        let withdraw_unchecked = decode_withdraw_mapping_unchecked(withdraw_json).unwrap();
        assert_eq!(withdraw_checked, withdraw_unchecked);

        let withdraw_json = include_str!("../tests/test_empty_block/withdraw.json");
        let withdraw_checked = decode_withdraw_mapping::<CurrentNetwork>(withdraw_json).unwrap();
        let withdraw_unchecked = decode_withdraw_mapping_unchecked(withdraw_json).unwrap();
        assert_eq!(withdraw_checked, withdraw_unchecked);
    }
}

use anyhow::{bail, Error, Result};

use snarkvm::prelude::{
    Address, Block, Identifier, Input, Literal, Network, Plaintext, ProgramID, Transactions, Value,
};
use std::collections::HashMap;
use std::str::FromStr;

pub type TransactionAmounts<N> = HashMap<<N as Network>::TransactionID, (Address<N>, u64)>;
pub type BondedMapping<N> = HashMap<Address<N>, (Address<N>, u64)>;

/*
This function reads in the json form of an Aleo block
Returns a Result of Transaction<N> where N is the generic type for a given Network
Examples are: TestnetV0, MainnetV0
 */
pub fn gather_block_transactions<N: Network>(json: &str) -> Result<Transactions<N>> {
    let block: serde_json::Result<Block<N>> = serde_json::from_str(json);

    match block {
        Ok(value) => {
            // this may be [] due to some blocks with zero transactions
            Ok(value.transactions().clone())
        }
        // throw anyhow result and bail
        Err(e) => {
            bail!("Unable to parse Block object - {}", e);
        }
    }
}

/*
Iterates over all transactions in a block to calculate their respective values
If the transition calls credits.aleo program, then we evaluate specific function calls
These calls are (unbond_public and claim_unbond_public)
Returns a HashMap<TransactionID, (Address, u64)>
 */
pub fn process_block_transactions<N: Network>(
    bonded_mapping: &str,
    unbonding_mapping: &str,
    withdraw_mapping: &str,
    block: &str,
) -> Result<TransactionAmounts<N>> {
    // Initialize the bonded mapping.
    let mut bonded_map = deserialize_bonded_mapping::<N>(bonded_mapping)?;
    // Initialize the unbonding mapping.
    let mut unbonding_map =
        deserialize_unbonding_mapping::<N>(unbonding_mapping).unwrap_or_default();
    // Initialize the withdraw mapping.
    let withdraw_map = deserialize_withdraw_mapping::<N>(withdraw_mapping).unwrap_or_default();
    // resulting map to return
    let mut tx_balances: HashMap<N::TransactionID, (Address<N>, u64)> = HashMap::new();
    let block_txs: Transactions<N> = gather_block_transactions::<N>(block)?;

    // Iterate over transactions - check internal state, update, continue
    for tx in block_txs.executions() {
        for transition in tx.transitions() {
            if transition.program_id() == &ProgramID::from_str("credits.aleo")? {
                match transition.function_name().to_string().as_str() {
                    "bond_public" => {
                        // Get the inputs.
                        let inputs = transition.inputs();

                        // Check that the number of inputs is correct.
                        if inputs.len() != 3 {
                            bail!("Incorrect number of inputs");
                        }

                        // Get the validator address from the inputs.
                        let validator_address = get_address_from_input(&inputs[0])?;
                        // Get the bond amount from the inputs.
                        let bond_amount = get_u64_from_input(&inputs[2])?;
                        // Update the bonded mapping.
                        bonded_map
                            .entry(validator_address)
                            .and_modify(|(_, microcredits)| *microcredits += bond_amount)
                            .or_insert((validator_address, bond_amount));
                    }
                    "unbond_public" => {
                        // Get the inputs.
                        let inputs = transition.inputs();

                        // Check that the number of inputs is correct.
                        if inputs.len() != 2 {
                            bail!("Incorrect number of inputs");
                        }

                        // Get the staker address from the inputs.
                        let staker_address = get_address_from_input(&inputs[0])?;
                        // Get the unbond amount from the inputs.
                        let unbond_amount = get_u64_from_input(&inputs[1])?;

                        // Get the bond state for the staker address.
                        let bond_state = bonded_map
                            .get(&staker_address)
                            .ok_or(Error::msg("Failed to get bond state"))?;

                        // Get the threshold for unbonding.
                        // If the staker is a validator, the threshold is 10_000_000_000 microcredits.
                        // Otherwise, the threshold is 10_000_000_000 microcredits.
                        let threshold = if staker_address == bond_state.0 {
                            10_000_000_000_000u64
                        } else {
                            10_000_000_000u64
                        };

                        // Get the previous bonded amount for the staker address.
                        let (previous_validator, previous_bonded) = *bonded_map
                            .get(&staker_address)
                            .unwrap_or(&(staker_address, 0u64));

                        // If the new bonded amount is less than the threshold, unbond the entire amount.
                        // Otherwise, unbond the specified amount.
                        if previous_bonded - unbond_amount < threshold {
                            // Update the unbonding mapping.
                            unbonding_map
                                .entry(staker_address)
                                .and_modify(|x| *x += previous_bonded)
                                .or_insert(previous_bonded);
                            // Update the bonded mapping.
                            bonded_map.insert(staker_address, (previous_validator, 0));
                            // Update the transaction balances.
                            tx_balances.insert(tx.id(), (staker_address, previous_bonded));
                        } else {
                            // Update the unbonding mapping.
                            unbonding_map
                                .entry(staker_address)
                                .and_modify(|x| *x += unbond_amount)
                                .or_insert(unbond_amount);
                            // Update the bonded mapping.
                            bonded_map.insert(
                                staker_address,
                                (previous_validator, previous_bonded - unbond_amount),
                            );
                            // Update the transaction balances.
                            tx_balances.insert(tx.id(), (staker_address, unbond_amount));
                        }
                    }
                    "claim_unbond_public" => {
                        // Get the inputs.
                        let inputs = transition.inputs();

                        // Check that the number of inputs is correct.
                        if inputs.len() != 1 {
                            bail!("Incorrect number of inputs");
                        }

                        // Get the staker address from the inputs.
                        let staker_address = get_address_from_input(&inputs[0])?;
                        // Get the withdrawal address from the withdraw mapping.
                        let withdrawal_address = withdraw_map
                            .get(&staker_address)
                            .ok_or(Error::msg("Failed to get withdrawal address"))?;
                        // Get the claim amount from the unbonding mapping.
                        let claim_amount = Ok(unbonding_map.get(&staker_address))
                            .and_then(|x| x.ok_or(Error::msg("Failed to get claim amount")))?;

                        // Update the transaction balances.
                        tx_balances.insert(tx.id(), (*withdrawal_address, *claim_amount));
                    }
                    _ => continue,
                }
            }
        }
    }
    // output the updated map
    Ok(tx_balances)
}

// A helper function to get an address from an input.
fn get_address_from_input<N: Network>(input: &Input<N>) -> Result<Address<N>> {
    match input {
        Input::Public(_, Some(Plaintext::Literal(Literal::Address(address), _))) => Ok(*address),
        _ => bail!("Failed to extract address"),
    }
}

// A helper function to get an u64 from an input.
fn get_u64_from_input<N: Network>(input: &Input<N>) -> Result<u64> {
    match input {
        Input::Public(_, Some(Plaintext::Literal(Literal::U64(value), _))) => Ok(**value),
        _ => bail!("Failed to extract address"),
    }
}

// A helper function to deserialize the bonded mapping from a JSON string.
fn deserialize_bonded_mapping<N: Network>(string: &str) -> Result<BondedMapping<N>> {
    // Deserialize the JSON string into a vector of entries.
    let entries: Vec<(Plaintext<N>, Value<N>)> = serde_json::from_str(string)?;
    // Map the entries into a map of addresses to bonded amounts.
    let mapping = entries.into_iter().map(|(key, value)| {
        // Extract the address.
        let address = match key {
            Plaintext::Literal(Literal::Address(address), _) => address,
            _ => bail!("Failed to extract address info"),
        };
        // Extract the validator and microcredits.
        let data = match value {
            Value::Plaintext(Plaintext::Struct(members, _)) => {
                let validator = match members.get(&Identifier::from_str("validator")?) {
                    Some(Plaintext::Literal(Literal::Address(address), _)) => *address,
                    _ => bail!("Failed to extract validator"),
                };
                let microcredits = match members.get(&Identifier::from_str("microcredits")?) {
                    Some(Plaintext::Literal(Literal::U64(microcredits), _)) => **microcredits,
                    _ => bail!("Failed to extract microcredits"),
                };
                (validator, microcredits)
            }
            _ => bail!("Failed to extract validator address"),
        };
        Ok((address, data))
    });
    mapping.collect()
}

// A helper function to deserialize the unbonding mapping from a JSON string.
fn deserialize_unbonding_mapping<N: Network>(string: &str) -> Result<HashMap<Address<N>, u64>> {
    // Deserialize the JSON string into a vector of entries.
    let entries: Vec<(Plaintext<N>, Value<N>)> = serde_json::from_str(string)?;
    // Map the entries into a map of addresses to bonded amounts.
    let mapping = entries.into_iter().map(|(key, value)| {
        // Extract the address.
        let address = match key {
            Plaintext::Literal(Literal::Address(address), _) => address,
            _ => bail!("Failed to extract address info"),
        };
        // Extract the microcredits.
        let microcredits = match value {
            Value::Plaintext(Plaintext::Struct(members, _)) => {
                let value = members.get(&Identifier::from_str("microcredits")?);
                match value {
                    Some(Plaintext::Literal(Literal::U64(value), _)) => **value,
                    _ => bail!("Failed to extract bond amount"),
                }
            }
            _ => bail!("Failed to extract bond amount"),
        };
        Ok((address, microcredits))
    });
    mapping.collect()
}

// A helper function to deserialize the withdraw mapping from a JSON string.
fn deserialize_withdraw_mapping<N: Network>(
    string: &str,
) -> Result<HashMap<Address<N>, Address<N>>> {
    // Deserialize the JSON string into a vector of entries.
    let entries: Vec<(Plaintext<N>, Value<N>)> = serde_json::from_str(string)?;
    // Map the entries into a map of addresses to bonded amounts.
    let mapping = entries.into_iter().map(|(key, value)| {
        // Extract the staker address.
        let staker_address = match key {
            Plaintext::Literal(Literal::Address(address), _) => address,
            _ => bail!("Failed to extract staker address"),
        };
        // Extract the withdrawal address.
        let withdrawal_address = match value {
            Value::Plaintext(Plaintext::Literal(Literal::Address(address), _)) => address,
            _ => bail!("Failed to extract withdrawal address"),
        };
        Ok((staker_address, withdrawal_address))
    });
    mapping.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::prelude::TestnetV0;
    use std::fs::File;
    use std::io::Read;
    use std::string::String;

    type CurrentNetwork = TestnetV0;

    #[test]
    fn read_block_with_zero_txs() {
        // read in json block file from tests
        let fp = "tests/test_empty_block/block.json";
        let mut file = File::open(fp).expect("Failed to open file");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to process json");
        // pass in block with zero transactions
        let test = gather_block_transactions::<CurrentNetwork>(&buffer).unwrap();
        assert_eq!(test.len(), 0)
    }

    #[test]
    fn test_read_bonded_mapping() {
        // read in json block file from tests
        let fp = "tests/test_empty_block/bonded.json";
        let mut file = File::open(fp).expect("Failed to open file");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to process json");
        assert!(deserialize_bonded_mapping::<CurrentNetwork>(&buffer).is_ok());
    }

    #[test]
    fn test_read_unbonding_mapping() {
        // read in json block file from tests
        let fp = "tests/test_empty_block/unbonding.json";
        let mut file = File::open(fp).expect("Failed to open file");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to process json");
        assert!(deserialize_unbonding_mapping::<CurrentNetwork>(&buffer).is_ok());
    }

    #[test]
    fn test_read_withdraw_mapping() {
        // read in json block file from tests
        let fp = "tests/test_empty_block/withdraw.json";
        let mut file = File::open(fp).expect("Failed to open file");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to process json");
        assert!(deserialize_withdraw_mapping::<CurrentNetwork>(&buffer).is_ok());
    }

    #[test]
    fn test_fill_map() {
        // read in json block file from tests
        let fp = "tests/test_bond_public/block.json";
        let mut file = File::open(fp).expect("Failed to open file");
        let mut block_buffer = String::new();
        file.read_to_string(&mut block_buffer)
            .expect("Failed to process json");

        let bonded_fp = "tests/test_bond_public/bonded.json";
        let mut bonded_file = File::open(bonded_fp).expect("Failed to open file");
        let mut bonded_buffer = String::new();
        bonded_file
            .read_to_string(&mut bonded_buffer)
            .expect("Failed to process json");

        let result_map =
            process_block_transactions::<CurrentNetwork>(&bonded_buffer, "", "", &block_buffer)
                .unwrap();
        assert!(!result_map.is_empty())
    }

    #[test]
    fn test_complex_mapping() {
        // read in json block file from tests
        let fp = "tests/test_complex_bond_and_unbond/block.json";
        let mut file = File::open(fp).expect("Failed to open file");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to process json");

        let bonded_fp = "tests/test_complex_bond_and_unbond/bonded.json";
        let mut bonded_file = File::open(bonded_fp).expect("Failed to open file");
        let mut bonded_buffer = String::new();
        bonded_file
            .read_to_string(&mut bonded_buffer)
            .expect("Failed to process json");

        let unbonded_fp = "tests/test_complex_bond_and_unbond/bonded.json";
        let mut unbonded_file = File::open(unbonded_fp).expect("Failed to open file");
        let mut unbonded_buffer = String::new();
        unbonded_file
            .read_to_string(&mut unbonded_buffer)
            .expect("Failed to process json");

        let withdraw_fp = "tests/test_complex_bond_and_unbond/withdraw.json";
        let mut withdraw_file = File::open(withdraw_fp).expect("Failed to open file");
        let mut withdraw_buffer = String::new();
        withdraw_file
            .read_to_string(&mut withdraw_buffer)
            .expect("Failed to process json");

        let result_map = process_block_transactions::<CurrentNetwork>(
            &bonded_buffer,
            &unbonded_buffer,
            &withdraw_buffer,
            &buffer,
        )
        .unwrap();
        assert_eq!(result_map.iter().next().unwrap().1 .1, 15000000u64)
    }
}

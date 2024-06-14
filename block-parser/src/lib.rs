use anyhow::{bail, Error, Result};

use snarkvm::prelude::integer_type::CheckedAbs;
use snarkvm::prelude::{Address, bech32, Block, FromBytes, Identifier, Input, Itertools, Literal, Network, Output, Parser, Plaintext, ProgramID, TestnetV0, Transactions, Transition, Value};
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::Map;
use std::ops::{Add, Deref};
use std::slice::Iter;
use std::str::FromStr;

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
These calls are (bond_public, unbond_public, and claim_unbond_public)
Returns a HashMap<TransactionID, (Address, u64)>
 */
pub fn process_block_transactions<N: Network>(
    bonded_mapping: &str,
    unbonding_mapping: &str,
    block: &str,
) -> HashMap<N::TransactionID, (Address<N>, u64)> {
    // Initialize the bonded mapping.
    let mut bonded_map = deserialize_mapping::<N>(bonded_mapping).unwrap();
    // Initialize the unbonding mapping.
    let mut unbonding_map = deserialize_mapping::<N>(unbonding_mapping).unwrap_or_default();
    // resulting map to return
    let mut tx_balances: HashMap<N::TransactionID, (Address<N>, u64)> = HashMap::new();
    let block_txs: Transactions<N> = gather_block_transactions::<N>(block).unwrap();

    // Iterate over transactions - check internal state, update, continue
    for tx in block_txs.executions() {
        if tx.is_accepted() {
            for transition in tx.transitions() {
                if transition.program_id() == &ProgramID::from_str("credits.aleo").unwrap() {
                    match transition.function_name() {
                        bond if &Identifier::<N>::from_str("bond_public").unwrap() == bond => {
                            // get bonded value and add to internal state map
                            let input = transition.inputs().get(0).unwrap();
                            let input_value = transition.inputs().get(1).unwrap();

                            let address = match input {
                                Input::Public(
                                    _,
                                    Some(Plaintext::Literal(Literal::Address(address), _)),
                                ) => address,
                                _ => panic!("Unexpected"),
                            };

                            let value = match input_value {
                                Input::Public(
                                    _,
                                    Some(Plaintext::Literal(Literal::U64(value), _)),
                                ) => value,
                                _ => panic!("Caught no value"),
                            };

                            if bonded_map.get(address).is_none() {
                                bonded_map.insert(*address, **value);
                            } else {
                                // get previous value and add
                                let tmp_value = bonded_map.get(address).unwrap();
                                bonded_map.insert(*address, tmp_value + **value);
                            }
                            tx_balances.insert(tx.id(), (*address, *bonded_map.get(address).unwrap()));
                        }
                        unbond
                            if &Identifier::<N>::from_str("unbond_public").unwrap() == unbond =>
                        {
                            //todo double check this logic -- do we need to know if staked_address == validator
                            let input = transition.inputs().get(0).unwrap();
                            let input_value = transition.inputs().get(1).unwrap();
                            let staked_address = match input {
                                Input::Public(
                                    _,
                                    Some(Plaintext::Literal(Literal::Address(address), _)),
                                ) => address,
                                _ => panic!("Unexpected"),
                            };

                            let unbond_amount = match input_value {
                                Input::Public(
                                    _,
                                    Some(Plaintext::Literal(Literal::U64(value), _)),
                                ) => value,
                                _ => panic!("Caught no value"),
                            };
                            // get previous unbonding
                            let prev_unbonding = *unbonding_map.get(staked_address).unwrap();
                            let prev_bonded = bonded_map.get(staked_address).unwrap();
                            let tmp_new_amount = prev_unbonding + **unbond_amount;

                            if prev_unbonding - **unbond_amount < 10_000_000u64 {
                                unbonding_map.insert(*staked_address, tmp_new_amount);
                                bonded_map.insert(*staked_address, prev_bonded - prev_unbonding);
                                tx_balances.insert(tx.id(), (*staked_address, prev_unbonding));
                            } else {
                                let new_amount = prev_unbonding + (prev_unbonding - **unbond_amount);
                                let new_bond = prev_bonded - (prev_unbonding - **unbond_amount);
                                unbonding_map.insert(*staked_address, new_amount);
                                bonded_map.insert(*staked_address, new_bond);
                                tx_balances.insert(tx.id(), (*staked_address, **unbond_amount));
                            }

                        }
                        claim
                            if &Identifier::<N>::from_str("claim_unbond_public").unwrap()
                                == claim =>
                        {
                            let input = transition.inputs().get(0).unwrap();
                            let claim_address = match input {
                                Input::Public(
                                    _,
                                    Some(Plaintext::Literal(Literal::Address(address), _)),
                                ) => address,
                                _ => panic!("Unexpected"),
                            };
                            let claim_amount = unbonding_map.get(claim_address).unwrap_or(&0u64);
                            tx_balances.insert(tx.id(), (*claim_address, *claim_amount));
                        }
                        _ => continue,
                    }
                }
            }
        }
    }
    // output the updated map
    println!("Block State is: {:?}", tx_balances);
    tx_balances
}

// A helper function to deserialize a mapping from a JSON string.
// Note that this function can be used for both the bonded and unbonding mappings.
// This is because both mappings use addresses as keys and use structs (`bond_state` and `unbond_state`)
// containing a `microcredits` field as values.
fn deserialize_mapping<N: Network>(string: &str) -> Result<HashMap<Address<N>, u64>> {
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
                let value = members
                    .get(&Identifier::from_str("microcredits").unwrap())
                    .unwrap();
                match value {
                    Plaintext::Literal(Literal::U64(value), _) => **value,
                    _ => bail!("Failed to extract bond amount"),
                }
            }
            _ => bail!("Failed to extract bond amount"),
        };
        Ok((address, microcredits))
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

    #[test]
    fn read_block_with_zero_txs() {
        // read in json block file from tests
        let fp = "tests/test_empty_block/block.json";
        let mut file = File::open(fp).expect("Failed to open file");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to process json");
        // pass in block with zero transactions
        let test = gather_block_transactions::<TestnetV0>(&buffer).unwrap();
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
        assert!(deserialize_mapping::<TestnetV0>(&buffer).is_ok());
    }

    #[test]
    fn test_read_unbonding_mapping() {
        // read in json block file from tests
        let fp = "tests/test_empty_block/unbonding.json";
        let mut file = File::open(fp).expect("Failed to open file");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to process json");
        assert!(deserialize_mapping::<TestnetV0>(&buffer).is_ok());
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
        bonded_file.read_to_string(&mut bonded_buffer)
            .expect("Failed to process json");
        let result_map = process_block_transactions::<TestnetV0>(&bonded_buffer, "", &block_buffer);
        assert!(!result_map.is_empty())
    }
}

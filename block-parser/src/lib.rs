use anyhow::{bail, Error, Result};

use snarkvm::prelude::integer_type::CheckedAbs;
use snarkvm::prelude::{
    Address, Block, FromBytes, Identifier, Input, Itertools, Literal, Network, Output, Parser,
    Plaintext, ProgramID, TestnetV0, Transactions, Transition, Value,
};
use std::collections::HashMap;
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
    let mut unbonding_map = deserialize_mapping::<N>(unbonding_mapping).unwrap();
    // resulting map to return
    let tx_values: HashMap<N::TransactionID, (Address<N>, u64)> = HashMap::new();
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
                        }
                        unbond
                            if &Identifier::<N>::from_str("unbond_public").unwrap() == unbond =>
                        {
                            //todo evaluate unbonding amount based on previous bonded/unbonding actions in block & block-1 mapping
                            println!("2 {:?}", unbond)
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
                            //todo: get the claim value for the address from the unbonding mapping by passing it in
                            calculate_mappings::<N>(bonded_mapping, unbonding_mapping, "")
                        }
                        _ => continue,
                    }
                }
            }
        }
    }
    // output the updated map
    println!("INTERNAL STATE: {:?}", bonded_map);
    tx_values
}

/*
Flow of calculation:
    - Pass in function call, prev_bonded_map, prev_unbond_map, address
 */
fn calculate_mappings<N>(
    prev_blocks_bonded: &str,
    prev_blocks_unbonding: &str,
    claim_address: &str,
) {
    let bonded: Result<Vec<(Plaintext<TestnetV0>, Value<TestnetV0>)>, serde_json::Error> =
        serde_json::from_str(prev_blocks_bonded);
    let unbonding: Result<Vec<(Plaintext<TestnetV0>, Value<TestnetV0>)>, serde_json::Error> =
        serde_json::from_str(prev_blocks_unbonding);

    let bondings = bonded.unwrap();
    let unbondings = unbonding.unwrap_or(Vec::new());

    //todo process the bonding/unbonding mappings into IndexMap to allow easy lookup by Address<N>

    // let address_bonded_amount = bondings.map(|(key, value)| {
    //     let address_rep: Result<(Address<TestnetV0>, Value<TestnetV0>), anyhow::Error> = match key {
    //          Plaintext::Literal(Literal::Address(address), _) => Ok((*address, value.clone())),
    //         _ => bail!("Failed to extract address info"),
    //     };
    //     address_rep
    // });
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
    fn check_program_calls() {
        // read in json block file from tests
        let fp = "tests/block.json";
        let mut file = File::open(fp).expect("Failed to open file");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to process json");
        // pass in block with zero transactions
        let test = process_block_transactions::<TestnetV0>("", "", &buffer);
    }

    #[test]
    fn process_bonded_map() {
        // read in json block file from tests
        let fp = "tests/bonded.json";
        let mut file = File::open(fp).expect("Failed to open file");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to process json");
        // pass in block with zero transactions
        let test = calculate_mappings::<TestnetV0>(
            &buffer,
            "",
            "aleo1s3ws5tra87fjycnjrwsjcrnw2qxr8jfqqdugnf0xzqqw29q9m5pqem2u4t",
        );
    }
}

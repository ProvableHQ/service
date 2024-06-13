use anyhow::{bail, Error, Result};

use snarkvm::prelude::{Address, Block, FromBytes, Identifier, Input, Itertools, Literal, Network, Output, Parser, Plaintext, ProgramID, TestnetV0, Transactions, Transition, Value};
use std::collections::HashMap;
use std::ops::Add;
use std::str::FromStr;
use snarkvm::prelude::integer_type::CheckedAbs;
use snarkvm::prelude::LiteralType::Address;

/*
This function reads in the json form of an Aleo block
Returns a Result of Transaction<N> where N is the generic type for a given Network
Examples are: TestnetV0, MainnetV0
 */
pub fn gather_block_transactions<N: Network>(json: &str) -> Result<Transactions<N>, anyhow::Error> {
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

pub fn process_block_transactions<N: Network>(
    bonded_mapping: &str,
    unbonded_mapping: &str,
    block: &str,
) -> HashMap<String, (String, u64)> {
    let mut internal_block_state: HashMap<&Address<N>, &u64> = HashMap::new();
    let tx_values: HashMap<String, (String, u64)> = HashMap::new();

    // let bonded = serde_json::from_str(bonded_mapping);
    // let unbonded = serde_json::from_str(unbonded_mapping);
    let block_txs: Transactions<N> = gather_block_transactions::<N>(block).unwrap();

    match block_txs.len() {
        0 => tx_values,
        _ => {
            // iterate over transactions - check internal state, update, continue
            for tx in block_txs.executions() {
                if tx.is_accepted() {
                    for transition in tx.transitions() {
                        if transition.program_id() == &ProgramID::from_str("credits.aleo").unwrap()
                        {
                            match transition.function_name() {
                                bond if &Identifier::<N>::from_str("bond_public").unwrap()
                                    == bond =>
                                {
                                    // get bonded value and add to internal state map
                                    let input = transition.inputs().get(0).unwrap();
                                    let input_value = transition.inputs().get(1).unwrap();

                                    let address = match input {
                                        Input::Public(_, Some(Plaintext::Literal(Literal::Address(address), _))) => address,
                                        _ => panic!("Unexpected")
                                    };

                                    let value = match input_value {
                                        Input::Public(_, Some(Plaintext::Literal(Literal::U64(value),_))) => value,
                                        _ => panic!("Caught no value")
                                    };

                                    if internal_block_state.get(address).is_none() {
                                        internal_block_state.insert(address, value);
                                    } else {
                                        // get previous value and add
                                        println!("{:?}", value.checked_abs());
                                        // internal_block_state.insert(address, value.add(internal_block_state.get(address).unwrap()));
                                    }
                                }
                                unbond
                                    if &Identifier::<N>::from_str("unbond_public").unwrap()
                                        == unbond =>
                                {
                                    println!("2 {:?}", unbond)
                                }
                                claim
                                    if &Identifier::<N>::from_str("claim_unbond_public")
                                        .unwrap()
                                        == claim =>
                                {
                                    // get the claim value for the address from the unbonding mapping
                                    calculate_mappings(bonded_mapping, unbonded_mapping, address)
                                }
                                _ => continue,
                            }
                        }
                    }
                }
            }
            // output the updated map
            println!("INTERNAL STATE: {:?}", internal_block_state);
            tx_values
        }
    }
}

/*
Flow of calculation:
    - Pass in function call, prev_bonded_map, prev_unbond_map, address
 */
fn calculate_mappings<N>(
    prev_blocks_bonded: &str,
    prev_blocks_unbonded: &str,
    staked_address: &str,
) {
    let bonded: Result<Vec<(Plaintext<TestnetV0>, Value<TestnetV0>)>, serde_json::Error> =
        serde_json::from_str(prev_blocks_bonded);

    let bondings = bonded.unwrap();
    let address_bonded_amount = bondings.iter().map(|(key, value)| {
        let address_rep: Result<(Address<TestnetV0>, Value<TestnetV0>), anyhow::Error> = match key {
             Address::from_str(staked_address) == Plaintext::Literal(Literal::Address(address), _) => Ok((*address, value.clone())),
            _ => bail!("Failed to extract address"),
        };
        address_rep
    });
    println!("{:?}", address_bonded_amount)

    // let unbonded: Result<Vec<(Plaintext<TestnetV0>, Value<TestnetV0>)>, serde_json::Error> = serde_json::from_str(prev_blocks_unbonded);
    // println!("{:?}", unbonded);

    //use values from one or both to calculate actual amount
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
        // read in json block file from resources
        let fp = "resources/block-1.json";
        let mut file = File::open(fp).expect("Failed to open file");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to process json");
        // pass in block with zero transactions
        let test = gather_block_transactions::<TestnetV0>(&buffer).unwrap();
        // assert_eq!(test.len(), 0)
    }

    #[test]
    fn check_program_calls() {
        // read in json block file from resources
        let fp = "resources/block-1.json";
        let mut file = File::open(fp).expect("Failed to open file");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to process json");
        // pass in block with zero transactions
        let test = process_block_transactions::<TestnetV0>("", "", &buffer);
    }

    #[test]
    fn process_bonded_map() {
        // read in json block file from resources
        let fp = "resources/block-1-bonded.json";
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

extern crate core;

pub mod address_bytes;
pub use address_bytes::*;

pub mod decoders;
pub use decoders::*;

use snarkvm::prelude::SizeInBytes;
use snarkvm::prelude::{
    Address, Block, FromBytes, Identifier, Input, Literal, Network, Plaintext, ProgramID,
    Transactions, Value,
};

use anyhow::{bail, ensure, Error, Result};
use std::cell::OnceCell;
use std::collections::HashMap;
use std::io::{Read, Result as IoResult};
use std::str::FromStr;

pub type TransactionAmounts<N> = HashMap<<N as Network>::TransactionID, (AddressBytes<N>, u64)>;

/*
Iterates over all transactions in a block to calculate their respective values
If the transition calls credits.aleo program, then we evaluate specific function calls
These calls are (unbond_public and claim_unbond_public)
Returns a HashMap<TransactionID, (Address, u64)>
 */
pub fn process_block_transactions<N: Network, R: Read>(
    bonded_bytes_le: R,
    unbonding_bytes_le: R,
    withdraw_bytes_le: R,
    block_bytes_le: R,
) -> Result<TransactionAmounts<N>> {
    // Decode the bonded mapping from little-endian bytes.
    let mut bonded_map = decode_bonded_mapping::<N, _>(bonded_bytes_le)?;
    // Decode the unbonding mapping from little-endian bytes.
    let mut unbonding_map = decode_unbonding_mapping::<N, _>(unbonding_bytes_le)?;
    // Decode the withdraw mapping from little-endian bytes.
    let withdraw_map = decode_withdraw_mapping::<N, _>(withdraw_bytes_le)?;

    // Initialize the transaction amounts.
    let mut tx_balances: TransactionAmounts<N> = HashMap::new();

    // Decode the block.
    let block = decode_block(block_bytes_le)?;

    // Get the block transactions.
    let block_txs = block.transactions();

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
                            .entry(validator_address.clone())
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
                        let (previous_validator, previous_bonded) = bonded_map
                            .get(&staker_address)
                            .cloned()
                            .unwrap_or((staker_address.clone(), 0u64));

                        // Calculate the new height.
                        let new_height = block.height() + 360;

                        // If the new bonded amount is less than the threshold, unbond the entire amount.
                        // Otherwise, unbond the specified amount.
                        if previous_bonded - unbond_amount < threshold {
                            // Update the unbonding mapping.
                            unbonding_map
                                .entry(staker_address.clone())
                                .and_modify(|(amount, height)| {
                                    *amount += previous_bonded;
                                    *height = new_height
                                })
                                .or_insert((previous_bonded, new_height));
                            // Update the bonded mapping.
                            bonded_map.insert(staker_address.clone(), (previous_validator, 0));
                            // Update the transaction balances.
                            tx_balances.insert(tx.id(), (staker_address, previous_bonded));
                        } else {
                            // Update the unbonding mapping.
                            unbonding_map
                                .entry(staker_address.clone())
                                .and_modify(|(amount, height)| {
                                    *amount += unbond_amount;
                                    *height = new_height;
                                })
                                .or_insert((unbond_amount, new_height));
                            // Update the bonded mapping.
                            bonded_map.insert(
                                staker_address.clone(),
                                (previous_validator, previous_bonded - unbond_amount),
                            );
                            // Update the transaction balances.
                            tx_balances.insert(tx.id(), (staker_address.clone(), unbond_amount));
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
                        tx_balances.insert(tx.id(), (withdrawal_address.clone(), claim_amount.0));
                    }
                    _ => continue,
                }
            }
        }
    }
    // output the updated map
    Ok(tx_balances)
}

// A helper function to get an address from an input as `AddressBytes`.
fn get_address_from_input<N: Network>(input: &Input<N>) -> Result<AddressBytes<N>> {
    match input {
        Input::Public(_, Some(Plaintext::Literal(Literal::Address(address), _))) => {
            AddressBytes::from_address(*address)
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::prelude::{CanaryV0, TestnetV0};

    type CurrentNetwork = CanaryV0;

    // #[test]
    // fn read_block_with_zero_txs() {
    //     // read in json block file from tests
    //     let fp = "tests/test_empty_block/block.json";
    //     let mut file = File::open(fp).expect("Failed to open file");
    //     let mut buffer = String::new();
    //     file.read_to_string(&mut buffer)
    //         .expect("Failed to process json");
    //     // pass in block with zero transactions
    //     let test = gather_block_transactions::<CurrentNetwork>(&buffer).unwrap();
    //     assert_eq!(test.len(), 0)
    // }

    // #[test]
    // fn test_read_bonded_mapping() {
    //     // Read in bonded mapping bytes from tests
    //     let fp = "tests/test_empty_block/bonded";
    //     let mut file = File::open(fp).expect("Failed to open file");
    //     let mut buffer = Vec::new();
    //     file.read(&mut buffer)
    //         .expect("Failed to read bytes");
    //     assert!(decode_bonded_mapping::<CurrentNetwork>(&buffer).is_ok());
    // }
    //
    // #[test]
    // fn test_read_unbonding_mapping() {
    //     // read in json block file from tests
    //     let fp = "tests/test_empty_block/unbonding.json";
    //     let mut file = File::open(fp).expect("Failed to open file");
    //     let mut buffer = String::new();
    //     file.read_to_string(&mut buffer)
    //         .expect("Failed to process json");
    //     assert!(deserialize_unbonding_mapping::<CurrentNetwork>(&buffer).is_ok());
    // }
    //
    // #[test]
    // fn test_read_withdraw_mapping() {
    //     // read in json block file from tests
    //     let fp = "tests/test_empty_block/withdraw.json";
    //     let mut file = File::open(fp).expect("Failed to open file");
    //     let mut buffer = String::new();
    //     file.read_to_string(&mut buffer)
    //         .expect("Failed to process json");
    //     assert!(deserialize_withdraw_mapping::<CurrentNetwork>(&buffer).is_ok());
    // }
    //
    // #[test]
    // fn test_dont_add_bonded_map() {
    //     // read in json block file from tests
    //     let fp = "tests/test_bond_public/block.json";
    //     let mut file = File::open(fp).expect("Failed to open file");
    //     let mut block_buffer = String::new();
    //     file.read_to_string(&mut block_buffer)
    //         .expect("Failed to process json");
    //
    //     let bonded_fp = "tests/test_bond_public/bonded.json";
    //     let mut bonded_file = File::open(bonded_fp).expect("Failed to open file");
    //     let mut bonded_buffer = String::new();
    //     bonded_file
    //         .read_to_string(&mut bonded_buffer)
    //         .expect("Failed to process json");
    //
    //     let result_map =
    //         process_block_transactions::<CurrentNetwork>(&bonded_buffer, "", "", &block_buffer)
    //             .unwrap();
    //     assert!(result_map.is_empty())
    // }
    //
    // #[test]
    // fn test_complex_mapping() {
    //     // read in json block file from tests
    //     let fp = "tests/test_complex_bond_and_unbond/block.json";
    //     let mut file = File::open(fp).expect("Failed to open file");
    //     let mut buffer = String::new();
    //     file.read_to_string(&mut buffer)
    //         .expect("Failed to process json");
    //
    //     let bonded_fp = "tests/test_complex_bond_and_unbond/bonded.json";
    //     let mut bonded_file = File::open(bonded_fp).expect("Failed to open file");
    //     let mut bonded_buffer = String::new();
    //     bonded_file
    //         .read_to_string(&mut bonded_buffer)
    //         .expect("Failed to process json");
    //
    //     let unbonding_fp = "tests/test_complex_bond_and_unbond/bonded.json";
    //     let mut unbonding_file = File::open(unbonding_fp).expect("Failed to open file");
    //     let mut unbonding_buffer = String::new();
    //     unbonding_file
    //         .read_to_string(&mut unbonding_buffer)
    //         .expect("Failed to process json");
    //
    //     let withdraw_fp = "tests/test_complex_bond_and_unbond/withdraw.json";
    //     let mut withdraw_file = File::open(withdraw_fp).expect("Failed to open file");
    //     let mut withdraw_buffer = String::new();
    //     withdraw_file
    //         .read_to_string(&mut withdraw_buffer)
    //         .expect("Failed to process json");
    //
    //     let result_map = process_block_transactions::<CurrentNetwork>(
    //         &bonded_buffer,
    //         &unbonding_buffer,
    //         &withdraw_buffer,
    //         &buffer,
    //     )
    //     .unwrap();
    //     assert_eq!(result_map.iter().next().unwrap().1 .1, 10000000u64)
    // }
}

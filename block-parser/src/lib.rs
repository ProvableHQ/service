extern crate core;

// This module is not functional.
// It contains prototypes for processing byte-encoded historical data.
// pub mod archive;
// pub use archive::*;

pub mod block_json;
pub use block_json::*;

pub mod decoders;
pub use decoders::*;

pub mod credits_operations;
pub use credits_operations::*;

use snarkvm::prelude::{
    Address, Block, FromBytes, Identifier, Input, Literal, Network, Plaintext, ProgramID,
    Transactions, Value,
};
use snarkvm::prelude::{SizeInBytes, U64};

use anyhow::{bail, ensure, Error, Result};
use std::cell::OnceCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::io::{Read, Result as IoResult};
use std::marker::PhantomData;
use std::str::FromStr;

pub type BondedMapping = HashMap<String, (String, u64)>;
pub type UnbondedMapping = HashMap<String, (u64, u32)>;
pub type WithdrawMapping = HashMap<String, String>;
pub type TransactionAmounts = HashMap<String, (String, u64)>;

/*
Iterates over all transactions in a block to calculate their respective values
If the transition calls credits.aleo program, then we evaluate specific function calls
These calls are (unbond_public and claim_unbond_public)
Returns a HashMap<TransactionID, (Address, u64)>
 */
pub fn process_block_transactions(
    mut bonded_map: BondedMapping,
    mut unbonding_map: UnbondedMapping,
    withdraw_map: WithdrawMapping,
    transactions: Vec<CreditsOperations>,
    block_height: u32,
) -> Result<TransactionAmounts> {
    // Initialize the transaction amounts.
    let mut tx_balances: TransactionAmounts = HashMap::new();

    // Process the transactions and calculate the associated amounts.
    for transaction in transactions {
        match transaction {
            CreditsOperations::BondPublic { id: _, validator, withdrawal: _, amount } => {
                // Update the bonded mapping.
                bonded_map
                    .entry(validator.clone())
                    .and_modify(|(_, microcredits)| *microcredits += amount)
                    .or_insert((validator, amount));
            }
            CreditsOperations::ClaimUnbondPublic { id, staker } => {
                // Get the withdrawal address from the withdraw mapping.
                let withdrawal_address = withdraw_map
                    .get(&staker)
                    .ok_or(Error::msg("Failed to get withdrawal address"))?;
                // Get the claim amount from the unbonding mapping.
                let claim_amount = Ok(unbonding_map.get(&staker))
                    .and_then(|x| x.ok_or(Error::msg("Failed to get claim amount")))?;
                // Update the transaction balances.
                tx_balances.insert(id, (withdrawal_address.clone(), claim_amount.0));
            }
            CreditsOperations::UnbondPublic { id, staker, amount } => {
                // Get the bond state for the staker address.
                let bond_state = bonded_map
                    .get(&staker)
                    .ok_or(Error::msg("Failed to get bond state"))?;

                // Get the threshold for unbonding.
                // If the staker is a validator, the threshold is 10_000_000_000 microcredits.
                // Otherwise, the threshold is 10_000_000_000 microcredits.
                let threshold = if staker == bond_state.0 {
                    10_000_000_000_000u64
                } else {
                    10_000_000_000u64
                };

                // Get the previous bonded amount for the staker address.
                let (previous_validator, previous_bonded) = bonded_map
                    .get(&staker)
                    .cloned()
                    .unwrap_or((staker.clone(), 0u64));

                // Calculate the new height.
                let new_height = block_height + 360;

                // If the new bonded amount is less than the threshold, unbond the entire amount.
                // Otherwise, unbond the specified amount.
                if previous_bonded - amount < threshold {
                    // Update the unbonding mapping.
                    unbonding_map
                        .entry(staker.clone())
                        .and_modify(|(current_amount, current_height)| {
                            *current_amount += previous_bonded;
                            *current_height = new_height
                        })
                        .or_insert((previous_bonded, new_height));
                    // Update the bonded mapping.
                    bonded_map.insert(staker.clone(), (previous_validator, 0));
                    // Update the transaction balances.
                    tx_balances.insert(id, (staker, previous_bonded));
                } else {
                    // Update the unbonding mapping.
                    unbonding_map
                        .entry(staker.clone())
                        .and_modify(|(current_amount, current_height)| {
                            *current_amount += amount;
                            *current_height = new_height;
                        })
                        .or_insert((amount, new_height));
                    // Update the bonded mapping.
                    bonded_map.insert(
                        staker.clone(),
                        (previous_validator, previous_bonded - amount),
                    );
                    // Update the transaction balances.
                    tx_balances.insert(id, (staker.clone(), amount));
                }
            }
        }
    }
    // output the updated map
    Ok(tx_balances)
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::prelude::{CanaryV0, TestnetV0};

    use std::fs::File;

    type CurrentNetwork = CanaryV0;

    // #[test]
    // fn test_dont_add_bonded_map() {
    //     // read in utilities block file from tests
    //     let fp = "tests/test_bond_public/block.json";
    //     let mut file = File::open(fp).expect("Failed to open file");
    //     let mut block_buffer = String::new();
    //     file.read_to_string(&mut block_buffer)
    //         .expect("Failed to process JSON");
    //
    //     let bonded_fp = "tests/test_bond_public/bonded.json";
    //     let mut bonded_file = File::open(bonded_fp).expect("Failed to open file");
    //     let mut bonded_buffer = String::new();
    //     bonded_file
    //         .read_to_string(&mut bonded_buffer)
    //         .expect("Failed to process JSON");
    //
    //     let result_map =
    //         process_block_transactions(&bonded_buffer, "[]", "[]", block_buffer)
    //             .unwrap();
    //     assert!(result_map.is_empty())
    // }
    //
    // #[test]
    // fn test_complex_mapping() {
    //     // read in utilities block file from tests
    //     let fp = "tests/test_complex_bond_and_unbond/block.json";
    //     let mut file = File::open(fp).expect("Failed to open file");
    //     let mut buffer = String::new();
    //     file.read_to_string(&mut buffer)
    //         .expect("Failed to process JSON");
    //
    //     let bonded_fp = "tests/test_complex_bond_and_unbond/bonded.json";
    //     let mut bonded_file = File::open(bonded_fp).expect("Failed to open file");
    //     let mut bonded_buffer = String::new();
    //     bonded_file
    //         .read_to_string(&mut bonded_buffer)
    //         .expect("Failed to process JSON");
    //
    //     let unbonding_fp = "tests/test_complex_bond_and_unbond/unbonding.json";
    //     let mut unbonding_file = File::open(unbonding_fp).expect("Failed to open file");
    //     let mut unbonding_buffer = String::new();
    //     unbonding_file
    //         .read_to_string(&mut unbonding_buffer)
    //         .expect("Failed to process JSON");
    //
    //     let withdraw_fp = "tests/test_complex_bond_and_unbond/withdraw.json";
    //     let mut withdraw_file = File::open(withdraw_fp).expect("Failed to open file");
    //     let mut withdraw_buffer = String::new();
    //     withdraw_file
    //         .read_to_string(&mut withdraw_buffer)
    //         .expect("Failed to process JSON");
    //
    //     let result_map = process_block_transactions(
    //         &bonded_buffer,
    //         &unbonding_buffer,
    //         &withdraw_buffer,
    //         buffer,
    //     )
    //     .unwrap();
    //     assert_eq!(result_map.iter().next().unwrap().1 .1, 10000000u64)
    // }
}

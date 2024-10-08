pub mod input;
pub use input::*;

pub mod output;
pub use input::*;

pub mod transaction;
pub use transaction::*;

pub mod transition;
pub use transition::*;

use anyhow::{bail, Result};
use serde_json::{Map, Value};
use std::str::FromStr;

pub struct BlockJSON {
    // The block height.
    height: u32,
    // The transactions in the block.
    transactions: Vec<TransactionJSON>,
    // The block timestamp.
    timestamp: u64,
}

impl BlockJSON {
    // Constructs a new `BlockJSON` from a JSON string.
    // Note that this method does not validate the block.
    pub fn new(string: String) -> Result<Self> {
        // Construct the JSON value.
        let json = match Value::from_str(&string)? {
            Value::Object(json) => json,
            _ => bail!("Invalid JSON object"),
        };
        Self::try_from(json)
    }

    // Returns the block height.
    pub fn height(&self) -> u32 {
        self.height
    }

    // Returns the transactions in the block.
    pub fn transactions(&self) -> &Vec<TransactionJSON> {
        &self.transactions
    }

    // Returns the block timestamp
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl TryFrom<Map<String, Value>> for BlockJSON {
    type Error = anyhow::Error;

    fn try_from(json: serde_json::Map<String, serde_json::Value>) -> std::result::Result<Self, Self::Error> {
        let metadata = json["header"]["metadata"].clone();
        // Get the block height.
        let height = match metadata["height"].as_u64() {
            Some(height) => height as u32,
            None => bail!("Invalid block height"),
        };
        // Get the transactions in the block.
        let transactions = match json["transactions"].as_array() {
            Some(transactions) => transactions
                .iter()
                .map(|transaction| TransactionJSON::new(transaction.clone()))
                .collect::<Result<Vec<_>>>()?,
            None => bail!("Invalid transactions"),
        };
        // Get the block timestamp.
        let timestamp = match metadata["timestamp"].as_u64() {
            Some(timestamp) => timestamp,
            None => bail!("Invalid block timestamp"),
        };
        Ok(Self {
            height,
            transactions,
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::Read};

    #[test]
    fn test_parse_block() {
        let fp = "tests/test_claim_unbond_public/block.json";
        let mut file = File::open(fp).expect("Failed to open file");
        let mut block_buffer = String::new();
        file.read_to_string(&mut block_buffer)
            .expect("Failed to process JSON");
        
        let block = BlockJSON::new(block_buffer).expect("Failed to parse block");

        assert_eq!(427005, block.height());

        let transactions = block.transactions();
        assert_eq!(2, transactions.len());

        let transition_cases = [
            (
                "au18r8tss4c0mv5jmg40wnhh7ftn2yq8scqcznxesddjmv8kfa85upq0r6qmh".to_string(),
                "shadowfi_demo_v2_0.aleo".to_string(),
                "mint_zec".to_string(),
                2,
                2,
            ),
            (
                "au1gedkey8yvd4csh7e44gy94pwj2qdxv27fhnk7y6e6xnxa67v8gxq69j2tz".to_string(),
                "credits.aleo".to_string(),
                "bond_public".to_string(),
                3,
                1,
            ),
        ];

        let input_cases = [
            vec![
                (
                    "370013488506471349544562947493208305264427465239212108745442488484118223169field",
                    "private",
                    Some("ciphertext1qgqp52ga8dqfwgxyzlxf9z7eh23lqaxut6wcn23czt65qjjda2c4sp205ulnj6unktqx86u5qts7dvp3el46xkhzv0lxanelycnn02y7qqfu4t78".to_string())
                ),
                (
                    "5410583067684285043330655748316814847749155787510459694716459270918129458302field",
                    "private",
                    Some("ciphertext1qyqye88msxnquuj9lpw5ett3dtsxwucj3rmkfy4p5wf7m4scz7kfupc2l9h6z".to_string()),
                ),
            ],
            vec![
                (
                    "2868901785241866086424141678842939413409003440836646844804072949717167593753field",
                    "public",
                    Some("aleo1sln3ylyratwjext23gr5a94zhce77kfwz6kap49gakl90l9vnvqstf0tmr".to_string()),
                ),
                (
                    "2868901785241866086424141678842939413409003440836646844804072949717167593757field",
                    "public",
                    Some("aleo1sln3ylyratwjext23gr5a94zhce77kfwz6kap49gakl90l9vnvqstf0tmr".to_string()),
                ),
                (
                    "7586747543489290693889554560161278218432338152879668443442786499527878025142field",
                    "public",
                    Some("15000000u64".to_string()),
                ),
            ],
        ];

        let output_cases = [
            vec![
                (
                    "3159220527437781338339361064586599427779289116476989264696833245489339805887field",
                    "record",
                    Some("5212402562618795722610694598702544813617118804451155180894519265815940275932field".to_string()),
                    Some("record1qyqspp7t8sfvm3hqftt7kgl3phrnxax5a27y8c4t4yf0ea2qrusxk9ggqgrxzmt0w4h8ggcqqgqspq4a4nv890rsg57paha05cq7knvz6s9pgj95y3a62stll58hqgcgqf5kggcqqgqspzekxhcemt39fvam94k0tmzzn3svun4eseu7jcl3mjuvpt3mkyg2e8gnme2799q7h2hpx5990pmzp9hc4095fup650280cfq8229zs8s862lvl".to_string()),
                ),
                (
                    "2675031785265079585951940705434476409782237186142533639720460499355149128536field",
                    "future",
                    None,
                    Some("{\n  program_id: shadowfi_demo_v2_0.aleo,\n  function_name: mint_zec,\n  arguments: [\n    1000u64\n  ]\n}".to_string()),
                ),
            ],
            vec![
                (
                    "1873042494702374227156396392542195153958532664130815218448998614066833644326field",
                    "future",
                    None,
                    Some("{\n  program_id: credits.aleo,\n  function_name: transfer_public,\n  arguments: [\n    aleo1ca08xgn2zfelvpdgc90wrs7tkh0m7ud7ltkx007ldep2t3v86vys2jcads,\n    aleo1sln3ylyratwjext23gr5a94zhce77kfwz6kap49gakl90l9vnvqstf0tmr,\n    15000000u64\n  ]\n}".to_string()),
                ),
            ],
        ];

        for (i, transaction) in transactions.iter().enumerate() {
            assert!(transaction.is_accepted());
            assert!(transaction.is_execute());

            let transitions = transaction.transitions();
            assert_eq!(1, transitions.len());


            let (id, program, function, inputs_len, outputs_len) = &transition_cases[i];
            assert_eq!(id, transitions[0].id());
            assert_eq!(program, transitions[0].program_id());
            assert_eq!(function, transitions[0].function_name());

            let inputs = transitions[0].inputs();
            assert_eq!(*inputs_len, inputs.len());

            let outputs = transitions[0].outputs();
            assert_eq!(*outputs_len, outputs.len());

            for (j, input) in transitions[0].inputs().iter().enumerate() {
                let (input_id, input_type, input_value) = &input_cases[i][j];
                assert_eq!(*input_id, input.id());
                assert_eq!(*input_type, input.input_type());
                assert_eq!(input_value, input.value());
            }

            for (j, output) in transitions[0].outputs().iter().enumerate() {
                let (output_id, output_type, output_checksum, output_value) = &output_cases[i][j];
                assert_eq!(*output_id, output.id());
                assert_eq!(*output_type, output.output_type());
                assert_eq!(output_checksum, output.checksum());
                assert_eq!(output_value, output.value());
            }
        }
    }
}
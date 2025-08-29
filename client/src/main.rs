// Copyright (C) 2019-2024 Aleo Systems Inc.
// This file is part of the Aleo SDK library.

// The Aleo SDK library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Aleo SDK library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Aleo SDK library. If not, see <https://www.gnu.org/licenses/>.

use snarkvm::ledger::block::Transaction;
use snarkvm::prelude::{
    Field, FromBytes, Identifier, Literal, Network, PrivateKey, ProgramID, Request, ToBytes,
    Uniform, Value, ValueType, U64,
};

use authorize_service::*;
use execute_service::*;

use anyhow::{bail, Result};
use clap::Parser;
use rand_chacha::rand_core::SeedableRng;
use reqwest::Client;
use std::str::FromStr;

const KEYGEN_URL: &str = "http://localhost:8080/keygen";
const AUTHORIZE_URL: &str = "http://localhost:8080/authorize";
const AUTHORIZE_SIGNED_URL: &str = "http://localhost:8080/authorize_signed";
const EXECUTE_URL: &str = "http://localhost:8081/execute";

const BROADCAST_URL: &str = "http://localhost:3033/canary/transaction/broadcast";
const STATE_ROOT_URL: &str = "http://localhost:3033/canary/stateRoot/latest";

const DEVNET_PRIVATE_KEY: &str = "APrivateKey1zkp8CZNn3yeCseEtxuVPbDCwSyhGW6yZKUYKfgXmcpoGPWH";

type CurrentNetwork = snarkvm::prelude::CanaryV0;

/// Command-line arguments parser
#[derive(Parser)]
struct Args {
    /// Determine whether or not to let the client generate the request.
    #[arg(long, default_value = "false")]
    generate_requests: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Create a `Client` instance.
    let client = Client::new();

    // Initialize a random number generator.
    let rng = &mut rand_chacha::ChaCha20Rng::from_entropy();

    // Generate a seed.
    let seed = Field::<CurrentNetwork>::rand(rng);

    // Construct the url for the keygen request.
    let url = format!("{}/{}", KEYGEN_URL, *seed);

    // Send the request.
    let response = client.get(url).send().await?;

    // If the request was successful, deserialize the response as a `PrivateKey`.
    let _private_key = match response.status().is_success() {
        true => {
            let response = response.json::<KeygenResponse>().await?;
            response.private_key
        }
        false => bail!("Keygen request failed with status: {}", response.status()),
    };

    // Use the `DEVNET_PRIVATE_KEY`, if desired.
    let private_key = PrivateKey::<CurrentNetwork>::from_str(DEVNET_PRIVATE_KEY)?;

    println!("Using private key: {}", private_key);

    // Construct the recipient address.
    let recipient =
        Value::from_str("aleo16y9l270rdyun3tpfqjppj7hmvtwc03tl852q4v7fddfrus9ansrqsv35x7")?;
    // Construct the amount.
    let amount_in_microcredits = Value::from_str("100u64")?;
    // Construct the base fee.
    let base_fee_in_microcredits = U64::new(300_000); // TODO: Use a more precise fee.
                                                      // Construct the priority fee.
    let priority_fee_in_microcredits = U64::new(10);

    // Determine whether the authorization service will sign the request.
    if !args.generate_requests {
        sign_authorize_execute(
            &client,
            private_key,
            recipient,
            amount_in_microcredits,
            base_fee_in_microcredits,
            priority_fee_in_microcredits,
        )
        .await
    } else {
        authorize_execute(
            &client,
            private_key,
            recipient,
            amount_in_microcredits,
            base_fee_in_microcredits,
            priority_fee_in_microcredits,
            rng,
        )
        .await
    }
}

/// Sends an `AuthorizeRequest` to the authorization service and executes the transaction.
async fn sign_authorize_execute(
    client: &Client,
    private_key: PrivateKey<CurrentNetwork>,
    recipient: Value<CurrentNetwork>,
    amount_in_microcredits: Value<CurrentNetwork>,
    base_fee_in_microcredits: U64<CurrentNetwork>,
    priority_fee_in_microcredits: U64<CurrentNetwork>,
) -> Result<()> {
    // Construct the inputs for the request.
    let program_id = ProgramID::from_str("credits.aleo")?;
    let function_name = Identifier::from_str("transfer_public")?;
    let inputs = vec![recipient, amount_in_microcredits];

    // Construct an `AuthorizeRequest`.
    let authorize_request = AuthorizeRequest::<CurrentNetwork> {
        private_key,
        program_id,
        function_name,
        inputs,
        base_fee_in_microcredits,
        priority_fee_in_microcredits,
    };

    // Send the request.
    let response = client
        .post(AUTHORIZE_URL)
        .json(&authorize_request)
        .send()
        .await?;

    // If the request was successful, deserialize the response as an `AuthorizeResponse`.
    let authorize_response = match response.status().is_success() {
        true => response.json::<AuthorizeResponse<CurrentNetwork>>().await?,
        false => bail!(
            "Authorization request failed with status: {}",
            response.status()
        ),
    };

    // Get the latest state root.
    let response = client.get(STATE_ROOT_URL).send().await?;

    // If the request was successful, deserialize the response JSON as a `StateRoot`.
    let state_root = match response.status().is_success() {
        true => {
            response
                .json::<<CurrentNetwork as Network>::StateRoot>()
                .await?
        }
        false => bail!(
            "State root request failed with status: {}",
            response.status()
        ),
    };

    println!("Using state root: {}", state_root);

    // Construct an `ExecuteRequest`.
    let execute_request = ExecuteRequest::<CurrentNetwork> {
        function_authorization: authorize_response.function_authorization,
        fee_authorization: authorize_response.fee_authorization,
        state_root: Some(state_root),
        state_path: None,
    };

    // Send the request.
    let response = client
        .post(EXECUTE_URL)
        .body(execute_request.to_bytes_le()?)
        .header("Content-Type", "application/octet-stream")
        .send()
        .await?;

    // If the request was successful, deserialize the response archive as a `Transaction`.
    let transaction = match response.status().is_success() {
        true => {
            let bytes = response.bytes().await?;
            Transaction::<CurrentNetwork>::from_bytes_le(&bytes)?
        }
        false => bail!(
            "Execution request failed with status: {}",
            response.status()
        ),
    };

    // Send the transaction as a broadcast request as JSON.
    let response = client.post(BROADCAST_URL).json(&transaction).send().await?;

    // If the request was successful, print the response and the response body.
    match response.status().is_success() {
        true => {
            println!(
                "Broadcast request succeeded with status: {}",
                response.status()
            );
            println!(
                "Broadcast request response body: {}",
                response.text().await?
            );
        }
        false => bail!(
            "Broadcast request failed with status: {}",
            response.status()
        ),
    }

    Ok(())
}

/// Sends an `AuthorizeSignedRequest` to the authorization service and executes the transaction.
async fn authorize_execute(
    client: &Client,
    private_key: PrivateKey<CurrentNetwork>,
    recipient: Value<CurrentNetwork>,
    amount_in_microcredits: Value<CurrentNetwork>,
    base_fee_in_microcredits: U64<CurrentNetwork>,
    priority_fee_in_microcredits: U64<CurrentNetwork>,
    rng: &mut rand_chacha::ChaCha20Rng,
) -> Result<()> {
    // Construct the inputs for the request.
    let program_id = ProgramID::from_str("credits.aleo")?;
    let function_name = Identifier::from_str("transfer_public")?;
    let is_root = true;
    let root_tvk = None;
    let inputs = vec![recipient, amount_in_microcredits];
    let input_types = [
        ValueType::from_str("address.public").unwrap(),
        ValueType::from_str("u64.public").unwrap(),
    ];
    // Compute the request.
    let request = Request::sign(
        &private_key,
        program_id,
        function_name,
        inputs.into_iter(),
        &input_types,
        root_tvk,
        is_root,
        rng,
    )?;

    // Construct an `AuthorizeSignRequest`.
    let authorize_request = AuthorizeSignedRequest::<CurrentNetwork> { request };

    // Send the request.
    let response = client
        .post(AUTHORIZE_SIGNED_URL)
        .json(&authorize_request)
        .send()
        .await?;

    // If the request was successful, deserialize the response as an `AuthorizeSignedResponse`.
    let authorize_response = match response.status().is_success() {
        true => {
            response
                .json::<AuthorizeSignedResponse<CurrentNetwork>>()
                .await?
        }
        false => bail!(
            "Authorization request failed with status: {}",
            response.status()
        ),
    };

    // NOTE: in this example, it is critical for security reasons that the
    // client computes the execution id, and checks that the tcm/scm matches the
    // request.
    let execution_id = authorize_response.authorization.to_execution_id()?;

    // Construct the inputs for the fee request.
    let program_id = ProgramID::from_str("credits.aleo")?;
    let function_name = Identifier::from_str("fee_public")?;
    let is_root = true;
    let root_tvk = None;
    let inputs = vec![
        Value::from(Literal::U64(base_fee_in_microcredits)),
        Value::from(Literal::U64(priority_fee_in_microcredits)),
        Value::from(Literal::Field(execution_id)),
    ];
    let input_types = [
        ValueType::from_str("u64.public").unwrap(),
        ValueType::from_str("u64.public").unwrap(),
        ValueType::from_str("field.public").unwrap(),
    ];
    // Compute the request.
    let request = Request::sign(
        &private_key,
        program_id,
        function_name,
        inputs.into_iter(),
        &input_types,
        root_tvk,
        is_root,
        rng,
    )?;

    // Construct an `AuthorizeSignRequest`.
    let authorize_request = AuthorizeSignedRequest::<CurrentNetwork> { request };

    // Send the request.
    let response = client
        .post(AUTHORIZE_SIGNED_URL)
        .json(&authorize_request)
        .send()
        .await?;

    // If the request was successful, deserialize the response as an `AuthorizeSignedResponse`.
    let authorize_fee_response = match response.status().is_success() {
        true => {
            response
                .json::<AuthorizeSignedResponse<CurrentNetwork>>()
                .await?
        }
        false => bail!(
            "Authorization request failed with status: {}",
            response.status()
        ),
    };

    // Get the latest state root.
    let response = client.get(STATE_ROOT_URL).send().await?;

    // If the request was successful, deserialize the response JSON as a `StateRoot`.
    let state_root = match response.status().is_success() {
        true => {
            response
                .json::<<CurrentNetwork as Network>::StateRoot>()
                .await?
        }
        false => bail!(
            "State root request failed with status: {}",
            response.status()
        ),
    };

    println!("Using state root: {}", state_root);

    // Construct an `ExecuteRequest`.
    let execute_request = ExecuteRequest::<CurrentNetwork> {
        function_authorization: authorize_response.authorization,
        fee_authorization: authorize_fee_response.authorization,
        state_root: Some(state_root),
        state_path: None,
    };

    // Send the request.
    let response = client
        .post(EXECUTE_URL)
        .body(execute_request.to_bytes_le()?)
        .header("Content-Type", "application/octet-stream")
        .send()
        .await?;

    // If the request was successful, deserialize the response archive as a `Transaction`.
    let transaction = match response.status().is_success() {
        true => {
            let bytes = response.bytes().await?;
            Transaction::<CurrentNetwork>::from_bytes_le(&bytes)?
        }
        false => bail!(
            "Execution request failed with status: {}",
            response.status()
        ),
    };

    // Send the transaction as a broadcast request as JSON.
    let response = client.post(BROADCAST_URL).json(&transaction).send().await?;

    // If the request was successful, print the response and the response body.
    match response.status().is_success() {
        true => {
            println!(
                "Broadcast request succeeded with status: {}",
                response.status()
            );
            println!(
                "Broadcast request response body: {}",
                response.text().await?
            );
        }
        false => bail!(
            "Broadcast request failed with status: {}",
            response.status()
        ),
    }

    Ok(())
}

use anyhow::Result;
use authorize_service::{AuthorizeRequest, AuthorizeResponse};
use execute_service::ExecuteRequest;
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use snarkvm::algorithms::snark::varuna::VarunaVersion;
use snarkvm::prelude::*;
use std::str::FromStr;

// Run explicitly: cargo test -p transfer-client --test transaction -- --ignored --nocapture
// Requires credits.aleo proving keys. No transaction is broadcast and no funded keys are used.
#[test]
#[ignore = "expensive proof generation; requires snarkVM proving parameters"]
fn mainnet_public_transfer_proofs() -> Result<()> {
    type N = MainnetV0;
    let rng = &mut ChaCha20Rng::from_rng(&mut rand::rng());
    let private_key = PrivateKey::<N>::new(rng)?;
    let recipient = Address::<N>::try_from(&PrivateKey::<N>::new(rng)?)?;
    let request = AuthorizeRequest {
        private_key,
        program_id: ProgramID::from_str("credits.aleo")?,
        function_name: Identifier::from_str("transfer_public")?,
        inputs: vec![
            Value::from_str(&recipient.to_string())?,
            Value::from_str("100u64")?,
        ],
        base_fee_in_microcredits: U64::new(300_000),
        priority_fee_in_microcredits: U64::new(10),
    };

    eprintln!("Authorizing through authorize-service...");
    let response = authorize_service::authorize::<N>(serde_json::to_vec(&request)?.into())?;
    let response: AuthorizeResponse<N> = serde_json::from_value(response)?;
    // A real fixture root exercises proof binding, but does not fund the fresh sender.
    let block: Block<N> = serde_json::from_str(include_str!(
        "../../block-parser/tests/test_bond_public/block.json"
    ))?;
    let state_root = block.header().previous_state_root();
    let request = ExecuteRequest {
        function_authorization: response.function_authorization,
        fee_authorization: response.fee_authorization,
        state_root: Some(state_root),
        state_path: None,
    };

    eprintln!("Trying a public transfer without a state path or state root...");
    let mut request_without_state = request.clone();
    request_without_state.state_root = None;
    let error = execute_service::execute::<N>(request_without_state.to_bytes_le()?.into())
        .expect_err("a public transfer still requires a state root");
    assert_eq!(error.to_string(), "State root is not set.");
    eprintln!("Missing-state request returned: {error}");

    eprintln!("Generating execution and fee proofs through execute-service...");
    let bytes = execute_service::execute::<N>(request.to_bytes_le()?.into())?;
    let transaction = Transaction::<N>::from_bytes_le(&bytes)?;
    assert_eq!(bytes, transaction.to_bytes_le()?);
    let execution = transaction.execution().expect("execution transaction");
    let fee = transaction.fee_transition().expect("public fee");
    assert_eq!(execution.global_state_root(), state_root);
    assert_eq!(fee.global_state_root(), state_root);
    assert_eq!(*fee.amount()?, 300_010);

    eprintln!("Verifying execution and fee under consensus V21...");
    let process = Process::<N>::load()?;
    let stacks = process.get_stacks(execution.transitions(), false)?;
    Process::<N>::verify_execution(
        ConsensusVersion::V21,
        VarunaVersion::V3,
        InclusionVersion::V1,
        execution,
        &stacks,
    )?;
    process.verify_fee(
        ConsensusVersion::V21,
        VarunaVersion::V3,
        InclusionVersion::V1,
        &fee,
        execution.to_execution_id()?,
    )?;

    eprintln!("Checking V20 rejects V3 proofs...");
    assert!(Process::<N>::verify_execution(
        ConsensusVersion::V20,
        VarunaVersion::V2,
        InclusionVersion::V1,
        execution,
        &stacks,
    )
    .is_err());
    assert!(process
        .verify_fee(
            ConsensusVersion::V20,
            VarunaVersion::V2,
            InclusionVersion::V1,
            &fee,
            execution.to_execution_id()?,
        )
        .is_err());
    eprintln!("Verified transaction {}", transaction.id());
    Ok(())
}

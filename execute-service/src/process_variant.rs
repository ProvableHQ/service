// Copyright (C) 2019-2023 Aleo Systems Inc.
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

use super::*;

pub enum ProcessVariant {
    MainnetV0(Process<MainnetV0>),
    TestnetV0(Process<TestnetV0>),
    CanaryV0(Process<CanaryV0>),
}

impl ProcessVariant {
    pub fn execute(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        match self {
            ProcessVariant::MainnetV0(process) => {
                Self::handle_execute::<AleoV0, MainnetV0>(process, bytes)
            }
            ProcessVariant::TestnetV0(process) => {
                Self::handle_execute::<AleoTestnetV0, TestnetV0>(process, bytes)
            }
            ProcessVariant::CanaryV0(process) => {
                Self::handle_execute::<AleoCanaryV0, CanaryV0>(process, bytes)
            }
        }
    }

    fn handle_execute<A: Aleo<Network = N>, N: Network>(
        process: &Process<N>,
        bytes: &[u8],
    ) -> Result<Vec<u8>> {
        // Deserialize the `ExecuteRequest`.
        let execute_request = ExecuteRequest::<N>::from_bytes_le(bytes)?;
        // Initialize an RNG.
        let rng = &mut rand_chacha::ChaCha20Rng::from_entropy();

        // Get the function authorization.
        let function_authorization = execute_request.function_authorization;
        // Get the fee authorization.
        let fee_authorization = execute_request.fee_authorization;
        // Get the state root.
        let state_root = execute_request.state_root;
        // Get the state path.
        let state_path = execute_request.state_path;

        // Construct the query.
        let query = StaticQuery::<N>::new(state_root, state_path);

        // Construct the locator of the main function.
        let locator = {
            let request = function_authorization.peek_next()?;
            Locator::new(*request.program_id(), *request.function_name()).to_string()
        };

        // Execute the function authorization.
        let (_, mut trace) = process.execute::<A, _>(function_authorization, rng)?;

        // Prepare the trace.
        trace.prepare(query.clone())?;

        // Compute the proof and construct the execution.
        let execution = trace.prove_execution::<A, _>(&locator, rng)?;

        // Execute the fee authorization.
        let (_, mut trace) = process.execute::<A, _>(fee_authorization, rng)?;

        // Prepare the trace.
        trace.prepare(query)?;

        // Compute the proof and construct the fee.
        let fee = trace.prove_fee::<A, _>(rng)?;

        // Construct the transaction.
        let transaction = Transaction::<N>::from_execution(execution, Some(fee))?;

        // Serialize the transaction.
        transaction.to_bytes_le()
    }
}

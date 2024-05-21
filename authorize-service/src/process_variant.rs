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
}

impl ProcessVariant {
    pub fn authorize(&self, bytes: &[u8]) -> Result<Value> {
        match self {
            ProcessVariant::MainnetV0(process) => {
                Self::handle_authorize::<AleoV0, MainnetV0>(process, bytes)
            }
            ProcessVariant::TestnetV0(process) => {
                Self::handle_authorize::<AleoTestnetV0, TestnetV0>(process, bytes)
            }
        }
    }

    fn handle_authorize<A: Aleo<Network = N>, N: Network>(
        process: &Process<N>,
        bytes: &[u8],
    ) -> Result<Value> {
        // Deserialize the request.
        let request = serde_json::from_slice::<AuthorizeRequest<N>>(bytes)?;

        // Initialize the RNG.
        let rng = &mut rand_chacha::ChaCha20Rng::from_entropy();

        // Authorize the function.
        let function_authorization = process.authorize::<A, _>(
            &request.private_key,
            request.program_id,
            request.function_name,
            request.inputs.iter(),
            rng,
        )?;

        // Get the execution ID.
        let execution_id = function_authorization.to_execution_id()?;

        // Authorize the fee.
        let fee_authorization = process.authorize_fee_public::<A, _>(
            &request.private_key,
            *request.base_fee_in_microcredits,
            *request.priority_fee_in_microcredits,
            execution_id,
            rng,
        )?;

        // Construct the response.
        let response = AuthorizeResponse::<N> {
            function_authorization,
            fee_authorization,
        };

        // Return the response as JSON.
        Ok(serde_json::to_value(response)?)
    }
}
